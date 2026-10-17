//! Document lineage export handlers.

use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;

use super::cache::cached_kv_get;
use crate::error::ApiError;
use crate::handlers::isolation::verify_document_access;
use crate::middleware::TenantContext;
use crate::state::StorageRuntime;

/// Query parameters for lineage export.
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct ExportParams {
    /// Export format: "json" (default) or "csv".
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "json".to_string()
}

/// Export complete document lineage as JSON or CSV file.
///
/// OODA-22: Returns lineage data as a downloadable file with proper
/// Content-Disposition headers. CSV format flattens the hierarchical
/// lineage into a table with one row per chunk.
///
/// @implements F5: Single API call retrieves complete document lineage tree
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/lineage/export",
    tag = "Lineage",
    params(
        ("document_id" = String, Path, description = "Document ID to export lineage for"),
        ExportParams,
    ),
    responses(
        (status = 200, description = "Lineage export file (JSON or CSV)"),
        (status = 404, description = "Document or lineage not found")
    )
)]
pub async fn export_document_lineage(
    State(storage): State<StorageRuntime>,
    tenant_ctx: TenantContext,
    Path(document_id): Path<String>,
    Query(params): Query<ExportParams>,
) -> Result<impl IntoResponse, ApiError> {
    // SECURITY: Verify the document belongs to the requesting tenant/workspace.
    verify_document_access(storage.kv_storage.as_ref(), &document_id, &tenant_ctx).await?;

    // OODA-23: Use cached KV lookup for export
    let lineage_key = format!("{}-lineage", document_id);
    let lineage_data = cached_kv_get(storage.kv_storage.as_ref(), &lineage_key)
        .await?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Lineage for document '{}' not found. \
                 Document may not have been processed yet.",
                document_id
            ))
        })?;

    // Read metadata for context (cached)
    let metadata_key =
        crate::services::document_metadata_scan::metadata_key_for_document(&document_id);
    let metadata = cached_kv_get(storage.kv_storage.as_ref(), &metadata_key)
        .await?
        .unwrap_or(serde_json::json!({"id": document_id}));

    let combined = serde_json::json!({
        "document_id": document_id,
        "metadata": metadata,
        "lineage": lineage_data,
    });

    match params.format.as_str() {
        "csv" => {
            // WHY: CSV flattens hierarchical lineage into a chunk-per-row table.
            // This is useful for spreadsheet analysis and data pipeline ingestion.
            let mut lineage_data = lineage_data;
            fill_chunk_text(storage.kv_storage.as_ref(), &mut lineage_data).await;
            let csv_content = lineage_to_csv(&document_id, &lineage_data);
            let filename = format!("{}-lineage.csv", document_id);
            Ok((
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                csv_content,
            ))
        }
        _ => {
            // Default: JSON export
            let json_content =
                serde_json::to_string_pretty(&combined).unwrap_or_else(|_| "{}".to_string());
            let filename = format!("{}-lineage.json", document_id);
            Ok((
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        "application/json; charset=utf-8".to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                json_content,
            ))
        }
    }
}

/// Persisted lineage chunks carry ids + offsets only; fill `content` and
/// `token_count` from the chunk store in one batched read. Older payloads that
/// already inline those fields are left untouched.
async fn fill_chunk_text(
    kv: &dyn edgequake_storage::traits::KVStorage,
    lineage: &mut serde_json::Value,
) {
    let Some(chunks) = lineage.get_mut("chunks").and_then(|c| c.as_array_mut()) else {
        return;
    };
    let ids: Vec<String> = chunks
        .iter()
        .filter(|c| c.get("content").is_none())
        .filter_map(|c| c.get("chunk_id")?.as_str().map(str::to_owned))
        .collect();
    if ids.is_empty() {
        return;
    }
    let Ok(records) = kv.get_by_ids_ordered(&ids).await else {
        return;
    };
    let by_id: std::collections::HashMap<String, serde_json::Value> = ids
        .into_iter()
        .zip(records)
        .filter_map(|(id, rec)| Some((id, rec?)))
        .collect();
    for chunk in chunks.iter_mut() {
        let Some(rec) = chunk
            .get("chunk_id")
            .and_then(|v| v.as_str())
            .and_then(|id| by_id.get(id))
        else {
            continue;
        };
        if let Some(obj) = chunk.as_object_mut() {
            for key in ["content", "token_count"] {
                if let Some(v) = rec.get(key) {
                    obj.entry(key).or_insert_with(|| v.clone());
                }
            }
        }
    }
}

/// Convert lineage data to CSV format.
///
/// WHY: Flattens the document → chunks hierarchy into a tabular format
/// with one row per chunk, suitable for spreadsheets and data analysis.
pub(super) fn lineage_to_csv(document_id: &str, lineage: &serde_json::Value) -> String {
    let mut csv = String::new();
    csv.push_str(
        "document_id,chunk_index,content_preview,tokens,start_line,end_line,entity_count\n",
    );

    if let Some(chunks) = lineage.get("chunks").and_then(|c| c.as_array()) {
        for chunk in chunks {
            let index = chunk
                .get("chunk_index")
                .or_else(|| chunk.get("index"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let content = chunk.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let preview = edgequake_observability::utf8_prefix(content, 100);
            // WHY: Escape CSV fields — wrap in quotes and double any internal quotes
            let escaped_preview = preview.replace('"', "\"\"").replace('\n', " ");
            let tokens = chunk
                .get("tokens")
                .or_else(|| chunk.get("token_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let start_line = chunk
                .get("start_line")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_default();
            let end_line = chunk
                .get("end_line")
                .and_then(|v| v.as_u64())
                .map(|v| v.to_string())
                .unwrap_or_default();
            // Legacy payloads carry `entity_count`; persisted lineage carries `entity_ids`.
            let entity_count = chunk
                .get("entity_count")
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    chunk
                        .get("entity_ids")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len() as u64)
                })
                .unwrap_or(0);

            csv.push_str(&format!(
                "{},{},\"{}\",{},{},{},{}\n",
                document_id, index, escaped_preview, tokens, start_line, end_line, entity_count
            ));
        }
    }

    csv
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::traits::KVStorage;

    /// N1: chunk text + tokens come from the chunk store; entity_count from entity_ids.
    #[tokio::test]
    async fn csv_fills_chunk_text_from_store_for_persisted_lineage() {
        let kv = edgequake_storage::MemoryKVStorage::new("lineage-export-n1");
        kv.upsert(&[(
            "doc-chunk-0".to_string(),
            serde_json::json!({ "content": "Sarah Chen leads", "token_count": 7 }),
        )])
        .await
        .unwrap();
        let mut lineage = serde_json::json!({
            "chunks": [{
                "chunk_id": "doc-chunk-0",
                "chunk_index": 0,
                "start_line": 1,
                "end_line": 11,
                "entity_ids": ["a", "b", "c", "d", "e", "f"]
            }]
        });
        fill_chunk_text(&kv, &mut lineage).await;
        let csv = lineage_to_csv("doc", &lineage);
        assert_eq!(
            csv.lines().nth(1),
            Some("doc,0,\"Sarah Chen leads\",7,1,11,6")
        );
    }
}
