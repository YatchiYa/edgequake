//! SPEC-091 IP2 — transactional outbox writers (LAW-D3 / LAW-IP1).
//!
//! Schema: `outbox_events` (migrations 109 + 133). Writers are fail-open by
//! default so an outbox insert flake never breaks ingest (EC-IP3); callers may
//! observe via tracing + metrics.

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

use crate::error::Result as StorageResult;
#[cfg(feature = "postgres")]
use crate::error::StorageError;

pub const OUTBOX_AGGREGATE_DOCUMENT: &str = "document";

pub const OUTBOX_EVENT_CHUNK_DECLARED: &str = "chunk_declared";
pub const OUTBOX_EVENT_CHUNK_READY: &str = "chunk_ready";
pub const OUTBOX_EVENT_MERGE_DONE: &str = "merge_done";
pub const OUTBOX_EVENT_COMPENSATE: &str = "compensate";

/// Cross-store ingest milestone (DIP: persister depends on this port).
#[async_trait]
pub trait OutboxSink: Send + Sync {
    async fn enqueue(
        &self,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload: Value,
        workspace_id: Option<Uuid>,
    ) -> StorageResult<()>;
}

/// Default: no durable outbox (memory / tests).
pub struct NoopOutboxSink;

#[async_trait]
impl OutboxSink for NoopOutboxSink {
    async fn enqueue(
        &self,
        _aggregate_type: &str,
        _aggregate_id: Uuid,
        _event_type: &str,
        _payload: Value,
        _workspace_id: Option<Uuid>,
    ) -> StorageResult<()> {
        Ok(())
    }
}

/// Postgres writer for `public.outbox_events`.
#[cfg(feature = "postgres")]
pub struct PostgresOutboxSink {
    pool: sqlx::PgPool,
}

#[cfg(feature = "postgres")]
impl PostgresOutboxSink {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl OutboxSink for PostgresOutboxSink {
    async fn enqueue(
        &self,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload: Value,
        workspace_id: Option<Uuid>,
    ) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO outbox_events
                (aggregate_type, aggregate_id, event_type, payload, workspace_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .bind(workspace_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("outbox enqueue failed: {e}")))?;
        Ok(())
    }
}

/// Best-effort enqueue: log + continue on failure (EC-IP3).
pub async fn enqueue_outbox_best_effort(
    sink: Option<&dyn OutboxSink>,
    aggregate_type: &str,
    aggregate_id: Uuid,
    event_type: &str,
    payload: Value,
    workspace_id: Option<Uuid>,
) {
    let Some(sink) = sink else {
        return;
    };
    if let Err(e) = sink
        .enqueue(
            aggregate_type,
            aggregate_id,
            event_type,
            payload,
            workspace_id,
        )
        .await
    {
        tracing::warn!(
            error = %e,
            aggregate_type,
            %aggregate_id,
            event_type,
            "SPEC-091 IP2: outbox enqueue failed (ingest continues)"
        );
    }
}
