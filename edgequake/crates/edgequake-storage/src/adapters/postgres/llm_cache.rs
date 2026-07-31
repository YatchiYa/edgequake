//! SPEC-091 Wave D — typed home for LLM cache keys (`{hash}-cache`,
//! `{hash}-kwcache`, multimodal `{mode}-{type}:{hash}-cache`).
//!
//! WHY a separate migration-owned table and not the generic KV store: caches
//! are transient recomputation guards, not document facts — one typed table
//! with a TTL column beats a runtime-created JSONB grab-bag (SPEC-091 first
//! principle: schema ships via migrations, never runtime DDL).
//!
//! The KV adapter dispatches cache keys here when
//! `EDGEQUAKE_KV_FAMILY_CACHE=relational` (write-stop). Entries written before
//! the cutover stay in legacy KV until it expires or is dropped — a cold cache
//! only costs one LLM recomputation, never correctness.

use serde_json::Value;
use sqlx::PgPool;

use crate::error::StorageError;

/// Whether a KV key belongs to the cache family (`{hash}-cache` / `-kwcache`).
/// Excludes document shell keys (those end `-metadata`/`-content`/`-chunk-N`).
pub fn is_cache_key(key: &str) -> bool {
    key.ends_with("-cache") || key.ends_with("-kwcache")
}

/// Typed upsert scoped to the caller's namespace (migration 124 composite PK
/// `(cache_key, namespace)` — per-tenant isolation parity with the retired
/// per-namespace KV tables). Authoritative in relational mode: failures
/// propagate; a lost cache entry is only a missed recomputation guard.
pub async fn cache_upsert(
    pool: &PgPool,
    namespace: &str,
    pairs: &[(String, Value)],
) -> Result<(), StorageError> {
    if pairs.is_empty() {
        return Ok(());
    }
    let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
    let values: Vec<&Value> = pairs.iter().map(|(_, v)| v).collect();
    sqlx::query(
        "INSERT INTO public.llm_cache (cache_key, namespace, value) \
         SELECT k, $3, v FROM unnest($1::text[], $2::jsonb[]) AS batch(k, v) \
         ON CONFLICT (cache_key, namespace) \
         DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind(&keys)
    .bind(&values)
    .bind(namespace)
    .execute(pool)
    .await
    .map_err(|e| StorageError::Database(format!("llm_cache upsert failed: {e}")))?;
    Ok(())
}

/// Typed single-key read (`None` on miss/expired).
pub async fn cache_get(
    pool: &PgPool,
    namespace: &str,
    key: &str,
) -> Result<Option<Value>, StorageError> {
    let row: Option<(Value,)> = sqlx::query_as(
        "SELECT value FROM public.llm_cache WHERE cache_key = $1 AND namespace = $2 \
         AND (expires_at IS NULL OR expires_at > now())",
    )
    .bind(key)
    .bind(namespace)
    .fetch_optional(pool)
    .await
    .map_err(|e| StorageError::Database(format!("llm_cache get failed: {e}")))?;
    Ok(row.map(|(v,)| v))
}

/// Typed ordered batch read (miss → `None` at that position; expired rows are
/// treated as misses).
pub async fn cache_values_ordered(
    pool: &PgPool,
    namespace: &str,
    keys: &[String],
) -> Result<Vec<Option<Value>>, StorageError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<(Option<Value>,)> = sqlx::query_as(
        "SELECT c.value \
         FROM unnest($1::text[]) WITH ORDINALITY AS u(cache_key, ord) \
         LEFT JOIN public.llm_cache c ON c.cache_key = u.cache_key \
            AND c.namespace = $2 \
            AND (c.expires_at IS NULL OR c.expires_at > now()) \
         ORDER BY u.ord",
    )
    .bind(keys)
    .bind(namespace)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("llm_cache batch get failed: {e}")))?;
    Ok(rows.into_iter().map(|(v,)| v).collect())
}

/// Typed delete (idempotent).
pub async fn cache_delete(
    pool: &PgPool,
    namespace: &str,
    keys: &[String],
) -> Result<(), StorageError> {
    if keys.is_empty() {
        return Ok(());
    }
    sqlx::query("DELETE FROM public.llm_cache WHERE cache_key = ANY($1) AND namespace = $2")
        .bind(keys)
        .bind(namespace)
        .execute(pool)
        .await
        .map_err(|e| StorageError::Database(format!("llm_cache delete failed: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_classification() {
        assert!(is_cache_key("deadbeef-cache"));
        assert!(is_cache_key("deadbeef-kwcache"));
        assert!(is_cache_key("image-analysis:abc123-cache"));
        assert!(!is_cache_key(
            "019fa6e8-872e-7515-95d2-f15529ea64f3-metadata"
        ));
        assert!(!is_cache_key("doc-1-chunk-3"));
        assert!(!is_cache_key("doc:hash:ws:abc"));
    }
}
