//! AGE graph creation and index management (SPEC-017 P1-12).

use crate::error::{Result, StorageError};

use super::super::PostgresAGEGraphStorage;

impl PostgresAGEGraphStorage {
    pub(in crate::adapters::postgres::graph) async fn create_graph(&self) -> Result<()> {
        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        Self::setup_age_session_scoped(&mut conn, None).await?;

        let check_sql = format!(
            "SELECT 1 FROM ag_catalog.ag_graph WHERE name = '{}'",
            self.graph_name
        );

        let exists = sqlx::query(&check_sql)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Graph check failed: {}", e)))?;

        if exists.is_none() {
            let create_sql = format!(
                "SELECT * FROM ag_catalog.create_graph('{}')",
                self.graph_name
            );

            sqlx::query(&create_sql)
                .execute(&mut *conn)
                .await
                .map_err(|e| {
                    StorageError::Database(format!("Failed to create AGE graph: {}", e))
                })?;

            tracing::info!("Created AGE graph: {}", self.graph_name);
        }

        // SPEC-039: AGE creates label child tables lazily on first Cypher MERGE.
        // Native SQL reads/writes (get_nodes_batch, pg_upsert_nodes_batch_native)
        // require `"Node"` and `"EDGE"` to exist before any ingestion — bootstrap
        // them eagerly on fresh Docker / empty graphs.
        self.ensure_graph_labels(&mut conn).await?;

        Ok(())
    }

    /// Ensure EdgeQuake's canonical AGE labels exist (`Node`, `EDGE`).
    ///
    /// # First principle
    ///
    /// `create_graph()` only creates the graph schema + parent `_ag_label_*` tables.
    /// Child label tables are normally created on first Cypher use. SPEC-032/034
    /// batch SQL paths query `{graph}."Node"` directly — they fail on empty graphs
    /// with `relation does not exist` (SPEC-039 fresh Docker E2E).
    pub(in crate::adapters::postgres::graph) async fn ensure_graph_labels(
        &self,
        conn: &mut sqlx::PgConnection,
    ) -> Result<()> {
        self.ensure_age_label(conn, 'v', "Node").await?;
        self.ensure_age_label(conn, 'e', "EDGE").await?;
        Ok(())
    }

    async fn ensure_age_label(
        &self,
        conn: &mut sqlx::PgConnection,
        kind: char,
        label: &str,
    ) -> Result<()> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = $1 AND c.relname = $2
             )",
        )
        .bind(&self.graph_name)
        .bind(label)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or(false);

        if exists {
            tracing::debug!(
                graph = %self.graph_name,
                label,
                "AGE label table already exists"
            );
            return Ok(());
        }

        let create_fn = if kind == 'v' {
            "create_vlabel"
        } else {
            "create_elabel"
        };
        let create_sql = format!("SELECT {}('{}', '{}')", create_fn, self.graph_name, label);

        match sqlx::query(&create_sql).execute(&mut *conn).await {
            Ok(_) => {
                tracing::info!(
                    graph = %self.graph_name,
                    label,
                    "Created AGE label table (SPEC-039 bootstrap)"
                );
                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                // Race: another worker may have created the label between EXISTS and CREATE.
                if err_str.contains("already exists") || err_str.contains("duplicate") {
                    tracing::debug!(
                        graph = %self.graph_name,
                        label,
                        "AGE label already created by concurrent bootstrap"
                    );
                    return Ok(());
                }
                Err(StorageError::Database(format!(
                    "Failed to create AGE label {label} on graph {}: {e}",
                    self.graph_name
                )))
            }
        }
    }

    pub(in crate::adapters::postgres::graph) async fn ensure_indexes(&self) -> Result<()> {
        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        Self::setup_age_session_scoped(&mut conn, None).await?;

        let index_queries = [
            // ── "Node" label indexes (child table — contains all node rows) ──────────
            // REMOVED: idx_node_prop_node_id (agtype_access_operator form, 0 scans)
            //   → superseded by idx_node_prop_node_id_unique (UNIQUE btree, Migration 074)
            (
                "idx_node_props_gin",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_props_gin 
                       ON {}."Node" USING gin(properties)"#,
                    self.graph_name
                ),
            ),
            (
                "idx_node_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_id 
                       ON {}."Node" (id)"#,
                    self.graph_name
                ),
            ),
            (
                "idx_node_tenant_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_tenant_id 
                       ON {}."Node" (
                         (ag_catalog.agtype_to_json(properties)->>'tenant_id')
                       )"#,
                    self.graph_name
                ),
            ),
            (
                "idx_node_workspace_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_workspace_id 
                       ON {}."Node" (
                         (ag_catalog.agtype_to_json(properties)->>'workspace_id')
                       )"#,
                    self.graph_name
                ),
            ),
            // ── Migration 038: document-scoped source_ids (NAMEDATALEN-safe) ─────
            (
                "idx_node_source_id_expr",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_source_id_expr 
                       ON {}."Node" (
                         (ag_catalog.agtype_to_json(properties)->>'source_id')
                       )"#,
                    self.graph_name
                ),
            ),
            (
                "idx_node_source_ids_gin",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_node_source_ids_gin 
                       ON {}."Node" 
                       USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> 'source_ids') jsonb_ops)"#,
                    self.graph_name
                ),
            ),
            // ── "EDGE" label indexes ────────────────────────────────────────────────
            // REMOVED: idx_edge_start_end (composite, 0 scans — superseded by text-cast indexes)
            // REMOVED: idx_edge_props_gin (GIN on edge properties, 0 scans)
            (
                "idx_edge_start_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_start_id 
                       ON {}."EDGE" (start_id)"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_end_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_end_id 
                       ON {}."EDGE" (end_id)"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_source_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_source_id 
                       ON {}."EDGE" (
                         (ag_catalog.agtype_to_json(properties)->>'source_id')
                       )"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_target_id",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_target_id 
                       ON {}."EDGE" (
                         (ag_catalog.agtype_to_json(properties)->>'target_id')
                       )"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_source_ids_gin",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_source_ids_gin 
                       ON {}."EDGE" 
                       USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> 'source_ids') jsonb_ops)"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_start_id_text",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_start_id_text 
                       ON {}."EDGE" ((start_id::text))"#,
                    self.graph_name
                ),
            ),
            (
                "idx_edge_end_id_text",
                format!(
                    r#"CREATE INDEX IF NOT EXISTS idx_edge_end_id_text 
                       ON {}."EDGE" ((end_id::text))"#,
                    self.graph_name
                ),
            ),
            // REMOVED: All _ag_label_vertex and _ag_label_edge parent-table indexes.
            // WHY: Those are AGE inheritance parent tables with 0 rows. All data lives
            //      in the child label tables ("Node" and "EDGE"). Parent-table indexes
            //      are never scanned and cause write amplification on every INSERT.
            //      (SPEC-034 IMP-02, confirmed by pg_stat_user_indexes scan counts = 0)
        ];

        let mut indexes_created = 0;
        let mut indexes_skipped = 0;

        for (name, sql) in &index_queries {
            match sqlx::query(sql).execute(&mut *conn).await {
                Ok(_) => {
                    indexes_created += 1;
                    tracing::debug!("Created/verified index: {}", name);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("does not exist")
                        || err_str.contains("undefined_table")
                        || err_str.contains("relation")
                    {
                        indexes_skipped += 1;
                        tracing::debug!(
                            "Skipped index {} (table not yet created): {}",
                            name,
                            err_str
                        );
                    } else {
                        tracing::warn!(
                            error.source = "postgres_graph",
                            error.action = "create_index",
                            index = %name,
                            error.message = %e,
                            "Failed to create graph index"
                        );
                    }
                }
            }
        }

        if indexes_created > 0 {
            tracing::info!(
                "AGE graph indexes: {} created/verified, {} skipped (tables pending)",
                indexes_created,
                indexes_skipped
            );
        }

        // SPEC-062: denormalized text id columns — avoid per-row agtype_to_json on hot paths.
        self.ensure_eq_id_columns(&mut conn).await?;

        Ok(())
    }

    /// Add `eq_node_id` / `eq_source_id` / `eq_target_id` + btree UNIQUE + sync triggers.
    ///
    /// AGE label tables remain the source of truth for `properties`; these columns are
    /// maintained for native SQL JOIN / ON CONFLICT / degree aggregates (Wave 1).
    pub(in crate::adapters::postgres::graph) async fn ensure_eq_id_columns(
        &self,
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<()> {
        let g = &self.graph_name;
        let stmts = [
            format!(r#"ALTER TABLE {g}."Node" ADD COLUMN IF NOT EXISTS eq_node_id text"#),
            format!(r#"ALTER TABLE {g}."EDGE" ADD COLUMN IF NOT EXISTS eq_source_id text"#),
            format!(r#"ALTER TABLE {g}."EDGE" ADD COLUMN IF NOT EXISTS eq_target_id text"#),
            format!(
                r#"UPDATE {g}."Node" SET eq_node_id = ag_catalog.agtype_to_json(properties)->>'node_id'
                   WHERE eq_node_id IS NULL"#
            ),
            format!(
                r#"UPDATE {g}."EDGE" SET
                     eq_source_id = ag_catalog.agtype_to_json(properties)->>'source_id',
                     eq_target_id = ag_catalog.agtype_to_json(properties)->>'target_id'
                   WHERE eq_source_id IS NULL OR eq_target_id IS NULL"#
            ),
            format!(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_node_eq_node_id
                   ON {g}."Node" (eq_node_id) WHERE eq_node_id IS NOT NULL"#
            ),
            format!(
                r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_edge_eq_source_target
                   ON {g}."EDGE" (eq_source_id, eq_target_id)
                   WHERE eq_source_id IS NOT NULL AND eq_target_id IS NOT NULL"#
            ),
            format!(
                r#"CREATE INDEX IF NOT EXISTS idx_edge_eq_source_id
                   ON {g}."EDGE" (eq_source_id) WHERE eq_source_id IS NOT NULL"#
            ),
            format!(
                r#"CREATE INDEX IF NOT EXISTS idx_edge_eq_target_id
                   ON {g}."EDGE" (eq_target_id) WHERE eq_target_id IS NOT NULL"#
            ),
            // Triggers keep columns in sync when Cypher or legacy paths touch properties only.
            format!(
                r#"CREATE OR REPLACE FUNCTION {g}_eq_sync_node_id() RETURNS trigger AS $$
                   BEGIN
                     NEW.eq_node_id := ag_catalog.agtype_to_json(NEW.properties)->>'node_id';
                     RETURN NEW;
                   END;
                   $$ LANGUAGE plpgsql"#
            ),
            format!(
                r#"DROP TRIGGER IF EXISTS trg_eq_sync_node_id ON {g}."Node""#
            ),
            format!(
                r#"CREATE TRIGGER trg_eq_sync_node_id
                   BEFORE INSERT OR UPDATE OF properties ON {g}."Node"
                   FOR EACH ROW EXECUTE PROCEDURE {g}_eq_sync_node_id()"#
            ),
            format!(
                r#"CREATE OR REPLACE FUNCTION {g}_eq_sync_edge_ids() RETURNS trigger AS $$
                   BEGIN
                     NEW.eq_source_id := ag_catalog.agtype_to_json(NEW.properties)->>'source_id';
                     NEW.eq_target_id := ag_catalog.agtype_to_json(NEW.properties)->>'target_id';
                     RETURN NEW;
                   END;
                   $$ LANGUAGE plpgsql"#
            ),
            format!(
                r#"DROP TRIGGER IF EXISTS trg_eq_sync_edge_ids ON {g}."EDGE""#
            ),
            format!(
                r#"CREATE TRIGGER trg_eq_sync_edge_ids
                   BEFORE INSERT OR UPDATE OF properties ON {g}."EDGE"
                   FOR EACH ROW EXECUTE PROCEDURE {g}_eq_sync_edge_ids()"#
            ),
        ];

        for sql in &stmts {
            if let Err(e) = sqlx::query(sql).execute(&mut **conn).await {
                let msg = e.to_string();
                // Graph label tables may not exist yet on brand-new empty DBs.
                if msg.contains("does not exist") || msg.contains("undefined_table") {
                    tracing::debug!(error = %e, "SPEC-062 eq_id columns skipped (table pending)");
                    return Ok(());
                }
                tracing::warn!(
                    error = %e,
                    sql = %sql.chars().take(80).collect::<String>(),
                    "SPEC-062 eq_id DDL warning"
                );
            }
        }
        Ok(())
    }

    /// Bootstrap critical indexes CONCURRENTLY for existing databases (SPEC-032 W-01).
    ///
    /// # First Principle: Non-Blocking Index Creation
    ///
    /// `ensure_indexes()` uses `CREATE INDEX IF NOT EXISTS` which acquires a
    /// `ShareLock` on the label table for the duration of the build. For a graph
    /// with 100K+ nodes this blocks all concurrent writes for minutes.
    ///
    /// `CONCURRENTLY` builds the index without holding a table lock: reads and
    /// writes continue normally while the index is built in the background
    /// (PostgreSQL §CREATE INDEX §CONCURRENTLY).
    ///
    /// # Constraint
    ///
    /// `CREATE INDEX CONCURRENTLY` cannot run inside a transaction block.
    /// This function acquires a pool connection and sends commands directly,
    /// NOT inside a `BEGIN ... COMMIT` envelope.
    ///
    /// # When to Call
    ///
    /// Called ONCE at startup after `pg_initialize()` when the graph already
    /// has rows (existing databases). The `concurrent_indexes_bootstrapped`
    /// atomic flag prevents repeated runs.
    ///
    /// # Edge Cases Handled
    ///
    /// - Graph does not exist yet → returns Ok(()) silently (no label tables)
    /// - Index already exists → `IF NOT EXISTS` is a no-op
    /// - Index is INVALID (partial build interrupted) → detected via pg_index,
    ///   dropped and rebuilt
    /// - AGE not installed → returns Ok(()) silently
    /// - Timeout during build → index is left INVALID, next startup retries
    pub(in crate::adapters::postgres::graph) async fn bootstrap_concurrent_indexes(
        &self,
    ) -> Result<()> {
        // Quick check: is AGE available?
        let pool = self.pool.get().await?;

        let age_available: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
                .fetch_one(&pool)
                .await
                .unwrap_or(false);

        if !age_available {
            tracing::debug!("AGE not installed — skipping concurrent index bootstrap");
            return Ok(());
        }

        // Check if the Node label table exists (graph may not have been used yet)
        let node_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = $1 AND c.relname = 'Node'
             )",
        )
        .bind(&self.graph_name)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        if !node_table_exists {
            tracing::debug!(
                graph = %self.graph_name,
                "Node label table not yet created — skipping concurrent index bootstrap"
            );
            return Ok(());
        }

        // Check row count: if small, regular CREATE INDEX is fine; skip CONCURRENT
        let node_count: i64 = sqlx::query_scalar(
            r#"SELECT reltuples::bigint FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = $1 AND c.relname = 'Node'"#,
        )
        .bind(&self.graph_name)
        .fetch_one(&pool)
        .await
        .unwrap_or(0);

        // Below 10K rows: regular CREATE INDEX is fast enough (< 1 second).
        // Above 10K rows: use CONCURRENT to avoid blocking writes.
        const CONCURRENT_THRESHOLD: i64 = 10_000;
        let use_concurrent = node_count >= CONCURRENT_THRESHOLD;

        tracing::info!(
            graph = %self.graph_name,
            node_count,
            use_concurrent,
            "AGE graph bootstrap: checking critical indexes"
        );

        // ── Critical unique index for MERGE/ON CONFLICT performance ────────────
        // First principles: if a VALID unique index already exists, skip the
        // O(N) expression-based DELETE dedup on every boot. On a 140k-node graph
        // that DELETE was blocking HTTP listen for minutes (make-dev hang).
        // Dedup is only required when we are about to CREATE the unique index.
        let node_idx_name = "idx_node_prop_node_id_unique";
        let node_idx_state = self.index_validity(&pool, node_idx_name).await;
        if node_idx_state == Some(true) {
            tracing::debug!(
                index = %node_idx_name,
                graph = %self.graph_name,
                "Bootstrap: node unique index already valid — skip dedup + create"
            );
        } else {
            // WHY UNIQUE: Migration 074/083. Dedup first — CREATE UNIQUE fails on
            // duplicates and leaves the graph without ON CONFLICT support (edge #3).
            if let Err(e) = self.dedup_nodes_for_unique_index(&pool).await {
                tracing::warn!(
                    graph = %self.graph_name,
                    error = %e,
                    "Bootstrap: node dedup before UNIQUE index failed (non-fatal)"
                );
            }
            let node_idx_sql = format!(
                r#"CREATE UNIQUE INDEX {concurrent} IF NOT EXISTS {name}
                   ON {graph}."Node"
                   ((ag_catalog.agtype_to_json(properties)->>'node_id'))"#,
                concurrent = if use_concurrent { "CONCURRENTLY" } else { "" },
                name = node_idx_name,
                graph = self.graph_name,
            );
            self.ensure_critical_index_concurrent(
                &pool,
                node_idx_name,
                &node_idx_sql,
                use_concurrent,
            )
            .await;
        }

        // EDGE source_id + target_id composite index
        let edge_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
               SELECT 1 FROM pg_class c
               JOIN pg_namespace n ON n.oid = c.relnamespace
               WHERE n.nspname = $1 AND c.relname = 'EDGE'
             )",
        )
        .bind(&self.graph_name)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        if edge_table_exists {
            let edge_idx_name = "idx_edge_source_target_unique";
            let edge_idx_state = self.index_validity(&pool, edge_idx_name).await;
            if edge_idx_state == Some(true) {
                tracing::debug!(
                    index = %edge_idx_name,
                    graph = %self.graph_name,
                    "Bootstrap: edge unique index already valid — skip dedup + create"
                );
            } else {
                if let Err(e) = self.dedup_edges_for_unique_index(&pool).await {
                    tracing::warn!(
                        graph = %self.graph_name,
                        error = %e,
                        "Bootstrap: edge dedup before UNIQUE index failed (non-fatal)"
                    );
                }
                // WHY UNIQUE: Migration 074/083 for pg_upsert_edges_batch_native ON CONFLICT.
                self.ensure_critical_index_concurrent(
                    &pool,
                    edge_idx_name,
                    &format!(
                        r#"CREATE UNIQUE INDEX {concurrent} IF NOT EXISTS {name}
                           ON {graph}."EDGE"
                           (
                             (ag_catalog.agtype_to_json(properties)->>'source_id'),
                             (ag_catalog.agtype_to_json(properties)->>'target_id')
                           )"#,
                        concurrent = if use_concurrent { "CONCURRENTLY" } else { "" },
                        name = edge_idx_name,
                        graph = self.graph_name,
                    ),
                    use_concurrent,
                )
                .await;
            }
        }

        Ok(())
    }

    /// `Some(true)` = valid index, `Some(false)` = INVALID, `None` = missing.
    async fn index_validity(&self, pool: &sqlx::PgPool, index_name: &str) -> Option<bool> {
        sqlx::query_scalar(
            "SELECT i.indisvalid FROM pg_index i
             JOIN pg_class c ON c.oid = i.indexrelid
             JOIN pg_namespace n ON n.oid = c.relnamespace
             WHERE c.relname = $1 AND n.nspname = $2",
        )
        .bind(index_name)
        .bind(&self.graph_name)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
    }

    /// Dedup Node rows by node_id (keep max ctid) + drop NULL/empty node_id.
    async fn dedup_nodes_for_unique_index(&self, pool: &sqlx::PgPool) -> Result<()> {
        let graph = &self.graph_name;
        let delete_null = format!(
            r#"DELETE FROM {graph}."Node"
               WHERE COALESCE(ag_catalog.agtype_to_json(properties)->>'node_id', '') = ''"#
        );
        sqlx::query(&delete_null)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Database(format!("node null-id cleanup failed: {e}")))?;
        let dedup = format!(
            r#"DELETE FROM {graph}."Node"
               WHERE ctid NOT IN (
                 SELECT max(ctid) FROM {graph}."Node"
                 GROUP BY ag_catalog.agtype_to_json(properties)->>'node_id'
               )"#
        );
        sqlx::query(&dedup)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Database(format!("node dedup failed: {e}")))?;
        Ok(())
    }

    /// Dedup EDGE rows by (source_id, target_id), keep max ctid.
    async fn dedup_edges_for_unique_index(&self, pool: &sqlx::PgPool) -> Result<()> {
        let graph = &self.graph_name;
        let dedup = format!(
            r#"DELETE FROM {graph}."EDGE"
               WHERE ctid NOT IN (
                 SELECT max(ctid) FROM {graph}."EDGE"
                 GROUP BY (ag_catalog.agtype_to_json(properties)->>'source_id'),
                          (ag_catalog.agtype_to_json(properties)->>'target_id')
               )"#
        );
        sqlx::query(&dedup)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Database(format!("edge dedup failed: {e}")))?;
        Ok(())
    }

    /// Create or repair one critical index, handling INVALID state.
    ///
    /// Caller must already have decided the index is missing/INVALID (and run
    /// dedup if needed). This still re-checks validity for race safety.
    async fn ensure_critical_index_concurrent(
        &self,
        pool: &sqlx::PgPool,
        index_name: &str,
        create_sql: &str,
        is_concurrent: bool,
    ) {
        // Scope to this graph's schema — unqualified DROP can hit the wrong index
        // when multiple AGE graphs share the same index name (edge case #15).
        let index_state = self.index_validity(pool, index_name).await;

        match index_state {
            Some(true) => {
                tracing::debug!(index = %index_name, "Bootstrap: index already valid, skip");
                return;
            }
            Some(false) => {
                tracing::warn!(
                    index = %index_name,
                    graph = %self.graph_name,
                    "Bootstrap: found INVALID index (interrupted build), dropping and rebuilding"
                );
                let drop_sql = format!(
                    r#"DROP INDEX CONCURRENTLY IF EXISTS "{}"."{}""#,
                    self.graph_name, index_name
                );
                if let Err(e) = sqlx::query(&drop_sql).execute(pool).await {
                    tracing::warn!(
                        index = %index_name,
                        error = %e,
                        "Bootstrap: failed to drop INVALID index"
                    );
                    return;
                }
            }
            None => {
                tracing::debug!(index = %index_name, "Bootstrap: index missing, creating");
            }
        }

        match sqlx::query(create_sql).execute(pool).await {
            Ok(_) => {
                tracing::info!(
                    index = %index_name,
                    concurrent = is_concurrent,
                    "Bootstrap: critical index created successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    index = %index_name,
                    error = %e,
                    concurrent = is_concurrent,
                    "Bootstrap: failed to create critical index (non-fatal, will retry next startup)"
                );
            }
        }
    }
}
