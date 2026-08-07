//! Migration 131 — SPEC-111 fleet drop provenance checksum repair.
//!
//! Bodies:
//! - pre-SPEC-111: exact-name-only coverage
//! - SPEC-111 first: provenance + exact-name fallback (workspace-unscoped)
//! - SPEC-111 residual harden: provenance-only (LAW-C3 ≡ advisor)

use sqlx::PgPool;
use tracing::info;

use super::super::checksum_repair::{allow_checksum_repair, refuse_silent_repair_message};
use super::super::MIGRATION_131_VERSION;

/// SHA-384 of pre-SPEC-111 M131 (exact-name-only coverage).
pub(super) const M131_CHECKSUM_BROKEN_PRE111: &str =
    "461fa2a7c560513df711f954edd4f24444c91cd0385a70189e41cecdebaf2f53cca49c932122b0d002407a6c7fc0dbe8";

/// SHA-384 of first SPEC-111 M131 (provenance + exact-name fallback).
pub(super) const M131_CHECKSUM_BROKEN_SPEC111_FALLBACK: &str =
    "d6bc6c00b753f8599248dda86ce5d314e147491bcbb9932273c43afcbfc84a5d51c6a797387dfffeeca00588dc02c896";

/// SHA-384 of SPEC-111 residual harden M131 (provenance-only) — must match `checksums.lock`.
pub(super) const M131_CHECKSUM_FIXED_SPEC111: &str =
    "1b42205577666dc31fa346c42eb8e787c78208b6438da2822245ec61d65f3d538df8f985b132b7e3a3930b7272c87a14";

pub async fn repair_migration_131_checksum_if_needed(pool: &PgPool) -> Result<bool, sqlx::Error> {
    if !super::super::helpers::sqlx_migrations_table_exists(pool).await? {
        return Ok(false);
    }

    let current: Option<String> = sqlx::query_scalar(
        "SELECT encode(checksum, 'hex') FROM _sqlx_migrations \
         WHERE version = $1 AND success = true",
    )
    .bind(MIGRATION_131_VERSION)
    .fetch_optional(pool)
    .await?;

    let Some(current) = current else {
        return Ok(false);
    };

    let from = if current == M131_CHECKSUM_BROKEN_PRE111 {
        M131_CHECKSUM_BROKEN_PRE111
    } else if current == M131_CHECKSUM_BROKEN_SPEC111_FALLBACK {
        M131_CHECKSUM_BROKEN_SPEC111_FALLBACK
    } else {
        return Ok(false);
    };

    if !allow_checksum_repair(MIGRATION_131_VERSION) {
        return Err(sqlx::Error::Protocol(refuse_silent_repair_message(
            MIGRATION_131_VERSION,
            "SPEC-111 provenance-only guard",
        )));
    }

    sqlx::query(
        "UPDATE _sqlx_migrations SET checksum = decode($1, 'hex') \
         WHERE version = $2 AND success = true",
    )
    .bind(M131_CHECKSUM_FIXED_SPEC111)
    .bind(MIGRATION_131_VERSION)
    .execute(pool)
    .await?;

    info!(
        target: "edgequake.migration",
        step = "migration_131_checksum_repair",
        from,
        to = M131_CHECKSUM_FIXED_SPEC111,
        "Repaired migration 131 checksum (SPEC-111 residual; DEV_MODE)"
    );

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_and_fixed_checksums_are_distinct() {
        assert_ne!(M131_CHECKSUM_BROKEN_PRE111, M131_CHECKSUM_FIXED_SPEC111);
        assert_ne!(
            M131_CHECKSUM_BROKEN_SPEC111_FALLBACK,
            M131_CHECKSUM_FIXED_SPEC111
        );
        assert_eq!(M131_CHECKSUM_BROKEN_PRE111.len(), 96);
        assert_eq!(M131_CHECKSUM_FIXED_SPEC111.len(), 96);
    }

    #[test]
    fn contract_checksum_drift_uses_shared_allow_helper() {
        let src = include_str!("m131.rs");
        assert!(
            src.contains("allow_checksum_repair(MIGRATION_131_VERSION)")
                && src.contains("refuse_silent_repair_message"),
            "LAW-MIG / X-02: m131 must use shared checksum_repair helper"
        );
    }
}
