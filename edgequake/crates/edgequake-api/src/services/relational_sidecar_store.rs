//! SPEC-091 Wave B4/B5 — typed sidecar store (migration 116 tables).
//!
//! Single owner for `public.pipeline_checkpoints` and `public.document_artifacts`
//! I/O, replacing the per-document KV blobs (`{doc}-pipeline-checkpoint`,
//! `{doc}-extraction-snapshot`, `{doc}-lineage`, `{doc}-multimodal-manifest`,
//! `{doc}-multimodal-chunks`, MM cache).
//!
//! Cutover pattern (identical to B1/B3):
//! - **Writes**: callers keep KV authoritative; every save also lands here
//!   (warn-only) so the typed table converges while KV can still be rolled
//!   back to with a flag flip.
//! - **Reads**: flag-gated (`EDGEQUAKE_KV_FAMILY_CHECKPOINT` /
//!   `EDGEQUAKE_KV_FAMILY_ARTIFACT` = relational) typed-first; any gap
//!   (flag off, no pool, non-UUID doc id, typed miss/error) falls back to KV.
//!
//! One process-global pool registry serves every sidecar reader/writer
//! (DRY — mirrors the B2 quarantine sink and B3 membership wiring).

use serde_json::Value;

use edgequake_storage::kv_family_cutover::{
    kv_family_mode_from_env, KvFamilyMode, KV_FAMILY_ARTIFACT, KV_FAMILY_CHECKPOINT,
};

pub const CHECKPOINT_KIND_CRASH: &str = "checkpoint";
pub const CHECKPOINT_KIND_SNAPSHOT: &str = "snapshot";
pub const ARTIFACT_KIND_LINEAGE: &str = "lineage";
pub const ARTIFACT_KIND_MM_MANIFEST: &str = "multimodal-manifest";
pub const ARTIFACT_KIND_MM_CHUNKS: &str = "multimodal-chunks";

#[cfg(feature = "postgres")]
static SIDECAR_POOL: std::sync::RwLock<Option<&'static sqlx::PgPool>> =
    std::sync::RwLock::new(None);

/// Register the Postgres pool for all sidecar I/O.
///
/// Re-registration **replaces** the stored pool (last call wins). Production
/// registers once at startup; tests register a fresh pool per runtime. A
/// `OnceLock` (first-call-wins) is wrong for tests because each `#[tokio::test]`
/// runs its own runtime — a pool created on a prior test's runtime hangs
/// (`PoolTimedOut`) when reused from a later test. The pool is leaked to obtain
/// `&'static`, which is fine (one leak in prod, a handful per test process).
#[cfg(feature = "postgres")]
pub fn register_sidecar_pool(pool: sqlx::PgPool) {
    let leaked: &'static sqlx::PgPool = Box::leak(Box::new(pool));
    *SIDECAR_POOL.write().expect("sidecar pool lock") = Some(leaked);
}

#[cfg(feature = "postgres")]
pub fn sidecar_pool() -> Option<&'static sqlx::PgPool> {
    *SIDECAR_POOL.read().expect("sidecar pool lock")
}

pub fn checkpoints_prefer_relational() -> bool {
    kv_family_mode_from_env(KV_FAMILY_CHECKPOINT) == KvFamilyMode::Relational
}

pub fn artifacts_prefer_relational() -> bool {
    kv_family_mode_from_env(KV_FAMILY_ARTIFACT) == KvFamilyMode::Relational
}

/// SPEC-091 Wave D: typed writes are warn-only during dual-write; once the
/// family flag flips relational they are the ONLY write (the adapter write-stop
/// drops the KV upsert) — escalate failures to error! so an authoritative
/// loss is loud. Rollback = flip the family flag back to `kv`.
#[cfg(feature = "postgres")]
fn log_write_failure(relational: bool, op: &str, document_id: &str, kind: &str, e: &str) {
    if relational {
        tracing::error!(
            document_id,
            kind,
            op,
            error = %e,
            "SPEC-091: authoritative typed sidecar write FAILED — data is not persisted; \
             investigate the typed store or roll the family flag back to kv"
        );
    } else {
        tracing::warn!(document_id, kind, op, error = %e, "typed sidecar dual-write failed (KV remains)");
    }
}

/// Parse a UUID document id — typed sidecars are keyed by `documents.id`.
#[cfg(feature = "postgres")]
fn doc_uuid(document_id: &str) -> Option<uuid::Uuid> {
    uuid::Uuid::parse_str(document_id).ok()
}

/// Ensure the FK parent exists (checkpoints can be written for documents
/// whose admission row raced or predates Wave B3).
#[cfg(feature = "postgres")]
async fn ensure_parent(pool: &sqlx::PgPool, doc: uuid::Uuid) -> Result<(), String> {
    // Parent-only ensure for checkpoint FK; empty title → schema placeholder
    // repaired later by admission / staging shell when a real title arrives.
    edgequake_storage::ensure_admission_document_row(pool, doc, None, None, "")
        .await
        .map_err(|e| e.to_string())
}

// ── pipeline_checkpoints ────────────────────────────────────────────────────

/// True when a typed checkpoint write can land (pool installed + UUID doc id).
pub fn typed_checkpoint_writable(document_id: &str) -> bool {
    #[cfg(feature = "postgres")]
    {
        sidecar_pool().is_some() && doc_uuid(document_id).is_some()
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = document_id;
        false
    }
}

/// Typed upsert (warn-only). No-op without a pool or for non-UUID ids.
/// Returns whether the row was written successfully.
pub async fn typed_checkpoint_put(document_id: &str, kind: &str, payload: &Value) -> bool {
    #[cfg(feature = "postgres")]
    {
        let (Some(pool), Some(doc)) = (sidecar_pool(), doc_uuid(document_id)) else {
            return false;
        };
        let relational = checkpoints_prefer_relational();
        if let Err(e) = ensure_parent(pool, doc).await {
            log_write_failure(
                relational,
                "checkpoint_parent_ensure",
                document_id,
                kind,
                &e,
            );
            return false;
        }
        let result = sqlx::query(
            r#"
            INSERT INTO public.pipeline_checkpoints (document_id, kind, payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (document_id, kind) DO UPDATE SET
                payload = EXCLUDED.payload, updated_at = now()
            "#,
        )
        .bind(doc)
        .bind(kind)
        .bind(payload)
        .execute(pool)
        .await;
        match result {
            Ok(_) => true,
            Err(e) => {
                log_write_failure(
                    relational,
                    "checkpoint_upsert",
                    document_id,
                    kind,
                    &e.to_string(),
                );
                false
            }
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (document_id, kind, payload);
        false
    }
}

/// Typed read. `None` on miss, error, no pool, or non-UUID id (→ KV fallback).
pub async fn typed_checkpoint_get(document_id: &str, kind: &str) -> Option<Value> {
    #[cfg(feature = "postgres")]
    {
        let (pool, doc) = (sidecar_pool()?, doc_uuid(document_id)?);
        match sqlx::query_scalar::<_, Value>(
            "SELECT payload FROM public.pipeline_checkpoints WHERE document_id = $1 AND kind = $2",
        )
        .bind(doc)
        .bind(kind)
        .fetch_optional(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(document_id, kind, error = %e, "typed checkpoint read failed");
                None
            }
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (document_id, kind);
        None
    }
}

/// Typed delete (warn-only), paired with the caller's KV delete.
pub async fn typed_checkpoint_delete(document_id: &str, kind: &str) {
    #[cfg(feature = "postgres")]
    {
        let (Some(pool), Some(doc)) = (sidecar_pool(), doc_uuid(document_id)) else {
            return;
        };
        if let Err(e) = sqlx::query(
            "DELETE FROM public.pipeline_checkpoints WHERE document_id = $1 AND kind = $2",
        )
        .bind(doc)
        .bind(kind)
        .execute(pool)
        .await
        {
            log_write_failure(
                checkpoints_prefer_relational(),
                "checkpoint_delete",
                document_id,
                kind,
                &e.to_string(),
            );
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = (document_id, kind);
}

/// Startup sweep mirroring `cleanup_stale_checkpoints` for typed rows.
pub async fn cleanup_stale_typed_checkpoints(max_age_secs: u64) {
    #[cfg(feature = "postgres")]
    {
        let Some(pool) = sidecar_pool() else { return };
        match sqlx::query(
            "DELETE FROM public.pipeline_checkpoints \
             WHERE kind = 'checkpoint' AND updated_at < now() - make_interval(secs => $1)",
        )
        .bind(max_age_secs as i64)
        .execute(pool)
        .await
        {
            Ok(r) if r.rows_affected() > 0 => tracing::info!(
                cleaned = r.rows_affected(),
                "cleaned up stale typed pipeline checkpoints on startup"
            ),
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "typed checkpoint sweep failed"),
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = max_age_secs;
}

// ── document_artifacts ──────────────────────────────────────────────────────

/// Typed upsert (warn-only). No-op without a pool or for non-UUID ids.
pub async fn typed_artifact_put(document_id: &str, kind: &str, payload: &Value) {
    #[cfg(feature = "postgres")]
    {
        let (Some(pool), Some(doc)) = (sidecar_pool(), doc_uuid(document_id)) else {
            return;
        };
        let relational = artifacts_prefer_relational();
        if let Err(e) = ensure_parent(pool, doc).await {
            log_write_failure(relational, "artifact_parent_ensure", document_id, kind, &e);
            return;
        }
        let result = sqlx::query(
            r#"
            INSERT INTO public.document_artifacts (document_id, kind, payload)
            VALUES ($1, $2, $3)
            ON CONFLICT (document_id, kind) DO UPDATE SET
                payload = EXCLUDED.payload, updated_at = now()
            "#,
        )
        .bind(doc)
        .bind(kind)
        .bind(payload)
        .execute(pool)
        .await;
        if let Err(e) = result {
            log_write_failure(
                relational,
                "artifact_upsert",
                document_id,
                kind,
                &e.to_string(),
            );
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = (document_id, kind, payload);
}

/// Typed read. `None` on miss, error, no pool, or non-UUID id (→ KV fallback).
pub async fn typed_artifact_get(document_id: &str, kind: &str) -> Option<Value> {
    #[cfg(feature = "postgres")]
    {
        let (pool, doc) = (sidecar_pool()?, doc_uuid(document_id)?);
        match sqlx::query_scalar::<_, Value>(
            "SELECT payload FROM public.document_artifacts WHERE document_id = $1 AND kind = $2",
        )
        .bind(doc)
        .bind(kind)
        .fetch_optional(pool)
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(document_id, kind, error = %e, "typed artifact read failed");
                None
            }
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        let _ = (document_id, kind);
        None
    }
}

/// Delete every typed artifact for a document (deletion parity with the
/// legacy per-family KV key deletes).
pub async fn typed_artifact_delete_all(document_id: &str) {
    #[cfg(feature = "postgres")]
    {
        let (Some(pool), Some(doc)) = (sidecar_pool(), doc_uuid(document_id)) else {
            return;
        };
        if let Err(e) = sqlx::query("DELETE FROM public.document_artifacts WHERE document_id = $1")
            .bind(doc)
            .execute(pool)
            .await
        {
            log_write_failure(
                artifacts_prefer_relational(),
                "artifact_delete_all",
                document_id,
                "*",
                &e.to_string(),
            );
        }
    }
    #[cfg(not(feature = "postgres"))]
    let _ = document_id;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC-091 Wave D: flags default to RELATIONAL; typed accessors are
    /// still inert without a registered pool (memory/test mode unaffected),
    /// and the `kv` rollback env keeps working.
    #[tokio::test]
    async fn typed_accessors_inert_without_pool() {
        std::env::remove_var("EDGEQUAKE_KV_FAMILY_CHECKPOINT");
        std::env::remove_var("EDGEQUAKE_KV_FAMILY_ARTIFACT");
        assert!(checkpoints_prefer_relational());
        assert!(artifacts_prefer_relational());
        typed_checkpoint_put("doc", CHECKPOINT_KIND_CRASH, &serde_json::json!({"a": 1})).await;
        typed_artifact_put("doc", ARTIFACT_KIND_LINEAGE, &serde_json::json!({"b": 2})).await;
        typed_checkpoint_delete("doc", CHECKPOINT_KIND_CRASH).await;
        typed_artifact_delete_all("doc").await;

        std::env::set_var("EDGEQUAKE_KV_FAMILY_CHECKPOINT", "kv");
        std::env::set_var("EDGEQUAKE_KV_FAMILY_ARTIFACT", "kv");
        assert!(!checkpoints_prefer_relational());
        assert!(!artifacts_prefer_relational());
        std::env::remove_var("EDGEQUAKE_KV_FAMILY_CHECKPOINT");
        std::env::remove_var("EDGEQUAKE_KV_FAMILY_ARTIFACT");
    }
}
