//! SPEC-111 residual: stamp `legacy_vector_id` on typed fleet embeddings.
//!
//! When typed rows exist (normalize join) but provenance is NULL, migration 131
//! (provenance-only) would ABORT. This job UPDATEs provenance without re-copying
//! vectors. Unique conflicts (many-legacy → one typed) increment `failed`.

use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Digest, Sha384};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::embedding_family::{
    entity_name_from_legacy_id, parse_relationship_legacy_key, EmbeddingFamily,
};
use crate::error::StorageError;
use crate::graph_batch_dedupe::normalize_relation_type_str;

use super::coverage::{
    count_stamp_verify_coverage, list_vector_tables_ex, load_entity_name_index,
    resolve_relationship_id, resolve_workspace_id, scan_fleet_stamp_batch, EntityNameIndex,
};
use super::runner::{BackfillJob, BatchOutcome, VerifyReport};

const DESCRIPTOR_DEF: &str = concat!(
    "iw2-fleet-provenance-stamp/v2:",
    "source=legacy_vectors_fleet:keyset_per_table;",
    "action=update_legacy_vector_id_via_normalize_resolve;",
    "verify=stampable_provenance_coverage"
);

pub struct FleetProvenanceStampJob;

impl FleetProvenanceStampJob {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FleetProvenanceStampJob {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BackfillJob for FleetProvenanceStampJob {
    fn step_id(&self) -> &'static str {
        "iw2-fleet-provenance-stamp"
    }

    fn step_sha384(&self) -> String {
        Sha384::digest(DESCRIPTOR_DEF.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    fn schema_generation(&self) -> i32 {
        1
    }

    fn initial_cursor(&self) -> Value {
        json!({
            "family": EmbeddingFamily::Entity.backfill_family_key(),
            "table": Value::Null,
            "last_id": ""
        })
    }

    async fn estimate_total(&self, pool: &PgPool) -> Result<i64, StorageError> {
        // Rows that are join-resolvable but not yet drop-covered ≈ stamp work.
        // Cheap upper bound: uncovered fleet count (provenance gaps).
        super::coverage::count_uncovered_fleet_rows(pool).await
    }

    async fn run_batch(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cursor: &Value,
        limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        let family_key = cursor
            .get("family")
            .and_then(Value::as_str)
            .unwrap_or("entity");
        let family = match family_key {
            "relationship" => EmbeddingFamily::Relationship,
            "report" => EmbeddingFamily::Report,
            _ => EmbeddingFamily::Entity,
        };

        let tables = list_vector_tables_ex(&mut **tx).await?;
        if tables.is_empty() {
            return Ok(BatchOutcome {
                scanned: 0,
                written: 0,
                failed: 0,
                next_cursor: None,
            });
        }

        let cur_table = cursor
            .get("table")
            .and_then(Value::as_str)
            .map(str::to_string);
        let last_id = cursor
            .get("last_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        let active_idx = match cur_table.as_deref() {
            Some(t) => match tables.iter().position(|x| x == t) {
                Some(i) => i,
                None => match tables.iter().position(|x| x.as_str() > t) {
                    Some(i) => i,
                    None => return advance_family_or_finish(family, tables),
                },
            },
            None => 0,
        };
        let start_id = if cur_table.as_deref() == tables.get(active_idx).map(String::as_str) {
            last_id.clone()
        } else {
            String::new()
        };
        let table = &tables[active_idx];

        let rows = scan_fleet_stamp_batch(tx, table, family, &start_id, limit).await?;
        if rows.is_empty() {
            if active_idx + 1 < tables.len() {
                return Ok(BatchOutcome {
                    scanned: 0,
                    written: 0,
                    failed: 0,
                    next_cursor: Some(json!({
                        "family": family.backfill_family_key(),
                        "table": tables[active_idx + 1],
                        "last_id": ""
                    })),
                });
            }
            return advance_family_or_finish(family, tables);
        }

        let scanned = rows.len() as i64;
        let next_id = rows.last().map(|(id, _, _)| id.clone()).unwrap_or_default();
        let mut written = 0i64;
        let mut failed = 0i64;
        let mut index_cache: std::collections::HashMap<Uuid, EntityNameIndex> =
            std::collections::HashMap::new();

        for (id, meta, col_ws) in &rows {
            // Already stamped → skip
            if fleet_row_drop_covered_tx(tx, id, family).await? {
                continue;
            }
            let ws = resolve_workspace_id(meta.as_ref(), col_ws.as_deref());
            let Some(ws) = ws else {
                failed += 1;
                continue;
            };
            if let std::collections::hash_map::Entry::Vacant(e) = index_cache.entry(ws) {
                e.insert(load_entity_name_index(tx, ws).await?);
            }
            let index = index_cache.get(&ws).expect("just inserted");

            let stamp_result = match family {
                EmbeddingFamily::Entity => stamp_entity(tx, id, index).await,
                EmbeddingFamily::Relationship => stamp_relationship(tx, id, ws, index).await,
                EmbeddingFamily::Report => stamp_report(tx, id).await,
            };
            match stamp_result {
                Ok(StampOutcome::Written) => written += 1,
                Ok(StampOutcome::Skipped) => {}
                Ok(StampOutcome::Failed) => failed += 1,
                Err(e) => {
                    // Unique violation → durable miss
                    if let StorageError::Database(msg) = &e {
                        if msg.contains("unique")
                            || msg.contains("duplicate")
                            || msg.contains("23505")
                        {
                            failed += 1;
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }

        Ok(BatchOutcome {
            scanned,
            written,
            failed,
            next_cursor: Some(json!({
                "family": family.backfill_family_key(),
                "table": table,
                "last_id": next_id
            })),
        })
    }

    async fn verify(&self, pool: &PgPool) -> Result<VerifyReport, StorageError> {
        // Verify = provenance coverage for **stampable** fleet rows only
        // (typed spine exists via normalize join, or already covered).
        let (expected, actual) = count_stamp_verify_coverage(pool).await?;
        let stalls = super::coverage::count_provenance_stall_rows(pool).await?;
        let uncovered = super::coverage::count_uncovered_fleet_rows(pool).await?;
        Ok(VerifyReport {
            metric: format!(
                "iw2-fleet-provenance-stamp stampable={expected} covered={actual} \
                 uncovered_all={uncovered} stalls={stalls}"
            ),
            expected,
            actual,
            sampled: 0,
            // Surface dual-legacy stalls as mismatches so operators see them
            // in job verify output (coverage gap already fails actual < expected).
            mismatches: stalls as usize,
        })
    }
}

enum StampOutcome {
    Written,
    Skipped,
    Failed,
}

async fn fleet_row_drop_covered_tx(
    tx: &mut Transaction<'_, Postgres>,
    legacy_id: &str,
    family: EmbeddingFamily,
) -> Result<bool, StorageError> {
    let sql = match family {
        EmbeddingFamily::Entity => {
            "SELECT EXISTS (SELECT 1 FROM public.entity_embeddings WHERE legacy_vector_id = $1)"
        }
        EmbeddingFamily::Relationship => {
            "SELECT EXISTS (SELECT 1 FROM public.relationship_embeddings WHERE legacy_vector_id = $1)"
        }
        EmbeddingFamily::Report => {
            "SELECT EXISTS (SELECT 1 FROM public.report_embeddings \
             WHERE legacy_vector_id = $1 OR report_id = $1)"
        }
    };
    match sqlx::query_scalar::<_, bool>(sql)
        .bind(legacy_id)
        .fetch_one(&mut **tx)
        .await
    {
        Ok(v) => Ok(v),
        Err(sqlx::Error::Database(db))
            if db.code().as_deref() == Some("42703")
                || db.message().contains("legacy_vector_id") =>
        {
            Ok(false)
        }
        Err(e) => Err(StorageError::Database(format!("stamp covered check: {e}"))),
    }
}

async fn stamp_entity(
    tx: &mut Transaction<'_, Postgres>,
    legacy_id: &str,
    index: &EntityNameIndex,
) -> Result<StampOutcome, StorageError> {
    let Some(name) = entity_name_from_legacy_id(legacy_id) else {
        return Ok(StampOutcome::Failed);
    };
    let Some(eid) = index.resolve(name) else {
        return Ok(StampOutcome::Failed);
    };
    let has: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM public.entity_embeddings WHERE entity_id = $1)",
    )
    .bind(eid)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("stamp entity exists: {e}")))?;
    if !has {
        return Ok(StampOutcome::Failed);
    }
    let res = sqlx::query(
        "UPDATE public.entity_embeddings \
         SET legacy_vector_id = $1 \
         WHERE entity_id = $2 \
           AND legacy_vector_id IS NULL",
    )
    .bind(legacy_id)
    .bind(eid)
    .execute(&mut **tx)
    .await;
    match res {
        Ok(r) if r.rows_affected() > 0 => Ok(StampOutcome::Written),
        Ok(_) => {
            // Row already has a different legacy_vector_id → collision
            let existing: Option<String> = sqlx::query_scalar(
                "SELECT legacy_vector_id FROM public.entity_embeddings \
                 WHERE entity_id = $1 LIMIT 1",
            )
            .bind(eid)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| StorageError::Database(e.to_string()))?;
            if existing.as_deref() == Some(legacy_id) {
                Ok(StampOutcome::Skipped)
            } else {
                Ok(StampOutcome::Failed)
            }
        }
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => {
            Ok(StampOutcome::Failed)
        }
        Err(e) => Err(StorageError::Database(format!("stamp entity update: {e}"))),
    }
}

async fn stamp_relationship(
    tx: &mut Transaction<'_, Postgres>,
    legacy_id: &str,
    ws: Uuid,
    index: &EntityNameIndex,
) -> Result<StampOutcome, StorageError> {
    let Some((src, tgt, rel_type)) = parse_relationship_legacy_key(legacy_id) else {
        return Ok(StampOutcome::Failed);
    };
    let rel_type = normalize_relation_type_str(&rel_type);
    let Some(rid) = resolve_relationship_id(tx, ws, &src, &tgt, &rel_type, index).await? else {
        return Ok(StampOutcome::Failed);
    };
    let has: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM public.relationship_embeddings WHERE relationship_id = $1)",
    )
    .bind(rid)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| StorageError::Database(format!("stamp rel exists: {e}")))?;
    if !has {
        return Ok(StampOutcome::Failed);
    }
    let res = sqlx::query(
        "UPDATE public.relationship_embeddings \
         SET legacy_vector_id = $1 \
         WHERE relationship_id = $2 \
           AND legacy_vector_id IS NULL",
    )
    .bind(legacy_id)
    .bind(rid)
    .execute(&mut **tx)
    .await;
    match res {
        Ok(r) if r.rows_affected() > 0 => Ok(StampOutcome::Written),
        Ok(_) => Ok(StampOutcome::Failed),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => {
            Ok(StampOutcome::Failed)
        }
        Err(e) => Err(StorageError::Database(format!("stamp rel update: {e}"))),
    }
}

async fn stamp_report(
    tx: &mut Transaction<'_, Postgres>,
    legacy_id: &str,
) -> Result<StampOutcome, StorageError> {
    let res = sqlx::query(
        "UPDATE public.report_embeddings \
         SET legacy_vector_id = $1 \
         WHERE report_id = $1 \
           AND (legacy_vector_id IS NULL OR legacy_vector_id = $1)",
    )
    .bind(legacy_id)
    .execute(&mut **tx)
    .await;
    match res {
        Ok(r) if r.rows_affected() > 0 => Ok(StampOutcome::Written),
        Ok(_) => Ok(StampOutcome::Failed),
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23505") => {
            Ok(StampOutcome::Failed)
        }
        Err(e) => Err(StorageError::Database(format!("stamp report update: {e}"))),
    }
}

fn advance_family_or_finish(
    family: EmbeddingFamily,
    tables: Vec<String>,
) -> Result<BatchOutcome, StorageError> {
    let next_family = match family {
        EmbeddingFamily::Entity => Some(EmbeddingFamily::Relationship),
        EmbeddingFamily::Relationship => Some(EmbeddingFamily::Report),
        EmbeddingFamily::Report => None,
    };
    if let Some(next) = next_family {
        let first_table = tables.first().cloned().unwrap_or_default();
        return Ok(BatchOutcome {
            scanned: 0,
            written: 0,
            failed: 0,
            next_cursor: Some(json!({
                "family": next.backfill_family_key(),
                "table": if first_table.is_empty() { Value::Null } else { json!(first_table) },
                "last_id": ""
            })),
        });
    }
    Ok(BatchOutcome {
        scanned: 0,
        written: 0,
        failed: 0,
        next_cursor: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_id_is_stable() {
        assert_eq!(
            FleetProvenanceStampJob::new().step_id(),
            "iw2-fleet-provenance-stamp"
        );
    }
}
