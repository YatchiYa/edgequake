//! PostgreSQL implementation of LineageSink (SPEC-032 W-08).
//!
//! Writes chunk→entity and chunk→relation links to the relational tables
//! created by migration 066. All writes are best-effort: failures are logged
//! but never propagate to the ingestion path.
//!
//! # Feature gate
//!
//! Only compiled when the `postgres` feature is active.
//!
//! # Tables written
//!
//! - `chunk_entity_links`   (chunk_id, entity_name, workspace_id)
//! - `chunk_relation_links` (chunk_id, source_entity, target_entity, workspace_id)
//! - `entities.description_history` (JSONB append)

use std::sync::Arc;

use async_trait::async_trait;
use edgequake_pipeline::{EntityLineageLink, LineageSink, NoopLineageSink, RelationLineageLink};
#[cfg(feature = "postgres")]
use sqlx::PgPool;
use tracing::warn;

/// Production lineage sink backed by PostgreSQL (migration 066).
#[cfg(feature = "postgres")]
pub struct PostgresLineageSink {
    pool: Arc<PgPool>,
}

#[cfg(feature = "postgres")]
impl PostgresLineageSink {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Returns `PostgresLineageSink` when the `chunk_entity_links` table
    /// exists (migration 066 applied), otherwise falls back to `NoopLineageSink`.
    ///
    /// WHY: Enables zero-downtime rollout — servers without migration 066
    /// continue working; lineage links are persisted only after the migration.
    pub async fn create_if_migration_applied(pool: Arc<PgPool>) -> Arc<dyn LineageSink> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
               SELECT 1 FROM information_schema.tables
               WHERE table_schema = 'public'
                 AND table_name = 'chunk_entity_links'
             )",
        )
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(false);

        if exists {
            Arc::new(Self::new(pool))
        } else {
            tracing::debug!(
                "Migration 066 not yet applied — lineage sink disabled (NoopLineageSink)"
            );
            Arc::new(NoopLineageSink)
        }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl LineageSink for PostgresLineageSink {
    async fn record_entity_link(
        &self,
        chunk_id: &str,
        entity_name: &str,
        workspace_id: &str,
    ) -> edgequake_pipeline::error::Result<()> {
        let result = sqlx::query(
            "INSERT INTO chunk_entity_links (chunk_id, entity_name, workspace_id)
             VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(chunk_id)
        .bind(entity_name)
        .bind(workspace_id)
        .execute(self.pool.as_ref())
        .await;

        if let Err(e) = result {
            warn!(
                chunk_id, entity_name, workspace_id, error = %e,
                "PostgresLineageSink: failed to insert chunk_entity_link"
            );
        }
        Ok(())
    }

    async fn record_relation_link(
        &self,
        chunk_id: &str,
        source_entity: &str,
        target_entity: &str,
        workspace_id: &str,
    ) -> edgequake_pipeline::error::Result<()> {
        let result = sqlx::query(
            "INSERT INTO chunk_relation_links (chunk_id, source_entity, target_entity, workspace_id)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT DO NOTHING",
        )
        .bind(chunk_id)
        .bind(source_entity)
        .bind(target_entity)
        .bind(workspace_id)
        .execute(self.pool.as_ref())
        .await;

        if let Err(e) = result {
            warn!(
                chunk_id, source_entity, target_entity, workspace_id, error = %e,
                "PostgresLineageSink: failed to insert chunk_relation_link"
            );
        }
        Ok(())
    }

    async fn record_relation_links_batch(
        &self,
        links: &[RelationLineageLink],
    ) -> edgequake_pipeline::error::Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        let chunk_ids: Vec<&str> = links.iter().map(|l| l.chunk_id.as_str()).collect();
        let sources: Vec<&str> = links.iter().map(|l| l.source_entity.as_str()).collect();
        let targets: Vec<&str> = links.iter().map(|l| l.target_entity.as_str()).collect();
        let workspaces: Vec<&str> = links.iter().map(|l| l.workspace_id.as_str()).collect();

        let result = sqlx::query(
            "INSERT INTO chunk_relation_links (chunk_id, source_entity, target_entity, workspace_id)
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])
             ON CONFLICT DO NOTHING",
        )
        .bind(&chunk_ids)
        .bind(&sources)
        .bind(&targets)
        .bind(&workspaces)
        .execute(self.pool.as_ref())
        .await;

        if let Err(e) = result {
            warn!(
                count = links.len(),
                error = %e,
                "PostgresLineageSink: failed to batch insert chunk_relation_links"
            );
        }
        Ok(())
    }

    async fn record_entity_links_batch(
        &self,
        links: &[EntityLineageLink],
    ) -> edgequake_pipeline::error::Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        let chunk_ids: Vec<&str> = links.iter().map(|l| l.chunk_id.as_str()).collect();
        let entities: Vec<&str> = links.iter().map(|l| l.entity_name.as_str()).collect();
        let workspaces: Vec<&str> = links.iter().map(|l| l.workspace_id.as_str()).collect();

        let result = sqlx::query(
            "INSERT INTO chunk_entity_links (chunk_id, entity_name, workspace_id)
             SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
             ON CONFLICT DO NOTHING",
        )
        .bind(&chunk_ids)
        .bind(&entities)
        .bind(&workspaces)
        .execute(self.pool.as_ref())
        .await;

        if let Err(e) = result {
            warn!(
                count = links.len(),
                error = %e,
                "PostgresLineageSink: failed to batch insert chunk_entity_links"
            );
        }
        Ok(())
    }

    async fn append_description_history(
        &self,
        entity_name: &str,
        workspace_id: &str,
        description: &str,
        chunk_id: &str,
    ) -> edgequake_pipeline::error::Result<()> {
        let history_entry = serde_json::json!({
            "ts": chrono::Utc::now().to_rfc3339(),
            "description": description,
            "chunk_id": chunk_id,
        });

        let result = sqlx::query(
            "UPDATE entities
             SET description_history = description_history || $1::jsonb
             WHERE name = $2 AND workspace_id = $3",
        )
        .bind(serde_json::json!([history_entry]))
        .bind(entity_name)
        .bind(workspace_id)
        .execute(self.pool.as_ref())
        .await;

        if let Err(e) = result {
            warn!(
                entity_name, workspace_id, chunk_id, error = %e,
                "PostgresLineageSink: failed to append description_history"
            );
        }
        Ok(())
    }
}
