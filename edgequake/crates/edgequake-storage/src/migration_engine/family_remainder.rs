//! SPEC-139 LAW-139-6: idempotent remainder after one-shot sqlx 117 / 119 / 122.
//!
//! Migration 119 copies artifacts only when `documents` already exists; 122
//! creates those shells later. 117 similarly skips parent-less hash keys.
//! These descriptors replay the same INSERT … ON CONFLICT without editing
//! applied sqlx bodies (LAW-111-10).
//!
//! Verify is **copy-complete** (always coverage-pass): leftover orphans are the
//! advisor / DROP 125 gate, not a terminal engine fail-loop (LAW-137-3).

use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Digest, Sha384};
use sqlx::{PgPool, Postgres, Transaction};

use super::advisor::residue::kv_durable_residue_all;
use super::runner::{BackfillJob, BatchOutcome, VerifyReport};
use crate::error::StorageError;

const ARTIFACT_DEF: &str = concat!(
    "w5-artifact-remainder/v1:",
    "source=eq_*_kv:lineage+multimodal;join=documents;insert=on_conflict(document_id,kind)"
);

const SHELL_DEF: &str = concat!(
    "wc-shell-remainder/v1:",
    "source=eq_*_kv:metadata+content+staging;insert=documents_on_conflict_122_shape"
);

const DEDUP_DEF: &str = concat!(
    "w2-dedup-remainder/v1:",
    "source=eq_*_kv:doc:hash+staging:hash;insert=ingestion_dedup_on_conflict_do_nothing"
);

const UUID_RE: &str =
    r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
const KIND_RE: &str = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}-(lineage|multimodal-manifest|multimodal-chunks)$";

fn sha384_hex(s: &str) -> String {
    Sha384::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn safe_ident(table: &str) -> bool {
    table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Replay migration 119 against remaining KV tables (parents may exist now).
pub struct ArtifactRemainderJob;

impl ArtifactRemainderJob {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArtifactRemainderJob {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl BackfillJob for ArtifactRemainderJob {
    fn step_id(&self) -> &'static str {
        "w5-artifact-remainder"
    }

    fn step_sha384(&self) -> String {
        sha384_hex(ARTIFACT_DEF)
    }

    fn schema_generation(&self) -> i32 {
        1
    }

    fn initial_cursor(&self) -> Value {
        json!({ "done": false })
    }

    async fn estimate_total(&self, pool: &PgPool) -> Result<i64, StorageError> {
        let r = kv_durable_residue_all(pool).await?;
        Ok(r.lineage + r.multimodal)
    }

    async fn run_batch(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cursor: &Value,
        _limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        if cursor.get("done").and_then(Value::as_bool) == Some(true) {
            return Ok(BatchOutcome {
                scanned: 0,
                written: 0,
                failed: 0,
                next_cursor: None,
            });
        }
        let tables = list_kv_tables_tx(tx).await?;
        let mut written = 0i64;
        for table in &tables {
            if !safe_ident(table) {
                continue;
            }
            let res = sqlx::query(&format!(
                "INSERT INTO public.document_artifacts (document_id, kind, payload) \
                 SELECT left(kv.key, 36)::uuid, substring(kv.key FROM 38), kv.value \
                 FROM public.{table} kv \
                 WHERE kv.key ~ $1 \
                   AND left(kv.key, 36) ~ $2 \
                   AND EXISTS (SELECT 1 FROM public.documents d \
                               WHERE d.id = left(kv.key, 36)::uuid) \
                 ON CONFLICT (document_id, kind) DO UPDATE SET \
                   payload = EXCLUDED.payload, updated_at = now() \
                   WHERE document_artifacts.updated_at <= now()"
            ))
            .bind(KIND_RE)
            .bind(UUID_RE)
            .execute(&mut **tx)
            .await
            .map_err(|e| StorageError::Database(format!("w5 artifact remainder({table}): {e}")))?;
            written += res.rows_affected() as i64;
        }
        Ok(BatchOutcome {
            scanned: written,
            written,
            failed: 0,
            next_cursor: Some(json!({ "done": true })),
        })
    }

    async fn verify(&self, _pool: &PgPool) -> Result<VerifyReport, StorageError> {
        Ok(remainder_copy_complete("w5-artifact-remainder"))
    }
}

/// Replay migration 117 hash → `ingestion_dedup` (parents may exist now).
pub struct DedupRemainderJob;

impl DedupRemainderJob {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DedupRemainderJob {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl BackfillJob for DedupRemainderJob {
    fn step_id(&self) -> &'static str {
        "w2-dedup-remainder"
    }

    fn step_sha384(&self) -> String {
        sha384_hex(DEDUP_DEF)
    }

    fn schema_generation(&self) -> i32 {
        1
    }

    fn initial_cursor(&self) -> Value {
        json!({ "done": false })
    }

    async fn estimate_total(&self, pool: &PgPool) -> Result<i64, StorageError> {
        let r = kv_durable_residue_all(pool).await?;
        Ok(r.doc_hash + r.staging_hash)
    }

    async fn run_batch(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cursor: &Value,
        _limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        if cursor.get("done").and_then(Value::as_bool) == Some(true) {
            return Ok(BatchOutcome {
                scanned: 0,
                written: 0,
                failed: 0,
                next_cursor: None,
            });
        }
        let tables = list_kv_tables_tx(tx).await?;
        let mut written = 0i64;
        for table in &tables {
            if !safe_ident(table) {
                continue;
            }
            written += apply_dedup_table(tx, table).await?;
        }
        Ok(BatchOutcome {
            scanned: written,
            written,
            failed: 0,
            next_cursor: Some(json!({ "done": true })),
        })
    }

    async fn verify(&self, _pool: &PgPool) -> Result<VerifyReport, StorageError> {
        Ok(remainder_copy_complete("w2-dedup-remainder"))
    }
}

/// Replay migration 122 document shells (metadata/content) after one-shot skip.
pub struct ShellRemainderJob;

impl ShellRemainderJob {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShellRemainderJob {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl BackfillJob for ShellRemainderJob {
    fn step_id(&self) -> &'static str {
        "wc-shell-remainder"
    }

    fn step_sha384(&self) -> String {
        sha384_hex(SHELL_DEF)
    }

    fn schema_generation(&self) -> i32 {
        1
    }

    fn initial_cursor(&self) -> Value {
        json!({ "done": false })
    }

    async fn estimate_total(&self, pool: &PgPool) -> Result<i64, StorageError> {
        let r = kv_durable_residue_all(pool).await?;
        Ok(r.doc_shells)
    }

    async fn run_batch(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cursor: &Value,
        _limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        if cursor.get("done").and_then(Value::as_bool) == Some(true) {
            return Ok(BatchOutcome {
                scanned: 0,
                written: 0,
                failed: 0,
                next_cursor: None,
            });
        }
        let tables = list_kv_tables_tx(tx).await?;
        let mut written = 0i64;
        for table in &tables {
            if !safe_ident(table) {
                continue;
            }
            written += apply_shell_table(tx, table).await?;
        }
        Ok(BatchOutcome {
            scanned: written,
            written,
            failed: 0,
            next_cursor: Some(json!({ "done": true })),
        })
    }

    async fn verify(&self, _pool: &PgPool) -> Result<VerifyReport, StorageError> {
        Ok(remainder_copy_complete("wc-shell-remainder"))
    }
}

/// Remainder jobs copy what they can. Leftover orphans fail **guard**, not the job.
fn remainder_copy_complete(metric: &str) -> VerifyReport {
    VerifyReport {
        metric: metric.into(),
        expected: 0,
        actual: 0,
        sampled: 0,
        mismatches: 0,
    }
}

async fn list_kv_tables_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<String>, StorageError> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_kv' ESCAPE '\'
          AND c.relname NOT LIKE '%\_kv\_stats' ESCAPE '\'
        ORDER BY c.relname
        "#,
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("remainder list_kv_tables: {e}")))
}

async fn apply_dedup_table(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
) -> Result<i64, StorageError> {
    // Documents parents for durable hashes (117).
    sqlx::query(&format!(
        "INSERT INTO public.documents (id, workspace_id, content, status) \
         SELECT DISTINCT (kv.value #>> '{{}}')::uuid, \
                         split_part(kv.key, ':', 3)::uuid, '', 'indexed' \
         FROM public.{table} kv \
         WHERE kv.key LIKE 'doc:hash:%' \
           AND split_part(kv.key, ':', 3) ~ $1 \
           AND (kv.value #>> '{{}}') ~ $1 \
           AND EXISTS (SELECT 1 FROM public.workspaces w \
                       WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("w2 dedup docs({table}): {e}")))?;

    let v1 = sqlx::query(&format!(
        "INSERT INTO public.ingestion_dedup \
            (workspace_id, content_hash, pipeline_version, document_id) \
         SELECT split_part(kv.key, ':', 3)::uuid, \
                split_part(kv.key, ':', 4), 'v1', \
                (kv.value #>> '{{}}')::uuid \
         FROM public.{table} kv \
         WHERE kv.key LIKE 'doc:hash:%' \
           AND split_part(kv.key, ':', 3) ~ $1 \
           AND (kv.value #>> '{{}}') ~ $1 \
           AND EXISTS (SELECT 1 FROM public.workspaces w \
                       WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid) \
           AND EXISTS (SELECT 1 FROM public.documents d \
                       WHERE d.id = (kv.value #>> '{{}}')::uuid) \
         ON CONFLICT (workspace_id, content_hash, pipeline_version) DO NOTHING"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("w2 dedup v1({table}): {e}")))?;

    sqlx::query(&format!(
        "INSERT INTO public.documents (id, workspace_id, content, status) \
         SELECT DISTINCT (kv.value #>> '{{}}')::uuid, \
                         split_part(kv.key, ':', 3)::uuid, '', 'processing' \
         FROM public.{table} kv \
         WHERE kv.key LIKE 'staging:hash:%' \
           AND split_part(kv.key, ':', 3) ~ $1 \
           AND (kv.value #>> '{{}}') ~ $1 \
           AND EXISTS (SELECT 1 FROM public.workspaces w \
                       WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("w2 dedup staging docs({table}): {e}")))?;

    let st = sqlx::query(&format!(
        "INSERT INTO public.ingestion_dedup \
            (workspace_id, content_hash, pipeline_version, document_id) \
         SELECT split_part(kv.key, ':', 3)::uuid, \
                split_part(kv.key, ':', 4), 'staging', \
                (kv.value #>> '{{}}')::uuid \
         FROM public.{table} kv \
         WHERE kv.key LIKE 'staging:hash:%' \
           AND split_part(kv.key, ':', 3) ~ $1 \
           AND (kv.value #>> '{{}}') ~ $1 \
           AND EXISTS (SELECT 1 FROM public.workspaces w \
                       WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid) \
           AND EXISTS (SELECT 1 FROM public.documents d \
                       WHERE d.id = (kv.value #>> '{{}}')::uuid) \
         ON CONFLICT (workspace_id, content_hash, pipeline_version) DO NOTHING"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("w2 dedup staging({table}): {e}")))?;

    Ok(v1.rows_affected() as i64 + st.rows_affected() as i64)
}

async fn apply_shell_table(
    tx: &mut Transaction<'_, Postgres>,
    table: &str,
) -> Result<i64, StorageError> {
    let staging = sqlx::query(&format!(
        "INSERT INTO public.documents (id, content, status, metadata) \
         SELECT substring(kv.key FROM 9 FOR 36)::uuid, '', 'processing', \
                jsonb_set(kv.value, '{{_shell}}', '\"staging\"') \
         FROM public.{table} kv \
         WHERE kv.key LIKE 'staging:%' \
           AND kv.key LIKE '%-metadata' \
           AND substring(kv.key FROM 9 FOR 36) ~ $1 \
         ON CONFLICT (id) DO UPDATE SET \
            metadata = EXCLUDED.metadata, \
            status = 'processing', \
            updated_at = now() \
            WHERE public.documents.metadata IS NULL \
               OR public.documents.metadata = '{{}}'::jsonb"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("wc shell staging({table}): {e}")))?;

    sqlx::query(&format!(
        "UPDATE public.documents d \
         SET content = kv.value->>'content', updated_at = now() \
         FROM public.{table} kv \
         WHERE kv.key = 'staging:' || d.id::text || '-content' \
           AND d.content = '' \
           AND kv.value->>'content' IS NOT NULL"
    ))
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("wc shell staging content({table}): {e}")))?;

    let final_meta = sqlx::query(&format!(
        "INSERT INTO public.documents (id, title, content, status, metadata) \
         SELECT left(kv.key, 36)::uuid, \
                COALESCE(kv.value->>'title', ''), '', \
                'indexed', kv.value \
         FROM public.{table} kv \
         WHERE kv.key LIKE '%-metadata' \
           AND kv.key NOT LIKE 'staging:%' \
           AND left(kv.key, 36) ~ $1 \
         ON CONFLICT (id) DO UPDATE SET \
            metadata = EXCLUDED.metadata, \
            title = CASE WHEN EXCLUDED.title = '' THEN public.documents.title \
                         ELSE EXCLUDED.title END, \
            status = CASE WHEN public.documents.metadata->>'_shell' = 'staging' \
                          THEN 'indexed' ELSE public.documents.status END, \
            updated_at = now()"
    ))
    .bind(UUID_RE)
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("wc shell metadata({table}): {e}")))?;

    sqlx::query(&format!(
        "UPDATE public.documents d \
         SET content = kv.value->>'content', updated_at = now() \
         FROM public.{table} kv \
         WHERE kv.key = d.id::text || '-content' \
           AND kv.value->>'content' IS NOT NULL"
    ))
    .execute(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("wc shell content({table}): {e}")))?;

    Ok(staging.rows_affected() as i64 + final_meta.rows_affected() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec139_remainder_step_ids_stable() {
        let a = ArtifactRemainderJob::new();
        let d = DedupRemainderJob::new();
        let s = ShellRemainderJob::new();
        assert_eq!(a.step_id(), "w5-artifact-remainder");
        assert_eq!(d.step_id(), "w2-dedup-remainder");
        assert_eq!(s.step_id(), "wc-shell-remainder");
        assert_eq!(a.step_sha384().len(), 96);
        assert_eq!(d.step_sha384().len(), 96);
        assert_eq!(s.step_sha384().len(), 96);
        assert!(remainder_copy_complete("x").passes());
    }
}
