//! SPEC-091 W4 serving fence — SQL pushdown for vector search results.
//!
//! Fail-closed: when the serving fence is enabled (default on; unset counts as
//! on), chunk vectors are visible only with `chunk_serving_state.state = 'ready'`.
//! Non-chunk ids (entity / relationship vectors) always pass through.
//!
//! The state lookup resolves chunk ids via the `UNIQUE (document_id,
//! chunk_index)` btree — one indexed round trip per search, only when on.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::StorageError;
use crate::kv_key_schema::kv_keys;
use crate::serving_fence::serving_fence_enabled_from_env;
use crate::traits::VectorSearchResult;

static FENCE_FILTERED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Results hidden by the serving fence since process start (SRE signal).
pub fn serving_fence_filtered_total() -> u64 {
    FENCE_FILTERED_TOTAL.load(Ordering::Relaxed)
}

/// Post-filter `results` by serving readiness. No-op when the fence is off.
pub async fn apply_serving_fence(
    pool: &PgPool,
    results: Vec<VectorSearchResult>,
) -> Result<Vec<VectorSearchResult>, StorageError> {
    if !serving_fence_enabled_from_env() || results.is_empty() {
        return Ok(results);
    }

    let mut parseable: Vec<(Uuid, i32)> = Vec::new();
    for result in &results {
        if let Some((doc_str, index)) = kv_keys::parse_doc_chunk(&result.id) {
            if let Ok(doc_uuid) = Uuid::parse_str(doc_str) {
                parseable.push((doc_uuid, index as i32));
            }
        }
    }
    if parseable.is_empty() {
        return Ok(results); // entity/relationship vectors — fence does not apply
    }

    let docs: Vec<Uuid> = parseable.iter().map(|p| p.0).collect();
    let idxs: Vec<i32> = parseable.iter().map(|p| p.1).collect();
    let ready_rows = sqlx::query_as::<_, (Uuid, i32)>(
        "SELECT c.document_id, c.chunk_index \
         FROM chunks c \
         JOIN public.chunk_serving_state s \
           ON s.chunk_id = c.id AND s.state = 'ready' \
         WHERE (c.document_id, c.chunk_index) IN (SELECT * FROM unnest($1::uuid[], $2::int[]))",
    )
    .bind(&docs)
    .bind(&idxs)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("serving fence state query failed: {e}")))?;

    let ready: HashSet<(Uuid, i32)> = ready_rows.into_iter().collect();
    let before = results.len();
    let filtered: Vec<VectorSearchResult> = results
        .into_iter()
        .filter(|result| {
            let Some((doc_str, index)) = kv_keys::parse_doc_chunk(&result.id) else {
                return true; // non-chunk vector — visible
            };
            match Uuid::parse_str(doc_str) {
                // Relational chunks always have UUID document ids: ready-only.
                Ok(doc_uuid) => ready.contains(&(doc_uuid, index as i32)),
                // Non-UUID ids cannot reference `chunks` (FK is uuid) → outside
                // the fence domain (e.g. entity vectors named "x-chunk-1").
                Err(_) => true,
            }
        })
        .collect();

    let hidden = before - filtered.len();
    if hidden > 0 {
        FENCE_FILTERED_TOTAL.fetch_add(hidden as u64, Ordering::Relaxed);
        tracing::debug!(
            hidden,
            kept = filtered.len(),
            "SPEC-091 serving fence hid non-ready chunks"
        );
    }
    Ok(filtered)
}

#[cfg(test)]
mod tests {
    #[test]
    fn contract_spec091_fence_uses_ready_join_not_seqscan() {
        // Regression guard: fence must resolve via chunks UNIQUE(document_id,
        // chunk_index) + serving-state PK — never a metadata->>'key' scan.
        // (Banned literal built at runtime so this test file stays clean.)
        let src = include_str!("serving_fence_query.rs");
        assert!(src.contains("s.state = 'ready'"));
        assert!(src.contains("unnest($1::uuid[], $2::int[])"));
        let banned = format!("metadata->>{}", "'legacy_chunk_key'");
        assert!(!src.contains(&banned));
        // Regression guard (realized 2026-07-29): the fence must join the SSOT
        // table `public.chunk_serving_state` — never the `edgequake` compat
        // schema, which has no chunk_serving_state view (write path uses public).
        // Built at runtime so the literal does not self-match via include_str!.
        assert!(src.contains("public.chunk_serving_state"));
        let wrong_schema = format!("{}.{}", "edgequake", "chunk_serving_state");
        assert!(!src.contains(&wrong_schema));
    }
}
