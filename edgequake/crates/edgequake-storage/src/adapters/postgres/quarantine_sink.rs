//! SPEC-091 Wave B2: typed [`QuarantineSink`] over `public.compensation_quarantine`.
//!
//! Retires the legacy `compensation_quarantine:{doc}:{uuid}` KV DLQ record for
//! Postgres deployments: failed compensation cleanups land in the typed table
//! (migration 107) where the drain worker (`compensation_drain.rs`) claims them
//! with `FOR UPDATE SKIP LOCKED`. Payload shape is identical to the legacy KV
//! record so operators see the same fields either way (SSOT).

use sqlx::PgPool;
use uuid::Uuid;

use crate::compensation::QuarantineSink;
use crate::error::StorageError;

pub struct PgQuarantineSink {
    pool: PgPool,
}

impl PgQuarantineSink {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl QuarantineSink for PgQuarantineSink {
    async fn insert(
        &self,
        document_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), StorageError> {
        let doc_uuid = Uuid::parse_str(document_id).map_err(|_| {
            StorageError::InvalidQuery(format!(
                "compensation_quarantine: non-UUID document_id '{document_id}'"
            ))
        })?;

        // FK: compensation_quarantine.document_id REFERENCES documents(id).
        // Compensation can fire before the document row is visible to this
        // connection, so ensure a minimal parent first (mirrors dedup pattern).
        crate::adapters::postgres::chunk_repository::ensure_document_parent(
            &self.pool, doc_uuid, None, None,
        )
        .await?;

        // GAP-091-13 (SPEC-091 IW0): attribute the entry to the document's
        // workspace so drains/operators can filter per workspace instead of
        // treating the DLQ as an unscoped global pool. NULL when the parent
        // row carries no workspace (tombstone shell after delete).
        let workspace_id: Option<Uuid> =
            sqlx::query_scalar("SELECT workspace_id FROM public.documents WHERE id = $1")
                .bind(doc_uuid)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::Database(format!("quarantine workspace lookup: {e}")))?
                .flatten();

        sqlx::query(
            r#"
            INSERT INTO public.compensation_quarantine (entry_id, document_id, workspace_id, payload)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(doc_uuid)
        .bind(workspace_id)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("compensation_quarantine insert: {e}")))?;
        Ok(())
    }
}
