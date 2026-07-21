//! DDL and schema maintenance for [`super::PgVectorStorage`].

use super::super::capabilities::{AnnIndexPolicy, HNSW_MAX_DIM_HALFVEC};
use super::super::config::VectorIndexType;
use super::super::hnsw_runtime_policy::HnswRuntimePolicy;
use super::super::row_count_stats::{self, RowCountStatsConfig};
use super::super::schema;
use super::PgVectorStorage;
use crate::error::{Result, StorageError};

impl PgVectorStorage {
    /// Create the vectors table and indexes.
    pub(crate) async fn create_table(&self) -> Result<()> {
        let pool = self.pool.get().await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
            .execute(&pool)
            .await
            .map_err(|e| {
                StorageError::Database(format!("Failed to create vector extension: {}", e))
            })?;

        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        let emb_type = self.embedding_pg_type();
        let opclass = self.embedding_opclass();
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                embedding {}({}) NOT NULL,
                metadata JSONB DEFAULT '{{}}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
            self.table_name, emb_type, self.dimension
        );

        sqlx::query(&sql).execute(&pool).await.map_err(|e| {
            StorageError::Database(format!("Failed to create vectors table: {}", e))
        })?;

        let index_sql = match self.index_type {
            VectorIndexType::IVFFlat if policy.hnsw_viable => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING ivfflat (embedding {}) WITH (lists = {})",
                self.prefix, self.table_name, opclass, self.ivfflat_lists
            ),
            VectorIndexType::HNSW if policy.hnsw_viable => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING hnsw (embedding {}) WITH (m = {}, ef_construction = {})",
                self.prefix, self.table_name, opclass, self.hnsw_m, self.hnsw_ef_construction
            ),
            VectorIndexType::IVFFlat | VectorIndexType::HNSW => {
                tracing::warn!(
                    table = %self.table_name,
                    dimension = self.dimension,
                    max_hnsw_dim = HNSW_MAX_DIM_HALFVEC,
                    "Skipping ANN index — embedding dimension exceeds pgvector HNSW limits (issue #275)"
                );
                String::new()
            }
            VectorIndexType::None => String::new(),
        };

        if !index_sql.is_empty() {
            // SPEC-046 OPS-P0.3: fail-closed — never swallow ANN index DDL errors.
            // Missing HNSW silently degrades to seq-scan (latency/recall cliff).
            sqlx::query(&index_sql).execute(&pool).await.map_err(|e| {
                StorageError::Database(format!(
                    "Failed to create ANN index on {}: {}",
                    self.table_name, e
                ))
            })?;
        }

        // SPEC-034 IMP-08: Vector metadata GIN index removed.
        // WHY: 0 query scans — all metadata lookups use metadata->>'key' = value
        // (equality on extracted text), served by doc_id_idx / tenant_ws_idx btrees.
        // This was 13 MB per workspace with zero benefit.
        // To restore: uncomment the line below.
        // sqlx::query(&format!(
        //     "CREATE INDEX IF NOT EXISTS eq_{}_vectors_metadata_idx ON {} USING GIN (metadata jsonb_path_ops)",
        //     self.prefix, self.table_name
        // )).execute(&pool).await.ok();

        let add_cols = format!(
            r#"
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS document_id TEXT;
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS tenant_id TEXT;
            ALTER TABLE {} ADD COLUMN IF NOT EXISTS workspace_id TEXT
            "#,
            self.table_name, self.table_name, self.table_name
        );
        for stmt in add_cols.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(stmt.trim()).execute(&pool).await.ok();
        }

        let doc_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_doc_id_idx ON {} (document_id) WHERE document_id IS NOT NULL",
            self.prefix, self.table_name
        );
        sqlx::query(&doc_idx).execute(&pool).await.ok();

        let tenant_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_tenant_ws_idx ON {} (tenant_id, workspace_id) WHERE tenant_id IS NOT NULL",
            self.prefix, self.table_name
        );
        sqlx::query(&tenant_idx).execute(&pool).await.ok();

        self.ensure_content_fts(&pool).await?;

        self.ensure_row_count_stats(&pool).await?;

        Ok(())
    }

    pub(crate) async fn ensure_row_count_stats(&self, pool: &sqlx::PgPool) -> Result<()> {
        row_count_stats::ensure_row_count_stats(
            pool,
            &RowCountStatsConfig {
                prefix: &self.prefix,
                table_name: &self.table_name,
                stats_table_name: &self.stats_table_name,
                kind: "vectors",
            },
        )
        .await
    }

    /// Drop the vectors table if it exists.
    pub async fn drop_table(&self) -> Result<()> {
        let pool = self.pool.get().await?;

        let sql = format!("DROP TABLE IF EXISTS {} CASCADE", self.table_name);

        sqlx::query(&sql)
            .execute(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("Failed to drop vectors table: {}", e)))?;

        tracing::info!(
            table = %self.table_name,
            "Dropped vector table for dimension migration"
        );

        Ok(())
    }

    /// Check if the table exists in the database.
    pub async fn table_exists(&self) -> Result<bool> {
        let pool = match self.pool.get().await {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };

        schema::relation_exists(&pool, &self.table_name).await
    }

    /// Name of the ANN embedding index for this table (SPEC-046 OPS-P0.3).
    pub fn ann_index_name(&self) -> String {
        format!("eq_{}_vectors_embedding_idx", self.prefix)
    }

    /// SPEC-062: build HNSW/IVFFlat after a deferred bulk load (`VectorIndexType::None`).
    ///
    /// Cold ingest pattern: create table without ANN → upsert heap rows → `ensure_ann_index`.
    /// Online ingest still creates HNSW at `initialize()` time (pays insert tax).
    pub async fn ensure_ann_index(&self) -> Result<()> {
        let pool = self.pool.get().await?;
        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        if !policy.hnsw_viable {
            return Ok(());
        }
        let opclass = self.embedding_opclass();
        let index_sql = match self.index_type {
            VectorIndexType::None | VectorIndexType::HNSW => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING hnsw (embedding {}) WITH (m = {}, ef_construction = {})",
                self.prefix, self.table_name, opclass, self.hnsw_m, self.hnsw_ef_construction
            ),
            VectorIndexType::IVFFlat => format!(
                "CREATE INDEX IF NOT EXISTS eq_{}_vectors_embedding_idx ON {} USING ivfflat (embedding {}) WITH (lists = {})",
                self.prefix, self.table_name, opclass, self.ivfflat_lists
            ),
        };
        sqlx::query(&index_sql).execute(&pool).await.map_err(|e| {
            StorageError::Database(format!(
                "Failed to ensure ANN index on {}: {}",
                self.table_name, e
            ))
        })?;
        self.mark_deferred_ann_ready();
        Ok(())
    }

    /// SPEC-064 Wave 2: workspace-scoped partial HNSW (opt-in / explicit hot-workspace path).
    ///
    /// Builds `WHERE workspace_id = $ws` so filtered ANN walks a smaller graph instead of
    /// over-filtering a global HNSW via `hnsw.iterative_scan`. Prefer for hot workspaces;
    /// keep global `ensure_ann_index` for sparse/small workspaces.
    ///
    /// Gate: call after heap load. Battle harness drops the global ANN first so the planner
    /// cannot prefer the larger index. Production callers should set
    /// `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1` before creating hot-workspace partials.
    pub async fn ensure_partial_hnsw_for_workspace(&self, workspace_id: &str) -> Result<()> {
        if workspace_id.is_empty() {
            return Err(StorageError::Database(
                "ensure_partial_hnsw_for_workspace: empty workspace_id".into(),
            ));
        }
        let pool = self.pool.get().await?;
        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        if !policy.hnsw_viable {
            return Ok(());
        }
        let opclass = self.embedding_opclass();
        let index_name = self.partial_ann_index_name(workspace_id);
        let lit = sql_string_literal(workspace_id);
        let index_sql = format!(
            "CREATE INDEX IF NOT EXISTS {index_name} ON {} USING hnsw (embedding {}) WITH (m = {}, ef_construction = {}) WHERE workspace_id = {lit}",
            self.table_name, opclass, self.hnsw_m, self.hnsw_ef_construction
        );
        sqlx::query(&index_sql).execute(&pool).await.map_err(|e| {
            StorageError::Database(format!(
                "Failed to create partial HNSW {index_name} on {}: {}",
                self.table_name, e
            ))
        })?;
        self.mark_deferred_ann_ready();
        tracing::info!(
            table = %self.table_name,
            index = %index_name,
            workspace_id,
            "Ensured workspace partial HNSW (SPEC-064)"
        );
        Ok(())
    }

    /// Drop the global ANN index (battle / rebuild helper). Partial indexes are left intact.
    pub async fn drop_global_ann_index(&self) -> Result<()> {
        let pool = self.pool.get().await?;
        let index_name = self.ann_index_name();
        let sql = format!("DROP INDEX IF EXISTS {index_name}");
        sqlx::query(&sql).execute(&pool).await.map_err(|e| {
            StorageError::Database(format!("Failed to drop ANN index {index_name}: {e}"))
        })?;
        Ok(())
    }

    /// Name of a workspace partial HNSW index.
    pub fn partial_ann_index_name(&self, workspace_id: &str) -> String {
        format!(
            "eq_{}_vectors_hnsw_ws_{}",
            self.prefix,
            workspace_index_slug(workspace_id)
        )
    }

    /// True when HNSW/IVFFlat index exists (fail-closed readiness probe).
    ///
    /// SPEC-065: accepts **global** ANN **or** any workspace partial HNSW on this table.
    /// `VectorIndexType::None` (deferred create) is not a readiness failure until/unless
    /// callers expect an index — then catalog probe still reports truth.
    pub async fn ann_index_exists(&self) -> Result<bool> {
        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        if !policy.hnsw_viable {
            return Ok(true); // ANN not expected
        }
        if matches!(self.index_type, VectorIndexType::None)
            && !self
                .deferred_ann_ready
                .load(std::sync::atomic::Ordering::Acquire)
        {
            // Deferred heap load — not a readiness failure yet.
            return Ok(true);
        }
        let pool = self.pool.get().await?;
        let global = self.ann_index_name();
        let partial_prefix = format!("eq_{}_vectors_hnsw_ws_", self.prefix);
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM pg_indexes
                WHERE schemaname = 'public'
                  AND (indexname = $1 OR indexname LIKE $2)
            )",
        )
        .bind(&global)
        .bind(format!("{partial_prefix}%"))
        .fetch_one(&pool)
        .await
        .map_err(|e| StorageError::Database(format!("ann_index_exists probe failed: {e}")))?;
        Ok(exists)
    }

    /// True when this table is a dedicated per-workspace table (`*_ws_*` namespace).
    pub fn is_dedicated_workspace_table(&self) -> bool {
        self.prefix.contains("_ws_") || self.table_name.contains("_ws_")
    }

    /// Count rows for a workspace (denorm column).
    pub async fn count_workspace_rows(&self, workspace_id: &str) -> Result<u64> {
        let pool = self.pool.get().await?;
        let sql = format!(
            "SELECT COUNT(*)::bigint FROM {} WHERE workspace_id = $1",
            self.table_name
        );
        let n: i64 = sqlx::query_scalar(&sql)
            .bind(workspace_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("count_workspace_rows failed: {e}")))?;
        Ok(n.max(0) as u64)
    }

    /// True when the workspace partial HNSW exists in the catalog.
    pub async fn partial_ann_index_exists(&self, workspace_id: &str) -> Result<bool> {
        let pool = self.pool.get().await?;
        let index_name = self.partial_ann_index_name(workspace_id);
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                SELECT 1 FROM pg_indexes
                WHERE schemaname = 'public' AND indexname = $1
            )",
        )
        .bind(&index_name)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            StorageError::Database(format!("partial_ann_index_exists probe failed: {e}"))
        })?;
        Ok(exists)
    }

    /// SPEC-065: productized Wave-2 path — create partial HNSW for a hot workspace
    /// when opt-in is on, table is shared (multi-WS), and row count ≥ threshold.
    ///
    /// Dedicated per-workspace tables are a no-op (already isolated). Keeps global HNSW.
    pub async fn ensure_hot_workspace_ann(&self, workspace_id: &str) -> Result<bool> {
        let runtime = HnswRuntimePolicy::from_env();
        if !runtime.partial_by_workspace {
            return Ok(false);
        }
        if workspace_id.is_empty() {
            return Err(StorageError::Database(
                "ensure_hot_workspace_ann: empty workspace_id".into(),
            ));
        }
        if self.is_dedicated_workspace_table() {
            tracing::debug!(
                table = %self.table_name,
                "Skipping partial HNSW — dedicated workspace table"
            );
            return Ok(false);
        }
        if self.partial_ann_index_exists(workspace_id).await? {
            self.mark_deferred_ann_ready();
            return Ok(false);
        }
        let rows = self.count_workspace_rows(workspace_id).await?;
        if rows < runtime.partial_min_rows {
            tracing::debug!(
                workspace_id,
                rows,
                min = runtime.partial_min_rows,
                "Skipping partial HNSW — below row threshold"
            );
            return Ok(false);
        }
        let policy = AnnIndexPolicy::resolve(self.dimension, self.storage_mode);
        if !policy.hnsw_viable {
            return Ok(false);
        }
        self.ensure_partial_hnsw_for_workspace(workspace_id).await?;
        // Fail-closed: catalog must show the partial after DDL (no silent seq-scan path).
        if !self.partial_ann_index_exists(workspace_id).await? {
            return Err(StorageError::Database(format!(
                "ensure_hot_workspace_ann: partial HNSW missing after CREATE for workspace {workspace_id}"
            )));
        }
        Ok(true)
    }

    /// Count vector tables that have no HNSW/IVFFlat index (bootstrap readiness).
    pub async fn count_vector_tables_missing_ann_index(pool: &sqlx::PgPool) -> Result<usize> {
        let missing: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM pg_tables t
            WHERE t.schemaname = 'public'
              AND t.tablename LIKE 'eq\_%\_vectors' ESCAPE '\'
              AND t.tablename NOT LIKE '%\_stats'
              AND NOT EXISTS (
                SELECT 1
                FROM pg_indexes i
                WHERE i.schemaname = 'public'
                  AND i.tablename = t.tablename
                  AND (
                    i.indexname = (t.tablename || '_embedding_idx')
                    OR i.indexdef ILIKE '% USING hnsw %'
                    OR i.indexdef ILIKE '% USING ivfflat %'
                  )
              )
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(|e| StorageError::Database(format!("missing ANN index scan failed: {e}")))?;
        Ok(missing.max(0) as usize)
    }

    /// Add writable GIN-backed `content_tsv` for native Postgres FTS (SPEC-023 I10 / SPEC-058).
    ///
    /// WHY writable (not GENERATED): chunk SSOT is KV via `content_ref`; generated
    /// columns from `metadata->>'content'` stay empty and block coalesce fallthrough.
    pub(crate) async fn ensure_content_fts(&self, pool: &sqlx::PgPool) -> Result<()> {
        let table_only = self
            .table_name
            .split('.')
            .next_back()
            .unwrap_or(&self.table_name);

        let ensure_col = format!(
            r#"
            DO $$
            DECLARE
                gen text;
            BEGIN
                SELECT a.attgenerated INTO gen
                FROM pg_attribute a
                JOIN pg_class c ON c.oid = a.attrelid
                JOIN pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname = 'public'
                  AND c.relname = '{table_only}'
                  AND a.attname = 'content_tsv'
                  AND a.attnum > 0
                  AND NOT a.attisdropped;

                IF gen IS NULL THEN
                    ALTER TABLE {table}
                    ADD COLUMN content_tsv TSVECTOR;
                ELSIF gen <> '' THEN
                    ALTER TABLE {table} DROP COLUMN content_tsv;
                    ALTER TABLE {table}
                    ADD COLUMN content_tsv TSVECTOR;
                END IF;
            END $$;
            "#,
            table_only = table_only,
            table = self.table_name
        );

        sqlx::query(&ensure_col).execute(pool).await.ok();

        let fts_idx = format!(
            "CREATE INDEX IF NOT EXISTS eq_{}_vectors_content_tsv_idx ON {} USING GIN (content_tsv)",
            self.prefix, self.table_name
        );
        sqlx::query(&fts_idx).execute(pool).await.ok();

        Ok(())
    }
}

/// Stable, index-safe slug for workspace_id (alnum/_ + short hash).
fn workspace_index_slug(workspace_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let safe: String = workspace_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .take(24)
        .collect();
    let mut hasher = DefaultHasher::new();
    workspace_id.hash(&mut hasher);
    format!("{safe}_{:04x}", hasher.finish() as u16)
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_index_slug_is_stable_and_safe() {
        let a = workspace_index_slug("ws-a");
        let b = workspace_index_slug("ws-a");
        assert_eq!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        assert!(!workspace_index_slug("ws';a").contains('\''));
    }

    #[test]
    fn sql_string_literal_escapes_quotes() {
        assert_eq!(sql_string_literal("o'reilly"), "'o''reilly'");
    }
}
