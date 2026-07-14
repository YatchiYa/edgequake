//! Persist / materialize document multimodal assets (SPEC-047 durable mm-assets).
//!
//! SSOT: filesystem layout from `document_assets` + `drawing_tags` paths;
//! durable copy in `DocumentMmAssetStorage` (Postgres BYTEA).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::json;
use tracing::{debug, info, warn};
use uuid::Uuid;

use edgequake_storage::{
    asset_id_from_path, classify_mm_asset_path, guess_mm_asset_content_type, normalize_mm_asset_id,
    normalize_mm_asset_path, DocumentMmAssetStorage, DocumentMmAssetSummary, StoreMmAssetRequest,
};

use crate::error::{ApiError, ApiResult};
use crate::services::document_assets::document_mm_assets_root;
use crate::state::AppState;

#[cfg(feature = "postgres")]
fn get_mm_asset_storage(state: &AppState) -> ApiResult<Arc<dyn DocumentMmAssetStorage>> {
    state
        .storage
        .mm_asset_storage
        .as_ref()
        .cloned()
        .ok_or_else(|| ApiError::Internal("MM asset storage not initialized".into()))
}

#[cfg(not(feature = "postgres"))]
fn get_mm_asset_storage(_state: &AppState) -> ApiResult<Arc<dyn DocumentMmAssetStorage>> {
    Err(ApiError::Internal(
        "MM asset storage not available (postgres feature disabled)".into(),
    ))
}

/// Collect PNG (and common image) files under `{assets_root}/assets/`.
pub fn collect_mm_asset_files(assets_root: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    let assets_dir = assets_root.join("assets");
    if !assets_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&assets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if !(lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".webp")
            || lower.ends_with(".gif"))
        {
            continue;
        }
        let rel = format!("assets/{name}");
        out.push((rel, path));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Build upsert requests from files already written under `assets_root`.
pub fn store_requests_from_dir(
    document_id: Uuid,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<Vec<StoreMmAssetRequest>> {
    let files = collect_mm_asset_files(assets_root).map_err(|e| {
        ApiError::Internal(format!("scan mm-assets dir {}: {e}", assets_root.display()))
    })?;
    let mut requests = Vec::with_capacity(files.len());
    for (rel, path) in files {
        let bytes = std::fs::read(&path)
            .map_err(|e| ApiError::Internal(format!("read mm-asset {}: {e}", path.display())))?;
        if bytes.is_empty() {
            warn!(%rel, "skipping empty mm-asset file");
            continue;
        }
        let path_norm =
            normalize_mm_asset_path(&rel).map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let asset_id = normalize_mm_asset_id(&asset_id_from_path(&path_norm))
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;
        let (kind, page_num) = classify_mm_asset_path(&path_norm);
        requests.push(StoreMmAssetRequest {
            document_id,
            workspace_id,
            asset_id,
            asset_path: path_norm,
            content_type: guess_mm_asset_content_type(&rel).to_string(),
            asset_data: bytes,
            asset_kind: kind,
            page_num,
        });
    }
    Ok(requests)
}

async fn mark_has_mm_assets_metadata(state: &AppState, document_id: &str) -> ApiResult<()> {
    use edgequake_storage::kv_keys;

    for key in [
        kv_keys::staging_doc_metadata(document_id),
        kv_keys::doc_metadata(document_id),
    ] {
        if let Some(mut meta) = state
            .storage
            .kv_storage
            .get_by_id(&key)
            .await
            .map_err(ApiError::from)?
        {
            if let Some(obj) = meta.as_object_mut() {
                obj.insert("has_mm_assets".into(), json!(true));
                state
                    .storage
                    .kv_storage
                    .upsert(&[(key, meta)])
                    .await
                    .map_err(ApiError::from)?;
            }
        }
    }
    Ok(())
}

/// Persist vision page/chart PNGs from disk into DB (idempotent upsert).
pub async fn persist_document_mm_assets_from_dir(
    storage: &dyn DocumentMmAssetStorage,
    document_id: Uuid,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let requests = store_requests_from_dir(document_id, workspace_id, assets_root)?;
    if requests.is_empty() {
        return Ok(0);
    }
    let n = requests.len();
    storage
        .store_assets(&requests)
        .await
        .map_err(ApiError::from)?;
    info!(
        %document_id,
        %workspace_id,
        count = n,
        assets_root = %assets_root.display(),
        "Persisted document mm-assets to database"
    );
    Ok(n)
}

/// AppState helper: persist + flag metadata for lineage/UI.
pub async fn persist_uploaded_mm_assets(
    state: &AppState,
    document_id: &str,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let Ok(document_uuid) = Uuid::parse_str(document_id) else {
        return Ok(0);
    };
    let storage = match get_mm_asset_storage(state) {
        Ok(s) => s,
        Err(_) => {
            debug!(%document_id, "mm-asset storage unavailable; skipping DB persist");
            return Ok(0);
        }
    };
    let n = persist_document_mm_assets_from_dir(
        storage.as_ref(),
        document_uuid,
        workspace_id,
        assets_root,
    )
    .await?;
    if n > 0 {
        let _ = mark_has_mm_assets_metadata(state, document_id).await;
    }
    Ok(n)
}

/// Same as [`persist_uploaded_mm_assets`] using processor-held storage (no AppState).
pub async fn persist_mm_assets_with_storage(
    storage: Option<&Arc<dyn DocumentMmAssetStorage>>,
    kv: &dyn edgequake_storage::traits::KVStorage,
    document_id: &str,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let Some(storage) = storage else {
        return Ok(0);
    };
    let Ok(document_uuid) = Uuid::parse_str(document_id) else {
        return Ok(0);
    };
    let n = persist_document_mm_assets_from_dir(
        storage.as_ref(),
        document_uuid,
        workspace_id,
        assets_root,
    )
    .await?;
    if n > 0 {
        use edgequake_storage::kv_keys;
        for key in [
            kv_keys::staging_doc_metadata(document_id),
            kv_keys::doc_metadata(document_id),
        ] {
            if let Some(mut meta) = kv.get_by_id(&key).await.map_err(ApiError::from)? {
                if let Some(obj) = meta.as_object_mut() {
                    obj.insert("has_mm_assets".into(), json!(true));
                    kv.upsert(&[(key, meta)]).await.map_err(ApiError::from)?;
                }
            }
        }
    }
    Ok(n)
}

/// Rehydrate DB assets onto the local cache dir (resume / multi-pod analyze).
pub async fn materialize_mm_assets_to_dir(
    storage: &dyn DocumentMmAssetStorage,
    document_id: Uuid,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let summaries = storage
        .list_asset_summaries(&workspace_id, &document_id)
        .await
        .map_err(ApiError::from)?;
    if summaries.is_empty() {
        return Ok(0);
    }
    let mut written = 0usize;
    for summary in &summaries {
        let dest = assets_root.join(&summary.asset_path);
        if dest.is_file() {
            continue;
        }
        let Some(asset) = storage
            .get_asset(&workspace_id, &document_id, &summary.asset_path)
            .await
            .map_err(ApiError::from)?
        else {
            continue;
        };
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ApiError::Internal(format!("create mm-assets dir {}: {e}", parent.display()))
            })?;
        }
        std::fs::write(&dest, &asset.asset_data)
            .map_err(|e| ApiError::Internal(format!("write mm-asset {}: {e}", dest.display())))?;
        written += 1;
    }
    if written > 0 {
        info!(
            %document_id,
            written,
            total = summaries.len(),
            "Materialized mm-assets from database to disk cache"
        );
    }
    Ok(written)
}

/// Load one asset by relative path: DB first, filesystem fallback.
pub async fn load_mm_asset_bytes(
    storage: Option<&dyn DocumentMmAssetStorage>,
    document_id: &str,
    workspace_id: Option<Uuid>,
    asset_path: &str,
) -> ApiResult<(Vec<u8>, String)> {
    let path =
        normalize_mm_asset_path(asset_path).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if let (Some(storage), Some(ws), Ok(doc)) =
        (storage, workspace_id, Uuid::parse_str(document_id))
    {
        if let Some(asset) = storage
            .get_asset(&ws, &doc, &path)
            .await
            .map_err(ApiError::from)?
        {
            return Ok((asset.asset_data, asset.content_type));
        }
    }

    let full = document_mm_assets_root(document_id).join(&path);
    let bytes = tokio::fs::read(&full)
        .await
        .map_err(|_| ApiError::NotFound(format!("mm-asset not found: {path}")))?;
    Ok((bytes, guess_mm_asset_content_type(&path).to_string()))
}

/// Load one asset by stable id (document + asset_id): DB first, then path derived from id.
pub async fn load_mm_asset_bytes_by_id(
    storage: Option<&dyn DocumentMmAssetStorage>,
    document_id: &str,
    workspace_id: Option<Uuid>,
    asset_id: &str,
) -> ApiResult<(Vec<u8>, String)> {
    let id = normalize_mm_asset_id(asset_id).map_err(|e| ApiError::BadRequest(e.to_string()))?;

    if let (Some(storage), Some(ws), Ok(doc)) =
        (storage, workspace_id, Uuid::parse_str(document_id))
    {
        if let Some(asset) = storage
            .get_asset_by_id(&ws, &doc, &id)
            .await
            .map_err(ApiError::from)?
        {
            return Ok((asset.asset_data, asset.content_type));
        }
    }

    // FS fallback: id is filename stem under assets/
    let path = format!("assets/{id}.png");
    load_mm_asset_bytes(storage, document_id, workspace_id, &path).await
}

/// Delete all mm-assets for a document (DB + local cache dir).
pub async fn delete_document_mm_assets(
    storage: Option<&dyn DocumentMmAssetStorage>,
    document_id: &str,
    workspace_id: Option<Uuid>,
) -> ApiResult<u64> {
    let mut deleted = 0u64;
    if let (Some(storage), Some(ws), Ok(doc)) =
        (storage, workspace_id, Uuid::parse_str(document_id))
    {
        deleted = storage
            .delete_assets_for_document(&ws, &doc)
            .await
            .map_err(ApiError::from)?;
    }
    let root = document_mm_assets_root(document_id);
    if root.exists() {
        if let Err(e) = tokio::fs::remove_dir_all(&root).await {
            warn!(
                %document_id,
                path = %root.display(),
                error = %e,
                "Failed to remove mm-assets filesystem cache"
            );
        } else {
            info!(%document_id, path = %root.display(), "Removed mm-assets filesystem cache");
        }
    }
    Ok(deleted)
}

/// Lineage-safe summaries (no BYTEA).
pub async fn list_mm_asset_summaries_for_document(
    storage: Option<&dyn DocumentMmAssetStorage>,
    document_id: &str,
    workspace_id: Uuid,
) -> ApiResult<Vec<DocumentMmAssetSummary>> {
    let Some(storage) = storage else {
        return Ok(Vec::new());
    };
    let Ok(document_uuid) = Uuid::parse_str(document_id) else {
        return Ok(Vec::new());
    };
    storage
        .list_asset_summaries(&workspace_id, &document_uuid)
        .await
        .map_err(ApiError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::MemoryMmAssetStorage;
    use std::sync::Arc;

    #[tokio::test]
    async fn persist_and_reload_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        std::fs::write(assets.join("page-0001.png"), b"\x89PNG\r\n\x1a\n").unwrap();
        std::fs::write(assets.join("page-0001-chart.png"), b"\x89PNG chart").unwrap();

        let storage = Arc::new(MemoryMmAssetStorage::new());
        let doc = Uuid::new_v4();
        let ws = Uuid::new_v4();
        let n = persist_document_mm_assets_from_dir(storage.as_ref(), doc, ws, dir.path())
            .await
            .unwrap();
        assert_eq!(n, 2);

        let summaries = storage.list_asset_summaries(&ws, &doc).await.unwrap();
        assert_eq!(summaries.len(), 2);
        assert!(summaries.iter().any(|s| s.page_num == Some(1)));
        assert!(summaries
            .iter()
            .any(|s| s.asset_kind == edgequake_storage::ASSET_KIND_PAGE_CHART_CROP));
        assert!(summaries.iter().any(|s| s.asset_id == "page-0001"));
        assert!(summaries.iter().any(|s| s.asset_id == "page-0001-chart"));

        let asset = storage
            .get_asset_by_id(&ws, &doc, "page-0001")
            .await
            .unwrap()
            .expect("page asset by id");
        assert!(asset.asset_data.starts_with(b"\x89PNG"));
    }
}
