//! SPEC-105 — legacy store census SSOT (LAW-L4).
//!
//! One SQL shape for “do `eq_%_kv` / `eq_%_vectors` still exist?” used by
//! cutover flag guard, migration 142 posture, and era-aware readers.
//!
//! ## First principles
//!
//! - A store that exists will be used (LAW-L1) — census is the truth, not env alone.
//! - ≤0.22 mid-upgrade keeps census > 0 until confirm-drop (LAW-L5 / L6).

use sqlx::PgPool;

use crate::error::StorageError;

/// Live counts of legacy public relations (never inferred from env).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyStoreCensus {
    /// Number of `public.eq_%_kv` tables.
    pub kv_table_count: i64,
    /// Number of `public.eq_%_vectors` tables (excludes `*_vectors_stats`).
    pub vectors_table_count: i64,
}

impl LegacyStoreCensus {
    pub fn kv_present(self) -> bool {
        self.kv_table_count > 0
    }

    pub fn vectors_present(self) -> bool {
        self.vectors_table_count > 0
    }

    pub fn any_legacy_table(self) -> bool {
        self.kv_present() || self.vectors_present()
    }
}

/// Count legacy KV / vectors relations in `public`.
pub async fn legacy_store_census(pool: &PgPool) -> Result<LegacyStoreCensus, StorageError> {
    let kv_table_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_name LIKE 'eq\_%\_kv' ESCAPE '\'
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Database(format!("legacy kv census failed: {e}")))?;

    // Match SPEC-091 fleet naming: `eq_%_vectors` but not `eq_%_vectors_stats`.
    let vectors_table_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_name LIKE 'eq\_%\_vectors' ESCAPE '\'
          AND table_name NOT LIKE '%\_stats' ESCAPE '\'
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Database(format!("legacy vectors census failed: {e}")))?;

    Ok(LegacyStoreCensus {
        kv_table_count,
        vectors_table_count,
    })
}

/// True when any legacy KV or vectors table still holds at least one row.
pub async fn any_legacy_rows(pool: &PgPool) -> Result<bool, StorageError> {
    let census = legacy_store_census(pool).await?;
    if !census.any_legacy_table() {
        return Ok(false);
    }

    // Dynamic relation names from information_schema (already allowlisted by LIKE).
    let tables: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND (
                table_name LIKE 'eq\_%\_kv' ESCAPE '\'
             OR (
                    table_name LIKE 'eq\_%\_vectors' ESCAPE '\'
                AND table_name NOT LIKE '%\_stats' ESCAPE '\'
             )
          )
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("legacy table list failed: {e}")))?;

    for name in tables {
        // Identifiers come only from information_schema LIKE filters (eq_* shape).
        if !is_safe_legacy_relname(&name) {
            continue;
        }
        let sql = format!("SELECT EXISTS (SELECT 1 FROM public.\"{name}\" LIMIT 1)");
        let has: bool = sqlx::query_scalar(&sql)
            .fetch_one(pool)
            .await
            .map_err(|e| StorageError::Database(format!("legacy row probe {name} failed: {e}")))?;
        if has {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_safe_legacy_relname(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_relname_allowlist() {
        assert!(is_safe_legacy_relname("eq_eq_default_kv"));
        assert!(is_safe_legacy_relname("eq_eq_default_vectors"));
        assert!(!is_safe_legacy_relname("eq;drop"));
        assert!(!is_safe_legacy_relname(""));
        assert!(!is_safe_legacy_relname("1bad"));
    }

    #[test]
    fn census_helpers() {
        let c = LegacyStoreCensus {
            kv_table_count: 1,
            vectors_table_count: 0,
        };
        assert!(c.kv_present());
        assert!(!c.vectors_present());
        assert!(c.any_legacy_table());
    }
}
