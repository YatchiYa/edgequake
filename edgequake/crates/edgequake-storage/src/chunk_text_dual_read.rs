//! SPEC-091 W1 dual-read: relational shadow reads + compare counters.
//!
//! During the `dual` phase KV stays authoritative; every hydration also reads
//! the relational `chunks` spine and records compare metrics (hit / missing /
//! mismatch). In `relational` mode the spine is the only source and misses are
//! counted (no silent KV fallback — that is the cutover contract).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::kv_key_schema::kv_keys;

static SHADOW_QUERY_TOTAL: AtomicU64 = AtomicU64::new(0);
static RELATIONAL_HIT_TOTAL: AtomicU64 = AtomicU64::new(0);
static MISSING_IN_RELATIONAL_TOTAL: AtomicU64 = AtomicU64::new(0);
static CONTENT_MISMATCH_TOTAL: AtomicU64 = AtomicU64::new(0);
static RELATIONAL_MISS_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Shadow queries executed (dual phase).
pub fn dual_read_shadow_query_total() -> u64 {
    SHADOW_QUERY_TOTAL.load(Ordering::Relaxed)
}

/// Keys whose relational content matched the KV authority.
pub fn dual_read_hit_total() -> u64 {
    RELATIONAL_HIT_TOTAL.load(Ordering::Relaxed)
}

/// Keys present in KV but not yet in `chunks` (backfill lag indicator).
pub fn dual_read_missing_in_relational_total() -> u64 {
    MISSING_IN_RELATIONAL_TOTAL.load(Ordering::Relaxed)
}

/// Keys present in both with different content (hard gate breach).
pub fn dual_read_content_mismatch_total() -> u64 {
    CONTENT_MISMATCH_TOTAL.load(Ordering::Relaxed)
}

/// Relational-mode reads that found no row (post-cutover fallback counter).
pub fn relational_read_miss_total() -> u64 {
    RELATIONAL_MISS_TOTAL.load(Ordering::Relaxed)
}

/// Reconstruct the legacy KV chunk JSON shape from a relational row.
///
/// SSOT mapping for the read cutover: the relational writer mirrors the same
/// field set into `chunks.metadata` (`relational_chunk_writer`), so every
/// legacy consumer (`content`, `index`, offsets, lines, pages, section,
/// modality, source_file) observes an unchanged shape.
pub fn legacy_json_from_chunk_fields(
    document_id: Uuid,
    chunk_index: i32,
    content: String,
    start_offset: Option<i32>,
    end_offset: Option<i32>,
    token_count: Option<i32>,
    metadata: &Value,
) -> Value {
    let mut value = json!({
        "content": content,
        "document_id": document_id.to_string(),
        "index": chunk_index,
        "start_offset": start_offset.unwrap_or(0),
        "end_offset": end_offset.unwrap_or(0),
        "token_count": token_count.unwrap_or(0),
    });
    for field in [
        "start_line",
        "end_line",
        "page_start",
        "page_end",
        "section",
        "modality",
        "source_file",
    ] {
        if let Some(v) = metadata.get(field) {
            if !v.is_null() {
                value[field] = v.clone();
            }
        }
    }
    value
}

#[derive(Clone, sqlx::FromRow)]
struct RelationalChunkRow {
    document_id: Uuid,
    chunk_index: i32,
    content: String,
    start_offset: Option<i32>,
    end_offset: Option<i32>,
    token_count: Option<i32>,
    metadata: Value,
}

/// Fetch full relational rows for chunk-parseable keys, mapped back to their
/// legacy string keys. One indexed round trip on `UNIQUE (document_id,
/// chunk_index)` (LAW-D8: no `metadata->>` GIN scan).
async fn fetch_relational_chunk_rows(
    pool: &PgPool,
    keys: &[String],
) -> Result<HashMap<String, RelationalChunkRow>, StorageError> {
    let mut originals: Vec<(String, Uuid, i32)> = Vec::new();
    for key in keys {
        if let Some((doc_str, index)) = kv_keys::parse_doc_chunk(key) {
            if let Ok(doc_uuid) = Uuid::parse_str(doc_str) {
                originals.push((key.clone(), doc_uuid, index as i32));
            }
        }
    }
    if originals.is_empty() {
        return Ok(HashMap::new());
    }

    let docs: Vec<Uuid> = originals.iter().map(|(_, d, _)| *d).collect();
    let idxs: Vec<i32> = originals.iter().map(|(_, _, i)| *i).collect();
    let rows = sqlx::query_as::<_, RelationalChunkRow>(
        "SELECT document_id, chunk_index, content, start_offset, end_offset, \
                token_count, metadata FROM chunks \
         WHERE (document_id, chunk_index) IN (SELECT * FROM unnest($1::uuid[], $2::int[]))",
    )
    .bind(&docs)
    .bind(&idxs)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("dual-read chunks fetch failed: {e}")))?;

    let by_pair: HashMap<(Uuid, i32), RelationalChunkRow> = rows
        .into_iter()
        .map(|r| ((r.document_id, r.chunk_index), r))
        .collect();
    Ok(originals
        .into_iter()
        .filter_map(|(key, d, i)| by_pair.get(&(d, i)).map(|r| (key, r.clone())))
        .collect())
}

/// Fetch legacy-shaped chunk JSON from the relational spine for keys that
/// parse as `{doc_uuid}-chunk-{n}` (full record cutover read).
pub async fn fetch_relational_chunk_json(
    pool: &PgPool,
    keys: &[String],
) -> Result<HashMap<String, Value>, StorageError> {
    let rows = fetch_relational_chunk_rows(pool, keys).await?;
    Ok(rows
        .into_iter()
        .map(|(key, r)| {
            (
                key,
                legacy_json_from_chunk_fields(
                    r.document_id,
                    r.chunk_index,
                    r.content,
                    r.start_offset,
                    r.end_offset,
                    r.token_count,
                    &r.metadata,
                ),
            )
        })
        .collect())
}

/// Single-key relational read for the `get_by_id` dispatch. Call only for
/// chunk-parseable keys; `None` is a genuine miss (cutover contract: no
/// silent KV fallback in relational mode).
pub async fn relational_value_by_key(
    pool: &PgPool,
    key: &str,
) -> Result<Option<Value>, StorageError> {
    let map = fetch_relational_chunk_json(pool, std::slice::from_ref(&key.to_string())).await?;
    match map.get(key) {
        Some(value) => Ok(Some(value.clone())),
        None => {
            RELATIONAL_MISS_TOTAL.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }
}

/// Fetch chunk text from the relational spine for keys that parse as
/// `{doc_uuid}-chunk-{n}`. Uses the `UNIQUE (document_id, chunk_index)` btree
/// — one indexed round trip, no `metadata->>` GIN scan (LAW-D8).
pub async fn fetch_relational_chunk_texts(
    pool: &PgPool,
    keys: &[String],
) -> Result<HashMap<String, String>, StorageError> {
    let rows = fetch_relational_chunk_rows(pool, keys).await?;
    Ok(rows.into_iter().map(|(key, r)| (key, r.content)).collect())
}

/// Dual-phase shadow compare: KV values stay authoritative; counters observe
/// the relational projection. Never fails the read path.
pub async fn shadow_compare(pool: &PgPool, keys: &[String], kv_values: &[Option<Value>]) {
    SHADOW_QUERY_TOTAL.fetch_add(1, Ordering::Relaxed);
    let relational = match fetch_relational_chunk_texts(pool, keys).await {
        Ok(map) => map,
        Err(e) => {
            tracing::debug!(error = %e, "SPEC-091 dual-read shadow query failed (ignored)");
            return;
        }
    };
    for (key, kv_value) in keys.iter().zip(kv_values) {
        if kv_keys::parse_doc_chunk(key).is_none() {
            continue; // non-chunk key family — outside this cutover
        }
        let kv_content = kv_value
            .as_ref()
            .and_then(crate::chunk_content::content_from_kv_value);
        match (kv_content, relational.get(key)) {
            (Some(kv_c), Some(rel_c)) if *rel_c == kv_c => {
                RELATIONAL_HIT_TOTAL.fetch_add(1, Ordering::Relaxed);
            }
            (Some(_), Some(_)) => {
                CONTENT_MISMATCH_TOTAL.fetch_add(1, Ordering::Relaxed);
                tracing::warn!(key = %key, "SPEC-091 dual-read CONTENT MISMATCH (kv vs chunks)");
            }
            (Some(_), None) => {
                MISSING_IN_RELATIONAL_TOTAL.fetch_add(1, Ordering::Relaxed);
            }
            (None, _) => {}
        }
    }
}

/// Relational-phase ordered values for `get_by_ids_ordered`: chunk-parseable
/// keys resolve from `chunks` as full legacy-shaped JSON; other keys return
/// `None` here (caller merges KV).
pub async fn relational_values_ordered(
    pool: &PgPool,
    keys: &[String],
) -> Result<Vec<Option<Value>>, StorageError> {
    let relational = fetch_relational_chunk_json(pool, keys).await?;
    Ok(keys
        .iter()
        .map(|key| {
            // Non-chunk family — caller fills from KV.
            kv_keys::parse_doc_chunk(key)?;
            match relational.get(key) {
                Some(value) => Some(value.clone()),
                None => {
                    RELATIONAL_MISS_TOTAL.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_counters_start_at_zero_or_monotonic() {
        // Counters are process-global; just assert the getters work and are
        // monotonic within this test's own increments.
        let before = dual_read_shadow_query_total();
        SHADOW_QUERY_TOTAL.fetch_add(1, Ordering::Relaxed);
        assert_eq!(dual_read_shadow_query_total(), before + 1);
    }

    #[test]
    fn contract_spec091_non_chunk_keys_excluded_from_cutover() {
        // `wsdoc:...`, `compensation_quarantine:...` must never hit `chunks`.
        assert!(kv_keys::parse_doc_chunk("wsdoc:ws-1:doc").is_none());
        assert!(kv_keys::parse_doc_chunk("doc-1-chunk-3").is_some());
    }
}
