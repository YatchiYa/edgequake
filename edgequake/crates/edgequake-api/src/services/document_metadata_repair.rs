//! Startup metadata integrity repair (SPEC-045).
//!
//! Scans all document metadata KV entries with order-preserving reads and
//! realigns JSON fields to their authoritative keys before orphan recovery.

use std::collections::HashMap;
use std::sync::Arc;

use edgequake_storage::document_metadata_integrity::{
    metadata_id_drift, overlay_relational_title, repair_document_metadata_in_place,
    DOCUMENT_METADATA_SUFFIX,
};
use edgequake_storage::traits::KVStorage;
use tracing::info;

/// Summary of a metadata integrity pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetadataRepairReport {
    pub scanned: usize,
    pub repaired: usize,
    pub id_drift_fixed: usize,
    pub title_restored: usize,
}

/// Load `(document_id, title)` from the relational table for title overlay.
#[cfg(feature = "postgres")]
async fn load_relational_titles(
    pool: &sqlx::PgPool,
) -> Result<HashMap<String, String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id::text, title FROM documents WHERE title IS NOT NULL AND title != ''",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

#[cfg(not(feature = "postgres"))]
#[allow(dead_code)]
async fn load_relational_titles(_pool: &()) -> Result<HashMap<String, String>, ()> {
    Ok(HashMap::new())
}

/// Repair all document metadata blobs so JSON `id` matches the KV key.
pub async fn repair_all_document_metadata(
    kv: Arc<dyn KVStorage>,
    #[cfg(feature = "postgres")] pg_pool: Option<&sqlx::PgPool>,
) -> crate::error::ApiResult<MetadataRepairReport> {
    let keys = kv
        .keys_with_suffix(DOCUMENT_METADATA_SUFFIX)
        .await
        .map_err(|e| crate::error::ApiError::Internal(format!("metadata repair key scan: {e}")))?;

    if keys.is_empty() {
        return Ok(MetadataRepairReport::default());
    }

    #[cfg(feature = "postgres")]
    let relational_titles = if let Some(pool) = pg_pool {
        load_relational_titles(pool).await.unwrap_or_default()
    } else {
        HashMap::new()
    };

    #[cfg(not(feature = "postgres"))]
    let relational_titles: HashMap<String, String> = HashMap::new();

    let values = kv.get_by_ids_ordered(&keys).await.map_err(|e| {
        crate::error::ApiError::Internal(format!("metadata repair batch read: {e}"))
    })?;

    let mut report = MetadataRepairReport {
        scanned: keys.len(),
        ..Default::default()
    };
    let mut upserts: Vec<(String, serde_json::Value)> = Vec::new();

    for (key, maybe_value) in keys.iter().zip(values.iter()) {
        let Some(mut value) = maybe_value.clone() else {
            continue;
        };
        let had_drift = metadata_id_drift(key, &value);
        let mut changed = repair_document_metadata_in_place(key, &mut value);

        if had_drift {
            report.id_drift_fixed += 1;
        }

        if let Some(doc_id) =
            edgequake_storage::document_metadata_integrity::document_id_from_metadata_key(key)
        {
            if let Some(rel_title) = relational_titles.get(&doc_id) {
                if overlay_relational_title(key, &mut value, rel_title, had_drift) {
                    report.title_restored += 1;
                    changed = true;
                }
            }
        }

        if changed {
            upserts.push((key.clone(), value));
            report.repaired += 1;
        }
    }

    if !upserts.is_empty() {
        kv.upsert(&upserts).await.map_err(|e| {
            crate::error::ApiError::Internal(format!("metadata repair upsert: {e}"))
        })?;
        info!(
            scanned = report.scanned,
            repaired = report.repaired,
            id_drift_fixed = report.id_drift_fixed,
            title_restored = report.title_restored,
            "SPEC-045: repaired document metadata KV integrity"
        );
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::MemoryKVStorage;
    use serde_json::json;

    #[tokio::test]
    async fn repairs_swapped_metadata_blobs_on_startup() {
        let kv = Arc::new(MemoryKVStorage::new("spec045-repair"));
        kv.initialize().await.unwrap();

        // Simulate corruption: doc-a key holds doc-b's id/title.
        kv.upsert(&[
            (
                "doc-a-metadata".into(),
                json!({ "id": "doc-b", "title": "wrong.pdf", "status": "pending" }),
            ),
            (
                "doc-b-metadata".into(),
                json!({ "id": "doc-a", "title": "other.pdf", "status": "completed" }),
            ),
        ])
        .await
        .unwrap();

        let report = repair_all_document_metadata(
            kv.clone(),
            #[cfg(feature = "postgres")]
            None,
        )
        .await
        .unwrap();

        assert_eq!(report.repaired, 2);
        assert_eq!(report.id_drift_fixed, 2);

        let a = kv.get_by_id("doc-a-metadata").await.unwrap().unwrap();
        let b = kv.get_by_id("doc-b-metadata").await.unwrap().unwrap();
        assert_eq!(a["id"], "doc-a");
        assert_eq!(b["id"], "doc-b");
    }
}
