//! Failed chunk persistence for retry queue (SPEC-003 / SPEC-046 OPS-P0.4).
//!
//! SOLID: single responsibility — CRUD against `failed_chunks` table.
//! Chunk content lives in KV (`kv_keys::doc_chunk`); this table stores failure metadata only.

use serde::{Deserialize, Serialize};

/// Row shape shared by API DTOs and storage (DRY).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailedChunkRecord {
    pub document_id: String,
    pub workspace_id: String,
    pub tenant_id: Option<String>,
    pub chunk_index: i32,
    pub chunk_id: String,
    pub error_message: String,
    pub was_timeout: bool,
    pub retry_attempts: i32,
    pub processing_time_ms: Option<i64>,
    pub status: String,
}

/// Input for inserting a new pending failure.
#[derive(Debug, Clone)]
pub struct FailedChunkInsert {
    pub document_id: String,
    pub workspace_id: String,
    pub tenant_id: Option<String>,
    pub chunk_index: usize,
    pub chunk_id: String,
    pub error_message: String,
    pub was_timeout: bool,
    pub retry_attempts: u32,
    pub processing_time_ms: u64,
}

impl FailedChunkInsert {
    pub fn to_record(&self) -> FailedChunkRecord {
        FailedChunkRecord {
            document_id: self.document_id.clone(),
            workspace_id: self.workspace_id.clone(),
            tenant_id: self.tenant_id.clone(),
            chunk_index: self.chunk_index as i32,
            chunk_id: self.chunk_id.clone(),
            error_message: self.error_message.clone(),
            was_timeout: self.was_timeout,
            retry_attempts: self.retry_attempts as i32,
            processing_time_ms: Some(self.processing_time_ms as i64),
            status: "pending".into(),
        }
    }
}

/// In-memory store for unit tests / non-postgres deployments (ISP: same ops surface).
#[derive(Debug, Default)]
pub struct InMemoryFailedChunkStore {
    rows: std::sync::Mutex<Vec<FailedChunkRecord>>,
}

impl InMemoryFailedChunkStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_pending(&self, inserts: &[FailedChunkInsert]) {
        let mut rows = self.rows.lock().expect("failed_chunks lock");
        for insert in inserts {
            let mut rec = insert.to_record();
            if let Some(existing) = rows.iter_mut().find(|r| {
                r.document_id == rec.document_id
                    && r.chunk_index == rec.chunk_index
                    && r.status == "pending"
            }) {
                existing.error_message = rec.error_message.clone();
                existing.was_timeout = rec.was_timeout;
                existing.retry_attempts = rec.retry_attempts;
                existing.processing_time_ms = rec.processing_time_ms;
            } else {
                // Unique constraint includes failed_at; for in-memory we collapse pending.
                rec.status = "pending".into();
                rows.push(rec);
            }
        }
    }

    pub fn list_for_document(&self, document_id: &str) -> Vec<FailedChunkRecord> {
        let rows = self.rows.lock().expect("failed_chunks lock");
        rows.iter()
            .filter(|r| r.document_id == document_id)
            .cloned()
            .collect()
    }

    pub fn list_pending(
        &self,
        document_id: &str,
        chunk_indices: Option<&[usize]>,
    ) -> Vec<FailedChunkRecord> {
        self.list_for_document(document_id)
            .into_iter()
            .filter(|r| r.status == "pending" || r.status == "retrying")
            .filter(|r| {
                chunk_indices.is_none_or(|idxs| idxs.iter().any(|i| *i as i32 == r.chunk_index))
            })
            .collect()
    }

    pub fn mark_status(&self, document_id: &str, chunk_index: usize, status: &str) {
        let mut rows = self.rows.lock().expect("failed_chunks lock");
        for row in rows.iter_mut() {
            if row.document_id == document_id && row.chunk_index == chunk_index as i32 {
                row.status = status.to_string();
                if status == "succeeded" || status == "abandoned" {
                    // resolved
                } else if status == "retrying" {
                    row.retry_attempts = row.retry_attempts.saturating_add(1);
                }
            }
        }
    }
}

#[cfg(feature = "postgres")]
pub mod postgres {
    use super::{FailedChunkInsert, FailedChunkRecord};
    use sqlx::PgPool;

    /// Persist failed chunk rows (idempotent on pending collision via ON CONFLICT DO NOTHING
    /// on unique(document_id, chunk_index, failed_at) — we insert a fresh pending row).
    pub async fn insert_failed_chunks(
        pool: &PgPool,
        inserts: &[FailedChunkInsert],
    ) -> Result<usize, sqlx::Error> {
        let mut written = 0usize;
        for insert in inserts {
            let result = sqlx::query(
                r#"
                INSERT INTO failed_chunks (
                    document_id, workspace_id, tenant_id,
                    chunk_index, chunk_id, error_message, was_timeout,
                    retry_attempts, processing_time_ms, status
                ) VALUES (
                    $1, $2::uuid, $3::uuid,
                    $4, $5, $6, $7,
                    $8, $9, 'pending'
                )
                "#,
            )
            .bind(&insert.document_id)
            .bind(&insert.workspace_id)
            .bind(insert.tenant_id.as_deref())
            .bind(insert.chunk_index as i32)
            .bind(&insert.chunk_id)
            .bind(&insert.error_message)
            .bind(insert.was_timeout)
            .bind(insert.retry_attempts as i32)
            .bind(insert.processing_time_ms as i64)
            .execute(pool)
            .await;

            match result {
                Ok(r) => written += r.rows_affected() as usize,
                Err(e) => {
                    // UUID parse failures (non-uuid workspace in tests) — surface.
                    tracing::warn!(error = %e, "failed_chunks insert error");
                    return Err(e);
                }
            }
        }
        Ok(written)
    }

    pub async fn list_failed_chunks(
        pool: &PgPool,
        document_id: &str,
    ) -> Result<Vec<FailedChunkRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<_, FailedChunkRow>(
            r#"
            SELECT document_id, workspace_id::text AS workspace_id,
                   tenant_id::text AS tenant_id,
                   chunk_index, chunk_id, error_message, was_timeout,
                   retry_attempts, processing_time_ms, status
            FROM failed_chunks
            WHERE document_id = $1
            ORDER BY chunk_index ASC, failed_at DESC
            "#,
        )
        .bind(document_id)
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(FailedChunkRow::into_record).collect())
    }

    pub async fn list_pending_for_retry(
        pool: &PgPool,
        document_id: &str,
        chunk_indices: Option<&[usize]>,
    ) -> Result<Vec<FailedChunkRecord>, sqlx::Error> {
        let all = list_failed_chunks(pool, document_id).await?;
        Ok(all
            .into_iter()
            .filter(|r| r.status == "pending" || r.status == "retrying")
            .filter(|r| {
                chunk_indices.is_none_or(|idxs| idxs.iter().any(|i| *i as i32 == r.chunk_index))
            })
            // Deduplicate by chunk_index keeping latest (already ordered DESC failed_at within index)
            .fold(Vec::new(), |mut acc: Vec<FailedChunkRecord>, rec| {
                if !acc.iter().any(|a| a.chunk_index == rec.chunk_index) {
                    acc.push(rec);
                }
                acc
            }))
    }

    pub async fn mark_chunk_status(
        pool: &PgPool,
        document_id: &str,
        chunk_index: usize,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        if status == "retrying" {
            sqlx::query(
                r#"
                UPDATE failed_chunks
                SET status = 'retrying',
                    last_retry_at = NOW(),
                    retry_attempts = retry_attempts + 1
                WHERE document_id = $1 AND chunk_index = $2
                  AND status IN ('pending', 'retrying')
                "#,
            )
            .bind(document_id)
            .bind(chunk_index as i32)
            .execute(pool)
            .await?;
        } else if status == "succeeded" || status == "abandoned" {
            sqlx::query(
                r#"
                UPDATE failed_chunks
                SET status = $3, resolved_at = NOW()
                WHERE document_id = $1 AND chunk_index = $2
                  AND status IN ('pending', 'retrying')
                "#,
            )
            .bind(document_id)
            .bind(chunk_index as i32)
            .bind(status)
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    #[derive(sqlx::FromRow)]
    struct FailedChunkRow {
        document_id: String,
        workspace_id: String,
        tenant_id: Option<String>,
        chunk_index: i32,
        chunk_id: String,
        error_message: String,
        was_timeout: bool,
        retry_attempts: i32,
        processing_time_ms: Option<i64>,
        status: String,
    }

    impl FailedChunkRow {
        fn into_record(self) -> FailedChunkRecord {
            FailedChunkRecord {
                document_id: self.document_id,
                workspace_id: self.workspace_id,
                tenant_id: self.tenant_id,
                chunk_index: self.chunk_index,
                chunk_id: self.chunk_id,
                error_message: self.error_message,
                was_timeout: self.was_timeout,
                retry_attempts: self.retry_attempts,
                processing_time_ms: self.processing_time_ms,
                status: self.status,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_upsert_and_list_pending() {
        let store = InMemoryFailedChunkStore::new();
        store.upsert_pending(&[FailedChunkInsert {
            document_id: "doc-1".into(),
            workspace_id: "00000000-0000-0000-0000-000000000001".into(),
            tenant_id: None,
            chunk_index: 2,
            chunk_id: "doc-1-chunk-2".into(),
            error_message: "timeout".into(),
            was_timeout: true,
            retry_attempts: 1,
            processing_time_ms: 100,
        }]);
        let pending = store.list_pending("doc-1", None);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].chunk_index, 2);
        store.mark_status("doc-1", 2, "succeeded");
        assert!(store.list_pending("doc-1", None).is_empty());
    }

    #[test]
    fn list_pending_filters_indices() {
        let store = InMemoryFailedChunkStore::new();
        for i in 0..3 {
            store.upsert_pending(&[FailedChunkInsert {
                document_id: "d".into(),
                workspace_id: "ws".into(),
                tenant_id: None,
                chunk_index: i,
                chunk_id: format!("d-chunk-{i}"),
                error_message: "e".into(),
                was_timeout: false,
                retry_attempts: 0,
                processing_time_ms: 1,
            }]);
        }
        let only = store.list_pending("d", Some(&[1usize]));
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].chunk_index, 1);
    }
}
