//! SPEC-091 IW3 — runtime refusal of stale cutover rollback flags (LD-14).
//!
//! After migration 125 (KV drop) or 126 (chunk-vector fleet drop), operators
//! must not set `kv`/`dual`/`legacy_tables` — those modes target relations
//! that no longer exist. Boot and the migration advisor both enforce this.

use sqlx::PgPool;

use crate::chunk_text_authority::{chunk_text_authority_from_env, ChunkTextAuthority};
use crate::error::StorageError;
use crate::kv_family_cutover::{
    kv_family_mode_from_env, KvFamilyMode, KV_FAMILY_ARTIFACT, KV_FAMILY_CACHE,
    KV_FAMILY_CHECKPOINT, KV_FAMILY_COMPENSATION_QUARANTINE, KV_FAMILY_DOC_HASH,
    KV_FAMILY_INJECTION, KV_FAMILY_METADATA, KV_FAMILY_WSDOC,
};
use crate::vector_backend::{vector_backend_from_env, VectorBackend};

/// Schema-derived cutover posture for flag validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CutoverSchemaPosture {
    /// Migration 125 applied or no `eq_*_kv` relations remain.
    pub kv_store_dropped: bool,
    /// Migration 126 applied (chunk-vector legacy fleet retired).
    pub chunk_vector_legacy_dropped: bool,
    /// Migration 131 applied (full legacy vector fleet dropped).
    /// Do not infer from an empty census alone — fresh DBs have zero tables.
    pub full_vector_legacy_dropped: bool,
}

/// Detect post-drop posture from the live schema (never from env alone).
pub async fn detect_cutover_posture(pool: &PgPool) -> Result<CutoverSchemaPosture, StorageError> {
    let kv_drop_applied = migration_applied(pool, 125).await?;
    let kv_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name LIKE 'eq\\_%\\_kv'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Database(format!("cutover kv table census failed: {e}")))?;

    let chunk_vector_legacy_dropped = migration_applied(pool, 126).await?;
    let full_vector_legacy_dropped = migration_applied(pool, 131).await?;

    Ok(CutoverSchemaPosture {
        kv_store_dropped: kv_drop_applied || kv_tables == 0,
        chunk_vector_legacy_dropped,
        full_vector_legacy_dropped,
    })
}

async fn migration_applied(pool: &PgPool, version: i64) -> Result<bool, StorageError> {
    match sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = $1 AND success)",
    )
    .bind(version)
    .fetch_one(pool)
    .await
    {
        Ok(v) => Ok(v),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => Ok(false),
        Err(e) => Err(StorageError::Database(format!(
            "cutover migration {version} probe failed: {e}"
        ))),
    }
}

/// Fail boot (or any caller) when env flags still target dropped stores.
pub fn validate_cutover_flags(posture: &CutoverSchemaPosture) -> Result<(), String> {
    let mut violations = Vec::new();

    if posture.kv_store_dropped {
        let chunk = chunk_text_authority_from_env();
        if matches!(chunk, ChunkTextAuthority::Kv | ChunkTextAuthority::Dual) {
            violations.push(format!(
                "{}={} but the generic KV store is DROPPED (migration 125) — \
                 set {}=relational and restart",
                crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV,
                match chunk {
                    ChunkTextAuthority::Kv => "kv",
                    ChunkTextAuthority::Dual => "dual",
                    ChunkTextAuthority::Relational => "relational",
                },
                crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV,
            ));
        }

        for (family, env_key) in KV_FAMILIES_WITH_FLAGS {
            if kv_family_mode_from_env(family) == KvFamilyMode::Kv {
                violations.push(format!(
                    "{env_key}=kv but the generic KV store is DROPPED (migration 125) — \
                     set {env_key}=relational and restart"
                ));
            }
        }
    }

    if posture.chunk_vector_legacy_dropped
        && vector_backend_from_env() == VectorBackend::LegacyTables
    {
        violations.push(format!(
            "{}=legacy_tables but chunk-vector legacy tables were DROPPED (migration 126) — \
             set {}=chunk_embeddings and restart",
            crate::vector_backend::VECTOR_BACKEND_ENV,
            crate::vector_backend::VECTOR_BACKEND_ENV,
        ));
    }

    // Migration 131 (or empty census) drops the full workspace fleet —
    // refuse legacy_tables even when only 131 applied without 126 bookkeeping.
    if posture.full_vector_legacy_dropped
        && !posture.chunk_vector_legacy_dropped
        && vector_backend_from_env() == VectorBackend::LegacyTables
    {
        violations.push(format!(
            "{}=legacy_tables but workspace eq_*_vectors are gone (migration 131 \
             or zero legacy vector tables) — set {}=typed_embeddings and restart",
            crate::vector_backend::VECTOR_BACKEND_ENV,
            crate::vector_backend::VECTOR_BACKEND_ENV,
        ));
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join("; "))
    }
}

/// Families that still expose a per-family env flag (STAGING_HASH retired — routes via DOC_HASH).
const KV_FAMILIES_WITH_FLAGS: &[(&str, &str)] = &[
    (KV_FAMILY_METADATA, "EDGEQUAKE_KV_FAMILY_METADATA"),
    (KV_FAMILY_WSDOC, "EDGEQUAKE_KV_FAMILY_WSDOC"),
    (KV_FAMILY_DOC_HASH, "EDGEQUAKE_KV_FAMILY_DOC_HASH"),
    (
        KV_FAMILY_COMPENSATION_QUARANTINE,
        "EDGEQUAKE_KV_FAMILY_COMPENSATION_QUARANTINE",
    ),
    (KV_FAMILY_CHECKPOINT, "EDGEQUAKE_KV_FAMILY_CHECKPOINT"),
    (KV_FAMILY_ARTIFACT, "EDGEQUAKE_KV_FAMILY_ARTIFACT"),
    (KV_FAMILY_INJECTION, "EDGEQUAKE_KV_FAMILY_INJECTION"),
    (KV_FAMILY_CACHE, "EDGEQUAKE_KV_FAMILY_CACHE"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn contract_spec091_cutover_flags_ok_pre_drop() {
        let _g = env_lock();
        std::env::remove_var(crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV);
        std::env::remove_var(crate::vector_backend::VECTOR_BACKEND_ENV);
        let posture = CutoverSchemaPosture {
            kv_store_dropped: false,
            chunk_vector_legacy_dropped: false,
            full_vector_legacy_dropped: false,
        };
        assert!(validate_cutover_flags(&posture).is_ok());
    }

    #[test]
    fn contract_spec091_cutover_flags_refuse_kv_post_drop() {
        let _g = env_lock();
        std::env::set_var(
            crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV,
            "dual",
        );
        let posture = CutoverSchemaPosture {
            kv_store_dropped: true,
            chunk_vector_legacy_dropped: false,
            full_vector_legacy_dropped: false,
        };
        let err = validate_cutover_flags(&posture).unwrap_err();
        assert!(err.contains("DROPPED"));
        assert!(err.contains("relational"));
        std::env::remove_var(crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV);
    }

    #[test]
    fn contract_spec091_cutover_flags_refuse_legacy_vector_post_126() {
        let _g = env_lock();
        std::env::remove_var(crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV);
        std::env::set_var(crate::vector_backend::VECTOR_BACKEND_ENV, "legacy_tables");
        let posture = CutoverSchemaPosture {
            kv_store_dropped: false,
            chunk_vector_legacy_dropped: true,
            full_vector_legacy_dropped: true,
        };
        let err = validate_cutover_flags(&posture).unwrap_err();
        assert!(err.contains("legacy_tables"));
        assert!(err.contains("chunk_embeddings"));
        std::env::remove_var(crate::vector_backend::VECTOR_BACKEND_ENV);
    }

    #[test]
    fn contract_spec091_cutover_flags_refuse_legacy_vector_post_131() {
        let _g = env_lock();
        std::env::remove_var(crate::chunk_text_authority::CHUNK_TEXT_AUTHORITY_ENV);
        std::env::set_var(crate::vector_backend::VECTOR_BACKEND_ENV, "legacy_tables");
        let posture = CutoverSchemaPosture {
            kv_store_dropped: false,
            chunk_vector_legacy_dropped: false,
            full_vector_legacy_dropped: true,
        };
        let err = validate_cutover_flags(&posture).unwrap_err();
        assert!(err.contains("legacy_tables"));
        assert!(err.contains("131") || err.contains("typed_embeddings"));
        std::env::remove_var(crate::vector_backend::VECTOR_BACKEND_ENV);
    }
}
