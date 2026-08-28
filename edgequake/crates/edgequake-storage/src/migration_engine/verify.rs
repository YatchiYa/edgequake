//! SPEC-091 W1 verification: coverage + sampled content equality between the
//! KV chunk family and the relational `chunks` spine (07-migration-engine.md
//! §Verification; 11-e2e-test-matrix E2E-091-02).

use serde_json::Value;
use sqlx::PgPool;

use super::runner::VerifyReport;
use crate::error::StorageError;
use crate::kv_key_schema::kv_keys;

/// Deterministic 1/16 sample probe: `md5(key) LIKE '0%'` — index-friendly
/// (PK range scan + filter), stable across runs, no full-table random sort.
const SAMPLE_LIMIT: i64 = 200;

pub async fn verify_chunk_text_backfill(
    pool: &PgPool,
    kv_table: &str,
) -> Result<VerifyReport, StorageError> {
    // Migrated count is source-independent (the chunks spine always exists).
    let migrated = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM chunks WHERE metadata ? 'legacy_chunk_key'",
    )
    .fetch_one(pool)
    .await
    .map_err(|e| StorageError::Database(format!("verify chunks count failed: {e}")))?;

    // EC-35/Wave D: the KV source may already be dropped — the migration is
    // complete by definition, so report a clean pass instead of a 42P01 error.
    let chunk_re = super::coverage::LEGACY_CHUNK_VECTOR_ID_RE;
    let kv_chunks = match sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM {kv_table} WHERE key ~ '{chunk_re}'"
    ))
    .fetch_one(pool)
    .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => {
            return Ok(VerifyReport {
                metric: "chunk_text_coverage+content".into(),
                expected: 0,
                actual: migrated,
                sampled: 0,
                mismatches: 0,
            });
        }
        Err(e) => {
            return Err(StorageError::Database(format!(
                "verify kv count failed: {e}"
            )))
        }
    };

    // Sampled content equality (checksum metric, sampled at scale per plan).
    let sample_rows = sqlx::query_as::<_, (String, Value)>(&format!(
        "SELECT key, value FROM {kv_table} \
         WHERE key ~ '{chunk_re}' AND md5(key) LIKE '0%' \
         ORDER BY key LIMIT $1"
    ))
    .bind(SAMPLE_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("verify sample failed: {e}")))?;

    let mut doc_ids = Vec::new();
    let mut parsed = Vec::new();
    for (key, value) in &sample_rows {
        if let Some((doc_str, index)) = kv_keys::parse_doc_chunk(key) {
            if let Ok(doc_uuid) = uuid::Uuid::parse_str(doc_str) {
                doc_ids.push(doc_uuid);
                parsed.push((doc_uuid, index as i32, value.clone()));
            }
        }
    }

    let mut content_map: std::collections::HashMap<(uuid::Uuid, i32), String> = Default::default();
    if !doc_ids.is_empty() {
        let pairs: Vec<(uuid::Uuid, i32)> = parsed.iter().map(|(d, i, _)| (*d, *i)).collect();
        let docs: Vec<uuid::Uuid> = pairs.iter().map(|p| p.0).collect();
        let idxs: Vec<i32> = pairs.iter().map(|p| p.1).collect();
        content_map = sqlx::query_as::<_, (uuid::Uuid, i32, String)>(
            "SELECT document_id, chunk_index, content FROM chunks \
             WHERE (document_id, chunk_index) IN (SELECT * FROM unnest($1::uuid[], $2::int[]))",
        )
        .bind(&docs)
        .bind(&idxs)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Database(format!("verify chunks fetch failed: {e}")))?
        .into_iter()
        .map(|(d, i, c)| ((d, i), c))
        .collect();
    }

    let mut mismatches = 0usize;
    for (doc_uuid, index, value) in &parsed {
        let kv_content = value
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default();
        match content_map.get(&(*doc_uuid, *index)) {
            Some(rel_content) if rel_content == kv_content => {}
            _ => mismatches += 1,
        }
    }

    Ok(VerifyReport {
        metric: "chunk_text_coverage+content".into(),
        expected: kv_chunks,
        actual: migrated,
        sampled: parsed.len(),
        mismatches,
    })
}

/// SPEC-091 W3 verification: coverage + sampled vector equality between legacy
/// `eq_*_vectors` chunk rows and typed `chunk_embeddings`. 42P01-safe (EC-35):
/// dropped legacy source ⇒ complete by definition.
///
/// SPEC-139 LAW-139-3: `actual` is **per-table coverage** (chunks ⋈
/// chunk_embeddings for this legacy table's `-chunk-` ids), never
/// `COUNT(*) FROM chunk_embeddings` (global).
pub async fn verify_chunk_embedding_backfill(
    pool: &PgPool,
    vectors_table: &str,
    model_name: &str,
) -> Result<VerifyReport, StorageError> {
    let chunk_re = super::coverage::LEGACY_CHUNK_VECTOR_ID_RE;
    let legacy_chunks = match sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM {vectors_table} WHERE id ~ '{chunk_re}'"
    ))
    .fetch_one(pool)
    .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => {
            return Ok(VerifyReport {
                metric: "chunk_embedding_coverage+vector".into(),
                expected: 0,
                actual: 0,
                sampled: 0,
                mismatches: 0,
            });
        }
        Err(e) => {
            return Err(StorageError::Database(format!(
                "w3 verify legacy count failed: {e}"
            )))
        }
    };

    let covered = match sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM {vectors_table} v \
         WHERE v.id ~ '{chunk_re}' \
           AND EXISTS ( \
                SELECT 1 FROM public.chunks c \
                JOIN public.chunk_embeddings ce ON ce.chunk_id = c.id \
                WHERE c.document_id = left(v.id, 36)::uuid \
                  AND c.chunk_index = substring(v.id from 44)::int)"
    ))
    .fetch_one(pool)
    .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => 0,
        Err(e) => {
            return Err(StorageError::Database(format!(
                "w3 verify coverage count failed: {e}"
            )))
        }
    };

    // Sampled vector equality: legacy chunk rows joined to typed rows through
    // the chunks spine. Dimension mismatch counts as a mismatch.
    let sample = sqlx::query_as::<_, (String, String)>(&format!(
        "SELECT id, embedding::text FROM {vectors_table} \
         WHERE id ~ '{chunk_re}' AND md5(id) LIKE '0%' ORDER BY id LIMIT $1"
    ))
    .bind(SAMPLE_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(|e| StorageError::Database(format!("w3 verify sample failed: {e}")))?;

    let mut parsed: Vec<(uuid::Uuid, i32, Vec<f32>)> = Vec::new();
    for (id, emb_text) in &sample {
        if let Some((doc_str, index)) = kv_keys::parse_doc_chunk(id) {
            if let Ok(doc_uuid) = uuid::Uuid::parse_str(doc_str) {
                if let Some(emb) = parse_vector_text_for_verify(emb_text) {
                    parsed.push((doc_uuid, index as i32, emb));
                }
            }
        }
    }

    let mut mismatches = 0usize;
    let mut sampled = 0usize;
    if !parsed.is_empty() {
        let docs: Vec<uuid::Uuid> = parsed.iter().map(|(d, _, _)| *d).collect();
        let idxs: Vec<i32> = parsed.iter().map(|(_, i, _)| *i).collect();
        // Resolve typed embedding for the same (document, chunk) via spine +
        // model registry (dimension from the model's registered row).
        let typed_rows = sqlx::query_as::<_, (uuid::Uuid, i32, String)>(
            "SELECT c.document_id, c.chunk_index, ce.embedding::text \
             FROM chunks c \
             JOIN chunk_embeddings ce ON ce.chunk_id = c.id \
             JOIN embedding_models em ON em.id = ce.model_id AND em.name = $3 \
             WHERE (c.document_id, c.chunk_index) IN (SELECT * FROM unnest($1::uuid[], $2::int[]))",
        )
        .bind(&docs)
        .bind(&idxs)
        .bind(model_name)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Database(format!("w3 verify typed fetch failed: {e}")))?;
        let typed_map: std::collections::HashMap<(uuid::Uuid, i32), Vec<f32>> = typed_rows
            .into_iter()
            .filter_map(|(d, i, t)| parse_vector_text_for_verify(&t).map(|v| ((d, i), v)))
            .collect();

        for (doc_uuid, index, legacy_emb) in &parsed {
            sampled += 1;
            match typed_map.get(&(*doc_uuid, *index)) {
                Some(typed_emb)
                    if typed_emb.len() == legacy_emb.len()
                        && typed_emb
                            .iter()
                            .zip(legacy_emb.iter())
                            .all(|(a, b)| (a - b).abs() < 1e-3) => {}
                _ => mismatches += 1,
            }
        }
    }

    Ok(VerifyReport {
        metric: "chunk_embedding_coverage+vector".into(),
        expected: legacy_chunks,
        actual: covered,
        sampled,
        mismatches,
    })
}

fn parse_vector_text_for_verify(raw: &str) -> Option<Vec<f32>> {
    let inner = raw.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        out.push(part.trim().parse::<f32>().ok()?);
    }
    Some(out)
}

/// SPEC-091 IW2 verification: coverage + sampled vector equality for one fleet
/// family in one legacy table. 42P01-safe when the legacy source is gone.
///
/// SPEC-111 honesty closeout: `actual` is **provenance-only** coverage
/// (`legacy_vector_id` match; reports also allow `report_id`). Normalize is
/// write/stamp SSOT only — never used as verify coverage. Pre-143 schema
/// (missing column) fail-closes entity/rel `actual = 0`.
pub async fn verify_fleet_embedding_backfill(
    pool: &PgPool,
    vectors_table: &str,
    family: crate::embedding_family::EmbeddingFamily,
    model_name: &str,
) -> Result<VerifyReport, StorageError> {
    use crate::embedding_family::EmbeddingFamily;

    let typed_table = family.typed_table();

    let legacy_filter = match family {
        EmbeddingFamily::Entity => {
            format!("SELECT COUNT(*) FROM {vectors_table} WHERE id LIKE 'entity:%'")
        }
        EmbeddingFamily::Relationship => format!(
            "SELECT COUNT(*) FROM {vectors_table} WHERE id LIKE '%->%:%' \
             AND id NOT LIKE 'entity:%' AND id NOT LIKE 'community_report:%'"
        ),
        EmbeddingFamily::Report => {
            format!("SELECT COUNT(*) FROM {vectors_table} WHERE id LIKE 'community_report:%'")
        }
    };

    let legacy_rows = match sqlx::query_scalar::<_, i64>(&legacy_filter)
        .fetch_one(pool)
        .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("42P01") => {
            return Ok(VerifyReport {
                metric: format!("{typed_table}_coverage+vector"),
                expected: 0,
                actual: 0,
                sampled: 0,
                mismatches: 0,
            });
        }
        Err(e) => {
            return Err(StorageError::Database(format!(
                "iw2 verify legacy count failed: {e}"
            )))
        }
    };

    // Provenance-only coverage (≡ migration 131 / LAW-C3).
    let covered_sql = match family {
        EmbeddingFamily::Entity => format!(
            "SELECT COUNT(*) FROM {vectors_table} v \
             WHERE v.id LIKE 'entity:%' \
               AND EXISTS (SELECT 1 FROM public.entity_embeddings ee \
                           WHERE ee.legacy_vector_id = v.id)"
        ),
        EmbeddingFamily::Relationship => format!(
            "SELECT COUNT(*) FROM {vectors_table} v \
             WHERE v.id LIKE '%->%:%' AND v.id NOT LIKE 'entity:%' \
               AND v.id NOT LIKE 'community_report:%' \
               AND EXISTS (SELECT 1 FROM public.relationship_embeddings re \
                           WHERE re.legacy_vector_id = v.id)"
        ),
        EmbeddingFamily::Report => format!(
            "SELECT COUNT(*) FROM {vectors_table} v \
             WHERE v.id LIKE 'community_report:%' \
               AND (EXISTS (SELECT 1 FROM public.report_embeddings re \
                            WHERE re.legacy_vector_id = v.id) \
                    OR EXISTS (SELECT 1 FROM public.report_embeddings re \
                               WHERE re.report_id = v.id))"
        ),
    };

    let covered = match sqlx::query_scalar::<_, i64>(&covered_sql)
        .fetch_one(pool)
        .await
    {
        Ok(n) => n,
        Err(sqlx::Error::Database(db))
            if db.code().as_deref() == Some("42703")
                || db.message().contains("legacy_vector_id") =>
        {
            // Pre-143: entity/rel cannot be provenance-covered → fail closed.
            // Reports still allow report_id (matches migration 131).
            match family {
                EmbeddingFamily::Entity | EmbeddingFamily::Relationship => 0,
                EmbeddingFamily::Report => sqlx::query_scalar::<_, i64>(&format!(
                    "SELECT COUNT(*) FROM {vectors_table} v \
                     WHERE v.id LIKE 'community_report:%' \
                       AND EXISTS (SELECT 1 FROM public.report_embeddings re \
                                   WHERE re.report_id = v.id)"
                ))
                .fetch_one(pool)
                .await
                .unwrap_or(0),
            }
        }
        Err(e) => {
            return Err(StorageError::Database(format!(
                "iw2 verify coverage count failed: {e}"
            )))
        }
    };

    let sample_sql = match family {
        EmbeddingFamily::Entity => format!(
            "SELECT id, embedding::text FROM {vectors_table} \
             WHERE id LIKE 'entity:%' AND md5(id) LIKE '0%' ORDER BY id LIMIT $1"
        ),
        EmbeddingFamily::Relationship => format!(
            "SELECT id, embedding::text FROM {vectors_table} \
             WHERE id LIKE '%->%:%' AND id NOT LIKE 'entity:%' \
             AND id NOT LIKE 'community_report:%' AND md5(id) LIKE '0%' ORDER BY id LIMIT $1"
        ),
        EmbeddingFamily::Report => format!(
            "SELECT id, embedding::text FROM {vectors_table} \
             WHERE id LIKE 'community_report:%' AND md5(id) LIKE '0%' ORDER BY id LIMIT $1"
        ),
    };

    let sample = sqlx::query_as::<_, (String, String)>(&sample_sql)
        .bind(SAMPLE_LIMIT)
        .fetch_all(pool)
        .await
        .map_err(|e| StorageError::Database(format!("iw2 verify sample failed: {e}")))?;

    let mut mismatches = 0usize;
    let mut sampled = 0usize;
    for (id, emb_text) in sample {
        let Some(legacy_emb) = parse_vector_text_for_verify(&emb_text) else {
            mismatches += 1;
            continue;
        };
        sampled += 1;
        // Prefer provenance lookup (normalize-safe).
        let typed_emb: Option<String> = match family {
            EmbeddingFamily::Entity => sqlx::query_scalar(
                "SELECT ee.embedding::text FROM entity_embeddings ee \
                 JOIN embedding_models em ON em.id = ee.model_id AND em.name = $2 \
                 WHERE ee.legacy_vector_id = $1 LIMIT 1",
            )
            .bind(&id)
            .bind(model_name)
            .fetch_optional(pool)
            .await
            .unwrap_or(None),
            EmbeddingFamily::Relationship => sqlx::query_scalar(
                "SELECT re.embedding::text FROM relationship_embeddings re \
                 JOIN embedding_models em ON em.id = re.model_id AND em.name = $2 \
                 WHERE re.legacy_vector_id = $1 LIMIT 1",
            )
            .bind(&id)
            .bind(model_name)
            .fetch_optional(pool)
            .await
            .unwrap_or(None),
            EmbeddingFamily::Report => sqlx::query_scalar(
                "SELECT re.embedding::text FROM report_embeddings re \
                 JOIN embedding_models em ON em.id = re.model_id AND em.name = $2 \
                 WHERE re.legacy_vector_id = $1 OR re.report_id = $1 LIMIT 1",
            )
            .bind(&id)
            .bind(model_name)
            .fetch_optional(pool)
            .await
            .map_err(|e| StorageError::Database(format!("iw2 verify report fetch: {e}")))?,
        };
        match typed_emb.and_then(|t| parse_vector_text_for_verify(&t)) {
            Some(typed)
                if typed.len() == legacy_emb.len()
                    && typed
                        .iter()
                        .zip(legacy_emb.iter())
                        .all(|(a, b)| (a - b).abs() < 1e-3) => {}
            _ => mismatches += 1,
        }
    }

    Ok(VerifyReport {
        metric: format!("{typed_table}_coverage+vector"),
        expected: legacy_rows,
        actual: covered,
        sampled,
        mismatches,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn contract_spec091_verify_uses_deterministic_sample() {
        // Regression guard: verification must never sort the whole table at
        // random — md5-prefix sampling is the plan's answer. (Banned literal
        // built at runtime so this test file stays clean.)
        let src = include_str!("verify.rs");
        assert!(src.contains("md5(key) LIKE '0%'"));
        let banned = format!("ORDER BY {}random()", "");
        assert!(!src.contains(&banned));
    }
}
