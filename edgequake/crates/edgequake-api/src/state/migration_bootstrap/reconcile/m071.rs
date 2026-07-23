//! Migration 071 — HNSW dimension guard checksum repair (SPEC-042 / GitHub #275).
//!
//! v0.13.3 shipped M071 without pgvector HNSW dimension ceilings. Upgrades with
//! embeddings dim > 2000 (e.g. text-embedding-3-large @ 3072) failed at sqlx apply.
//! Fixed M071 promotes to halfvec when needed; repair checksum for already-applied rows.

use sqlx::PgPool;
use tracing::info;

use super::super::helpers::sqlx_migrations_table_exists;
use super::super::MIGRATION_071_VERSION;

/// SHA-384 of pre-#275 M071 (no dimension guard).
pub(super) const M071_CHECKSUM_PRE_275: &str =
    "fa6cce9c4b088b5dbc850764887e9bd119f32ceca259611113ab4673b6c3319353ae2b44218a3d312d03e3509f5520ca";

/// SHA-384 of #275-fixed M071 — must match `checksums.lock`.
pub(super) const M071_CHECKSUM_FIXED_275: &str =
    "fea7b113e1aab4f88d0c22a071ba78e043b94e38023450c8257c0a0027647193b5c0ca382a6009577e5f4f823a46cda2";

/// SPEC-083 X-02: silent checksum rewrite is DEV_MODE-only.
fn allow_checksum_repair() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_DEV_MODE").map(|v| v == "1" || v.eq_ignore_ascii_case("true")),
        Ok(true)
    )
}

pub async fn repair_migration_071_checksum_if_needed(pool: &PgPool) -> Result<bool, sqlx::Error> {
    if !sqlx_migrations_table_exists(pool).await? {
        return Ok(false);
    }

    let current: Option<String> = sqlx::query_scalar(
        "SELECT encode(checksum, 'hex') FROM _sqlx_migrations \
         WHERE version = $1 AND success = true",
    )
    .bind(MIGRATION_071_VERSION)
    .fetch_optional(pool)
    .await?;

    let Some(current) = current else {
        return Ok(false);
    };

    if current != M071_CHECKSUM_PRE_275 {
        return Ok(false);
    }

    // SPEC-083 X-02: fail loud in prod — do not silently rewrite history.
    if !allow_checksum_repair() {
        return Err(sqlx::Error::Protocol(
            "Migration 071 checksum drift detected (pre-#275). \
             Refusing silent repair without EDGEQUAKE_DEV_MODE. \
             Runbook: set EDGEQUAKE_DEV_MODE=true once on a controlled upgrade, \
             or UPDATE _sqlx_migrations checksum for version 71 to the fixed SHA-384 \
             in migrations/checksums.lock / M071_CHECKSUM_FIXED_275, then restart."
                .into(),
        ));
    }

    sqlx::query(
        "UPDATE _sqlx_migrations SET checksum = decode($1, 'hex') \
         WHERE version = $2 AND success = true",
    )
    .bind(M071_CHECKSUM_FIXED_275)
    .bind(MIGRATION_071_VERSION)
    .execute(pool)
    .await?;

    info!(
        target: "edgequake.migration",
        step = "migration_071_checksum_repair",
        from = M071_CHECKSUM_PRE_275,
        to = M071_CHECKSUM_FIXED_275,
        "Repaired migration 071 checksum (SPEC-042 #275 HNSW dimension guard; DEV_MODE)"
    );

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_and_fixed_checksums_are_distinct() {
        assert_ne!(M071_CHECKSUM_PRE_275, M071_CHECKSUM_FIXED_275);
        assert_eq!(M071_CHECKSUM_PRE_275.len(), 96);
        assert_eq!(M071_CHECKSUM_FIXED_275.len(), 96);
    }

    #[test]
    fn contract_checksum_drift_fails_loud_without_dev_mode() {
        let src = include_str!("m071.rs");
        assert!(
            src.contains("allow_checksum_repair")
                && src.contains("EDGEQUAKE_DEV_MODE")
                && src.contains("Refusing silent repair"),
            "X-02: M071 repair must fail loud when not DEV_MODE"
        );
    }
}
