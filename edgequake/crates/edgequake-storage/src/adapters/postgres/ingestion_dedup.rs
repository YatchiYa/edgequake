//! SPEC-091 W2 — typed accessor for `public.ingestion_dedup` (migration 107).
//!
//! Replaces the `doc:hash:{ws}:{sha}` and `staging:hash:{ws}:{sha}` KV
//! families with a typed relation: one reservation row per
//! `(workspace_id, content_hash, pipeline_version)`.
//!
//! FK contract: `document_id REFERENCES documents(id)` — writers ensure the
//! minimal `documents` parent first (same helper as the chunk write path).
//! Non-UUID workspace/document ids cannot be represented; callers in legacy
//! string-id modes skip the typed write (KV stays authoritative there).

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageError;

/// Durable reservation discriminator (successfully ingested documents).
pub const DEDUP_VERSION_DURABLE: &str = "v1";
/// In-flight reservation discriminator (admission staging, pre-promote).
pub const DEDUP_VERSION_STAGING: &str = "staging";

fn parse_id(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw).ok()
}

/// Resolve the reserved document id, if any.
pub async fn lookup_document(
    pool: &PgPool,
    workspace_id: &str,
    content_hash: &str,
    pipeline_version: &str,
) -> Result<Option<String>, StorageError> {
    let Some(ws) = parse_id(workspace_id) else {
        return Ok(None);
    };
    let row: Option<(Option<Uuid>,)> = sqlx::query_as(
        "SELECT document_id FROM public.ingestion_dedup \
         WHERE workspace_id = $1 AND content_hash = $2 AND pipeline_version = $3",
    )
    .bind(ws)
    .bind(content_hash)
    .bind(pipeline_version)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ingestion_dedup lookup failed: {e}")))?;
    Ok(row.and_then(|(d,)| d).map(|u| u.to_string()))
}

/// Reserve (upsert) a hash for a document. Idempotent: the latest writer wins,
/// mirroring the KV `upsert` semantics it replaces. Ensures the `documents`
/// parent row exists first (FK).
pub async fn upsert_reservation(
    pool: &PgPool,
    workspace_id: &str,
    content_hash: &str,
    pipeline_version: &str,
    document_id: &str,
    tenant_id: Option<&str>,
) -> Result<(), StorageError> {
    let (Some(ws), Some(doc)) = (parse_id(workspace_id), parse_id(document_id)) else {
        tracing::debug!(
            workspace_id,
            document_id,
            "ingestion_dedup: non-uuid id — typed write skipped (KV authoritative)"
        );
        return Ok(());
    };
    crate::adapters::postgres::chunk_repository::ensure_document_parent(
        pool,
        doc,
        tenant_id.and_then(parse_id),
        Some(ws),
    )
    .await?;
    sqlx::query(
        "INSERT INTO public.ingestion_dedup \
             (workspace_id, content_hash, pipeline_version, document_id) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (workspace_id, content_hash, pipeline_version) \
         DO UPDATE SET document_id = EXCLUDED.document_id",
    )
    .bind(ws)
    .bind(content_hash)
    .bind(pipeline_version)
    .bind(doc)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ingestion_dedup upsert failed: {e}")))?;
    Ok(())
}

/// Delete one versioned reservation (staging release / rollback).
pub async fn delete_reservation(
    pool: &PgPool,
    workspace_id: &str,
    content_hash: &str,
    pipeline_version: &str,
) -> Result<u64, StorageError> {
    let Some(ws) = parse_id(workspace_id) else {
        return Ok(0);
    };
    let result = sqlx::query(
        "DELETE FROM public.ingestion_dedup \
         WHERE workspace_id = $1 AND content_hash = $2 AND pipeline_version = $3",
    )
    .bind(ws)
    .bind(content_hash)
    .bind(pipeline_version)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ingestion_dedup delete failed: {e}")))?;
    Ok(result.rows_affected())
}

/// Promote an in-flight reservation to durable: upsert the durable row and
/// drop the staging row (mirrors `staging_admission::promote_staging_to_final`).
pub async fn promote_staging(
    pool: &PgPool,
    workspace_id: &str,
    content_hash: &str,
    document_id: &str,
    tenant_id: Option<&str>,
) -> Result<(), StorageError> {
    upsert_reservation(
        pool,
        workspace_id,
        content_hash,
        DEDUP_VERSION_DURABLE,
        document_id,
        tenant_id,
    )
    .await?;
    delete_reservation(pool, workspace_id, content_hash, DEDUP_VERSION_STAGING).await?;
    Ok(())
}

/// Delete every reservation for a hash (recycle / delete parity).
pub async fn delete_all_versions(
    pool: &PgPool,
    workspace_id: &str,
    content_hash: &str,
) -> Result<u64, StorageError> {
    let Some(ws) = parse_id(workspace_id) else {
        return Ok(0);
    };
    let result = sqlx::query(
        "DELETE FROM public.ingestion_dedup \
         WHERE workspace_id = $1 AND content_hash = $2",
    )
    .bind(ws)
    .bind(content_hash)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("ingestion_dedup delete-all failed: {e}")))?;
    Ok(result.rows_affected())
}
