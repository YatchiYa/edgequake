//! PostgreSQL implementation of `RelationalEntitySink` (SPEC-021 P3-01/P3-02).
//!
//! Under `typed_embeddings`, this sink is always enabled (fleet embeddings FK to
//! `entities` / `relationships`). Entity names are stored **bare** (workspace
//! isolation is the `workspace_id` column) so fleet mirror lookups by
//! `entity:NAME` resolve.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use edgequake_pipeline::{
    EntitySinkRow, NoopEntitySink, RelationalEntitySink, RelationshipSinkRow,
};
use edgequake_storage::EntityId;
use sqlx::PgPool;
use tracing::{debug, info, warn};

/// PostgreSQL-backed relational entity (and relationship) sink.
pub struct PostgresEntitySink {
    pool: Arc<PgPool>,
    /// When true (typed vector backend), SQL failures fail closed.
    fail_closed: bool,
}

impl PostgresEntitySink {
    /// Create a fail-open sink (legacy CQRS dual-write mode).
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            fail_closed: false,
        }
    }

    /// Create a fail-closed sink required for typed fleet FK spine.
    pub fn new_fail_closed(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            fail_closed: true,
        }
    }

    /// Resolve sink for the current vector backend + entity_sync_mode.
    ///
    /// - `typed_embeddings` / `chunk_embeddings`: always `PostgresEntitySink`
    ///   (fail-closed) so fleet mirror has relational FKs.
    /// - otherwise: honor `entity_sync_mode` dual_write|full (fail-open).
    pub async fn create_for_runtime(pool: Arc<PgPool>) -> Arc<dyn RelationalEntitySink> {
        if edgequake_storage::vector_backend_reads_typed(
            edgequake_storage::vector_backend_from_env(),
        ) {
            info!("typed vector backend: forcing PostgresEntitySink (fleet FK spine, fail-closed)");
            return Arc::new(Self::new_fail_closed(pool));
        }
        Self::create_if_enabled(pool).await
    }

    /// Create the appropriate sink based on `entity_sync_mode` in server_config.
    ///
    /// Returns:
    /// - `PostgresEntitySink` when mode is `dual_write` or `full`
    /// - `NoopEntitySink` when mode is `disabled` or config is absent
    pub async fn create_if_enabled(pool: Arc<PgPool>) -> Arc<dyn RelationalEntitySink> {
        let mode: Option<String> = sqlx::query_scalar(
            "SELECT value::text FROM server_config WHERE key = 'entity_sync_mode'",
        )
        .fetch_optional(pool.as_ref())
        .await
        .unwrap_or(None);

        let mode_str = mode.as_deref().unwrap_or("\"disabled\"");
        let enabled = mode_str.contains("dual_write") || mode_str.contains("full");

        if enabled {
            tracing::info!(
                entity_sync_mode = %mode_str,
                "CQRS entity dual-write ENABLED (SPEC-021 P3-01)"
            );
            Arc::new(Self::new(pool))
        } else {
            tracing::info!(
                entity_sync_mode = %mode_str,
                "CQRS entity dual-write disabled (entity_sync_mode != dual_write|full)"
            );
            Arc::new(NoopEntitySink)
        }
    }

    fn map_sql_result(
        &self,
        label: &str,
        result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
    ) -> edgequake_pipeline::Result<()> {
        match result {
            Ok(_) => {
                debug!(target = %label, "Relational sink OK");
                Ok(())
            }
            Err(e) => {
                if self.fail_closed {
                    return Err(edgequake_pipeline::PipelineError::StorageError(
                        edgequake_storage::error::StorageError::Database(format!(
                            "relational sink failed ({label}): {e}"
                        )),
                    ));
                }
                warn!(target = %label, error = %e, "Relational sink failed (best-effort)");
                Ok(())
            }
        }
    }
}

#[async_trait]
impl RelationalEntitySink for PostgresEntitySink {
    async fn upsert_entity(
        &self,
        name: &str,
        entity_type: &str,
        description: &str,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
        source_chunk_ids: &[String],
    ) -> edgequake_pipeline::Result<()> {
        let bare = EntityId::bare_name_from_graph_node_id(name);
        if bare.is_empty() {
            return Ok(());
        }
        let tenant_uuid: Option<uuid::Uuid> = tenant_id.and_then(|t| t.parse().ok());
        let workspace_uuid: Option<uuid::Uuid> = workspace_id.and_then(|w| w.parse().ok());

        let result = sqlx::query(
            r#"INSERT INTO entities
                   (name, entity_type, description, tenant_id, workspace_id,
                    source_chunk_ids, sync_status, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, 'synced', NOW(), NOW())
               ON CONFLICT (tenant_id, workspace_id, name) DO UPDATE SET
                   entity_type      = EXCLUDED.entity_type,
                   description      = EXCLUDED.description,
                   source_chunk_ids = (
                       SELECT array_agg(DISTINCT elem)
                       FROM unnest(entities.source_chunk_ids || EXCLUDED.source_chunk_ids) AS t(elem)
                   ),
                   sync_status = 'synced',
                   updated_at  = NOW()"#,
        )
        .bind(bare)
        .bind(entity_type)
        .bind(description)
        .bind(tenant_uuid)
        .bind(workspace_uuid)
        .bind(source_chunk_ids)
        .execute(self.pool.as_ref())
        .await;

        self.map_sql_result(bare, result)
    }

    /// SPEC-091 IP1: one UNNEST upsert for the whole entity batch (LAW-IP2).
    async fn upsert_entities_batch(
        &self,
        rows: &[EntitySinkRow],
    ) -> edgequake_pipeline::Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Collapse duplicate (tenant, workspace, name) keys — Postgres rejects
        // "ON CONFLICT DO UPDATE cannot affect row a second time" in one INSERT.
        let mut by_key: HashMap<(Option<uuid::Uuid>, Option<uuid::Uuid>, String), EntitySinkRow> =
            HashMap::new();
        for row in rows {
            let bare = EntityId::bare_name_from_graph_node_id(&row.name);
            if bare.is_empty() {
                continue;
            }
            let tenant = row.tenant_id.as_deref().and_then(|t| t.parse().ok());
            let workspace = row.workspace_id.as_deref().and_then(|w| w.parse().ok());
            let key = (tenant, workspace, bare.to_string());
            by_key
                .entry(key)
                .and_modify(|existing| {
                    existing.entity_type = row.entity_type.clone();
                    existing.description = row.description.clone();
                    for s in &row.source_chunk_ids {
                        if !existing.source_chunk_ids.contains(s) {
                            existing.source_chunk_ids.push(s.clone());
                        }
                    }
                })
                .or_insert_with(|| EntitySinkRow {
                    name: bare.to_string(),
                    entity_type: row.entity_type.clone(),
                    description: row.description.clone(),
                    tenant_id: row.tenant_id.clone(),
                    workspace_id: row.workspace_id.clone(),
                    source_chunk_ids: row.source_chunk_ids.clone(),
                });
        }
        if by_key.is_empty() {
            return Ok(());
        }

        let mut names: Vec<String> = Vec::with_capacity(by_key.len());
        let mut types: Vec<String> = Vec::with_capacity(by_key.len());
        let mut descs: Vec<String> = Vec::with_capacity(by_key.len());
        let mut tenants: Vec<Option<uuid::Uuid>> = Vec::with_capacity(by_key.len());
        let mut workspaces: Vec<Option<uuid::Uuid>> = Vec::with_capacity(by_key.len());
        let mut sources_json: Vec<serde_json::Value> = Vec::with_capacity(by_key.len());
        for ((tenant, workspace, name), row) in by_key {
            names.push(name);
            types.push(row.entity_type);
            descs.push(row.description);
            tenants.push(tenant);
            workspaces.push(workspace);
            sources_json.push(serde_json::json!(row.source_chunk_ids));
        }

        let result = sqlx::query(
            r#"
            INSERT INTO entities
                (name, entity_type, description, tenant_id, workspace_id,
                 source_chunk_ids, sync_status, created_at, updated_at)
            SELECT
                n, t, d, tn, ws,
                COALESCE(
                    (SELECT array_agg(x) FROM jsonb_array_elements_text(src) AS x),
                    '{}'::text[]
                ),
                'synced', NOW(), NOW()
            FROM unnest(
                $1::text[],
                $2::text[],
                $3::text[],
                $4::uuid[],
                $5::uuid[],
                $6::jsonb[]
            ) AS u(n, t, d, tn, ws, src)
            ON CONFLICT (tenant_id, workspace_id, name) DO UPDATE SET
                entity_type      = EXCLUDED.entity_type,
                description      = EXCLUDED.description,
                source_chunk_ids = (
                    SELECT array_agg(DISTINCT elem)
                    FROM unnest(entities.source_chunk_ids || EXCLUDED.source_chunk_ids) AS t(elem)
                ),
                sync_status = 'synced',
                updated_at  = NOW()
            "#,
        )
        .bind(&names)
        .bind(&types)
        .bind(&descs)
        .bind(&tenants)
        .bind(&workspaces)
        .bind(&sources_json)
        .execute(self.pool.as_ref())
        .await;

        self.map_sql_result("entities_batch", result)
    }

    async fn upsert_relationship(
        &self,
        source_name: &str,
        target_name: &str,
        relation_type: &str,
        description: &str,
        weight: f32,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> edgequake_pipeline::Result<()> {
        let src = EntityId::bare_name_from_graph_node_id(source_name);
        let tgt = EntityId::bare_name_from_graph_node_id(target_name);
        if src.is_empty() || tgt.is_empty() {
            return Ok(());
        }
        let rel_type = if relation_type.trim().is_empty() {
            "RELATED_TO".to_string()
        } else {
            relation_type.trim().to_ascii_uppercase()
        };
        let tenant_uuid: Option<uuid::Uuid> = tenant_id.and_then(|t| t.parse().ok());
        let workspace_uuid: Option<uuid::Uuid> = workspace_id.and_then(|w| w.parse().ok());

        // Resolve endpoints (bare preferred; tolerate legacy scoped names).
        let src_id: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM entities
               WHERE workspace_id = $2
                 AND (name = $1 OR name = ($2::text || '::' || $1))
               ORDER BY CASE WHEN name = $1 THEN 0 ELSE 1 END
               LIMIT 1"#,
        )
        .bind(src)
        .bind(workspace_uuid)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            edgequake_pipeline::PipelineError::StorageError(
                edgequake_storage::error::StorageError::Database(e.to_string()),
            )
        })?;

        let tgt_id: Option<uuid::Uuid> = sqlx::query_scalar(
            r#"SELECT id FROM entities
               WHERE workspace_id = $2
                 AND (name = $1 OR name = ($2::text || '::' || $1))
               ORDER BY CASE WHEN name = $1 THEN 0 ELSE 1 END
               LIMIT 1"#,
        )
        .bind(tgt)
        .bind(workspace_uuid)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| {
            edgequake_pipeline::PipelineError::StorageError(
                edgequake_storage::error::StorageError::Database(e.to_string()),
            )
        })?;

        let (Some(source_id), Some(target_id)) = (src_id, tgt_id) else {
            let msg = format!(
                "relational relationship upsert skipped: missing entity FK \
                 src={src} tgt={tgt} workspace={workspace_uuid:?}"
            );
            if self.fail_closed {
                return Err(edgequake_pipeline::PipelineError::StorageError(
                    edgequake_storage::error::StorageError::Database(msg),
                ));
            }
            warn!("{msg}");
            return Ok(());
        };

        let result = sqlx::query(
            r#"INSERT INTO relationships
                   (source_id, target_id, tenant_id, workspace_id, relation_type,
                    description, weight, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
               ON CONFLICT (tenant_id, workspace_id, source_id, target_id, relation_type)
               DO UPDATE SET
                   description = EXCLUDED.description,
                   weight = EXCLUDED.weight,
                   updated_at = NOW()"#,
        )
        .bind(source_id)
        .bind(target_id)
        .bind(tenant_uuid)
        .bind(workspace_uuid)
        .bind(&rel_type)
        .bind(description)
        .bind(weight)
        .execute(self.pool.as_ref())
        .await;

        self.map_sql_result(&format!("{src}->{tgt}:{rel_type}"), result)
    }

    /// SPEC-091 IP1: resolve endpoints once, then one UNNEST relationship upsert.
    async fn upsert_relationships_batch(
        &self,
        rows: &[RelationshipSinkRow],
    ) -> edgequake_pipeline::Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // All rows in a merge share workspace/tenant; take from first non-empty.
        let workspace_uuid: Option<uuid::Uuid> = rows
            .iter()
            .find_map(|r| r.workspace_id.as_deref().and_then(|w| w.parse().ok()));
        let tenant_uuid: Option<uuid::Uuid> = rows
            .iter()
            .find_map(|r| r.tenant_id.as_deref().and_then(|t| t.parse().ok()));

        let mut bare_names: Vec<String> = Vec::new();
        for row in rows {
            let src = EntityId::bare_name_from_graph_node_id(&row.source_name);
            let tgt = EntityId::bare_name_from_graph_node_id(&row.target_name);
            if !src.is_empty() {
                bare_names.push(src.to_string());
            }
            if !tgt.is_empty() {
                bare_names.push(tgt.to_string());
            }
        }
        bare_names.sort();
        bare_names.dedup();

        let id_rows: Vec<(uuid::Uuid, String)> = sqlx::query_as(
            r#"
            SELECT id, name FROM entities
            WHERE workspace_id IS NOT DISTINCT FROM $2
              AND name = ANY($1)
            "#,
        )
        .bind(&bare_names)
        .bind(workspace_uuid)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            edgequake_pipeline::PipelineError::StorageError(
                edgequake_storage::error::StorageError::Database(e.to_string()),
            )
        })?;

        let mut by_name: HashMap<String, uuid::Uuid> = HashMap::new();
        for (id, name) in id_rows {
            by_name.insert(name, id);
        }

        let mut source_ids: Vec<uuid::Uuid> = Vec::new();
        let mut target_ids: Vec<uuid::Uuid> = Vec::new();
        let mut tenants: Vec<Option<uuid::Uuid>> = Vec::new();
        let mut workspaces: Vec<Option<uuid::Uuid>> = Vec::new();
        let mut rel_types: Vec<String> = Vec::new();
        let mut descs: Vec<String> = Vec::new();
        let mut weights: Vec<f32> = Vec::new();
        let mut missing = 0usize;

        for row in rows {
            let src = EntityId::bare_name_from_graph_node_id(&row.source_name);
            let tgt = EntityId::bare_name_from_graph_node_id(&row.target_name);
            if src.is_empty() || tgt.is_empty() {
                continue;
            }
            let (Some(source_id), Some(target_id)) = (by_name.get(src), by_name.get(tgt)) else {
                missing += 1;
                continue;
            };
            let rel_type = if row.relation_type.trim().is_empty() {
                "RELATED_TO".to_string()
            } else {
                row.relation_type.trim().to_ascii_uppercase()
            };
            source_ids.push(*source_id);
            target_ids.push(*target_id);
            tenants.push(
                row.tenant_id
                    .as_deref()
                    .and_then(|t| t.parse().ok())
                    .or(tenant_uuid),
            );
            workspaces.push(
                row.workspace_id
                    .as_deref()
                    .and_then(|w| w.parse().ok())
                    .or(workspace_uuid),
            );
            rel_types.push(rel_type);
            descs.push(row.description.clone());
            weights.push(row.weight);
        }

        if missing > 0 {
            let msg = format!(
                "relational relationship batch: {missing} row(s) missing entity FK \
                 workspace={workspace_uuid:?}"
            );
            if self.fail_closed {
                return Err(edgequake_pipeline::PipelineError::StorageError(
                    edgequake_storage::error::StorageError::Database(msg),
                ));
            }
            warn!("{msg}");
        }

        if source_ids.is_empty() {
            return Ok(());
        }

        let result = sqlx::query(
            r#"
            INSERT INTO relationships
                (source_id, target_id, tenant_id, workspace_id, relation_type,
                 description, weight, created_at, updated_at)
            SELECT s, t, tn, ws, rt, d, w, NOW(), NOW()
            FROM unnest(
                $1::uuid[],
                $2::uuid[],
                $3::uuid[],
                $4::uuid[],
                $5::text[],
                $6::text[],
                $7::real[]
            ) AS u(s, t, tn, ws, rt, d, w)
            ON CONFLICT (tenant_id, workspace_id, source_id, target_id, relation_type)
            DO UPDATE SET
                description = EXCLUDED.description,
                weight = EXCLUDED.weight,
                updated_at = NOW()
            "#,
        )
        .bind(&source_ids)
        .bind(&target_ids)
        .bind(&tenants)
        .bind(&workspaces)
        .bind(&rel_types)
        .bind(&descs)
        .bind(&weights)
        .execute(self.pool.as_ref())
        .await;

        self.map_sql_result("relationships_batch", result)
    }

    async fn remove_entity_sources(
        &self,
        name: &str,
        workspace_id: Option<&str>,
        _sources_to_remove: &[String],
        remaining_sources: &[String],
    ) -> edgequake_pipeline::Result<()> {
        let bare = EntityId::bare_name_from_graph_node_id(name);
        let workspace_uuid: Option<uuid::Uuid> = workspace_id.and_then(|w| w.parse().ok());

        let result = if remaining_sources.is_empty() {
            sqlx::query(
                "DELETE FROM entities \
                 WHERE (name = $1 OR name = (COALESCE($2::text, '') || '::' || $1)) \
                   AND (workspace_id = $2 OR ($2 IS NULL AND workspace_id IS NULL))",
            )
            .bind(bare)
            .bind(workspace_uuid)
            .execute(self.pool.as_ref())
            .await
        } else {
            sqlx::query(
                "UPDATE entities SET source_chunk_ids = $1, sync_status = 'synced', updated_at = NOW() \
                 WHERE (name = $2 OR name = (COALESCE($3::text, '') || '::' || $2)) \
                   AND (workspace_id = $3 OR ($3 IS NULL AND workspace_id IS NULL))",
            )
            .bind(remaining_sources)
            .bind(bare)
            .bind(workspace_uuid)
            .execute(self.pool.as_ref())
            .await
        };

        self.map_sql_result(bare, result)
    }
}
