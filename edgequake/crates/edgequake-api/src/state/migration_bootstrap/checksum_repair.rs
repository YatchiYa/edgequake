//! Checksum repair authorization (LAW-MIG / SPEC-083 X-02 / SPEC-090 §8.3).
//!
//! # First principles
//!
//! 1. **Applied migration SQL is immutable.** Edit → new version. Never patch a
//!    shipped `NNN_*.sql` body to “fix” field DBs (sqlx stores SHA-384 in
//!    `_sqlx_migrations`; byte drift aborts migrate).
//! 2. **Exception = bookkeeping only.** When a shipped body *must* change for
//!    source/LAW-C3 parity and the migration already applied (effect done),
//!    rewrite the stored checksum via an allowlisted repair module — never
//!    silently, never by re-running the SQL.
//! 3. **Authorization is narrow.** Prefer
//!    `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=71,78,118,121,125,131` over blanket
//!    `EDGEQUAKE_DEV_MODE` (which also disables auth). Local `make_dev` migrate
//!    sets the allowlist; production leaves both unset (fail loud).

/// Env: comma-separated migration versions allowed for one-shot checksum rewrite.
pub const ALLOW_CHECKSUM_REPAIR_ENV: &str = "EDGEQUAKE_ALLOW_CHECKSUM_REPAIR";

/// Versions that have a known broken→fixed repair module (Makefile SSOT twin).
///
/// When adding a repair module, append here **and** update
/// `KNOWN_CHECKSUM_REPAIR_VERSIONS` in the root `Makefile`.
pub const KNOWN_CHECKSUM_REPAIR_VERSIONS: &[i64] = &[71, 78, 118, 121, 125, 131];

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name).map(|v| v == "1" || v.eq_ignore_ascii_case("true")),
        Ok(true)
    )
}

/// Parse `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` (comma/space separated i64 versions).
pub fn parse_allow_checksum_repair_list(raw: &str) -> Vec<i64> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|part| {
            let t = part.trim();
            if t.is_empty() {
                return None;
            }
            t.parse::<i64>().ok()
        })
        .collect()
}

/// Authorize rewriting `_sqlx_migrations.checksum` for `version`.
///
/// Order:
/// 1. `EDGEQUAKE_DEV_MODE=1|true` → allow (local frictionless / legacy path)
/// 2. `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` contains `version` → allow (scoped)
/// 3. else deny (production default)
pub fn allow_checksum_repair(version: i64) -> bool {
    if env_truthy("EDGEQUAKE_DEV_MODE") {
        return true;
    }
    match std::env::var(ALLOW_CHECKSUM_REPAIR_ENV) {
        Ok(raw) => parse_allow_checksum_repair_list(&raw).contains(&version),
        Err(_) => false,
    }
}

/// Fail-loud protocol message shared by repair modules.
pub fn refuse_silent_repair_message(version: i64, reason: &str) -> String {
    format!(
        "Migration {version} checksum drift detected ({reason}). \
         Refusing silent repair without authorization. \
         Local: make_dev passes EDGEQUAKE_ALLOW_CHECKSUM_REPAIR / DEV_MODE. \
         Controlled upgrade: {ALLOW_CHECKSUM_REPAIR_ENV}={version} once, then unset. \
         Spec: specs/111-issues/10-migration-immutability.md."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allow_list_accepts_commas_and_spaces() {
        assert_eq!(
            parse_allow_checksum_repair_list("71, 78,125"),
            vec![71, 78, 125]
        );
        assert_eq!(parse_allow_checksum_repair_list("131"), vec![131]);
        assert!(parse_allow_checksum_repair_list("").is_empty());
        assert!(parse_allow_checksum_repair_list("nope").is_empty());
    }

    #[test]
    fn known_versions_are_sorted_unique() {
        let mut sorted = KNOWN_CHECKSUM_REPAIR_VERSIONS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted, KNOWN_CHECKSUM_REPAIR_VERSIONS);
    }

    #[test]
    fn refuse_message_names_scoped_env() {
        let msg = refuse_silent_repair_message(125, "SPEC-111 cast");
        assert!(msg.contains("EDGEQUAKE_ALLOW_CHECKSUM_REPAIR"));
        assert!(msg.contains("125"));
        assert!(msg.contains("10-migration-immutability"));
    }
}
