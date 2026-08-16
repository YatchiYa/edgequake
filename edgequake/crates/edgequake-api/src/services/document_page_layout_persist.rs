//! Persist SPEC-128 page layout sidecar into `document_pages` / `page_layout_regions`.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use tracing::{info, warn};
use uuid::Uuid;

use edgequake_pdf::{load_page_layout_sidecar, BBoxPdf};
use edgequake_storage::{
    DocumentPageLayoutStorage, LayoutBBoxPdf, ReplaceDocumentPagesRequest, UpsertDocumentPage,
    UpsertPageLayoutRegion,
};

use crate::error::{ApiError, ApiResult};

fn to_storage_bbox(b: BBoxPdf) -> LayoutBBoxPdf {
    LayoutBBoxPdf {
        x0: f64::from(b.x0),
        y0: f64::from(b.y0),
        x1: f64::from(b.x1),
        y1: f64::from(b.y1),
    }
}

/// Load `{assets_root}/page_layout.json` and replace all pages for the document.
pub async fn persist_page_layout_from_dir(
    storage: &dyn DocumentPageLayoutStorage,
    document_id: Uuid,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let Some(sidecar) = load_page_layout_sidecar(assets_root) else {
        return Ok(0);
    };
    let n_pages = sidecar.pages.len();
    let pages: Vec<UpsertDocumentPage> = sidecar
        .pages
        .into_iter()
        .map(|p| UpsertDocumentPage {
            page_number: p.page_number as i32,
            width_pt: f64::from(p.width_pt),
            height_pt: f64::from(p.height_pt),
            rotation: p.rotation,
            cropbox_pdf: p.cropbox_pdf.map(to_storage_bbox),
            raster_width_px: None,
            raster_height_px: None,
            layout_model: p.layout_model,
            layout_status: p.layout_status,
            regions: p
                .regions
                .into_iter()
                .map(|r| UpsertPageLayoutRegion {
                    class: r.class,
                    source: r.source,
                    bbox_pdf: to_storage_bbox(r.bbox_pdf),
                    confidence: r.confidence,
                    reading_order: r.reading_order,
                    asset_path: r.asset_path,
                    extra: r.extra,
                })
                .collect(),
        })
        .collect();
    storage
        .replace_document_pages(ReplaceDocumentPagesRequest {
            document_id,
            workspace_id,
            pages,
        })
        .await
        .map_err(ApiError::from)?;
    info!(%document_id, pages = n_pages, "Persisted SPEC-128 page layout");
    Ok(n_pages)
}

pub async fn persist_page_layout_with_storage(
    storage: Option<&Arc<dyn DocumentPageLayoutStorage>>,
    document_id: &str,
    workspace_id: Uuid,
    assets_root: &Path,
) -> ApiResult<usize> {
    let Some(store) = storage else {
        return Ok(0);
    };
    let doc = Uuid::parse_str(document_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid document_id: {e}")))?;
    persist_page_layout_from_dir(store.as_ref(), doc, workspace_id, assets_root).await
}

/// Best-effort persist; logs and swallows errors (fail-open for ingest).
pub async fn persist_page_layout_best_effort(
    storage: Option<&Arc<dyn DocumentPageLayoutStorage>>,
    document_id: &str,
    workspace_id: Uuid,
    assets_root: &Path,
) {
    let started = Instant::now();
    match persist_page_layout_with_storage(storage, document_id, workspace_id, assets_root).await {
        Ok(n) if n > 0 => {
            edgequake_observability::record_page_layout_persisted(n as u64);
            edgequake_observability::record_ingest_stage_duration(
                "page_layout_persist",
                started.elapsed().as_secs_f64(),
            );
        }
        Ok(_) => {
            edgequake_observability::record_page_layout_persist_skipped();
        }
        Err(e) => {
            edgequake_observability::record_page_layout_persist_error();
            warn!(document_id, error = %e, "Failed to persist page layout");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_pdf::{
        BBoxPdf, PageLayoutPageSidecar, PageLayoutRegionSidecar, PageLayoutSidecar,
        PAGE_LAYOUT_SIDECAR,
    };
    use edgequake_storage::MemoryPageLayoutStorage;

    #[tokio::test]
    async fn persist_sidecar_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let sidecar = PageLayoutSidecar {
            pages: vec![PageLayoutPageSidecar {
                page_number: 3,
                width_pt: 612.0,
                height_pt: 792.0,
                rotation: 0,
                cropbox_pdf: None,
                layout_model: Some("l0-l1".into()),
                layout_status: "extracted".into(),
                regions: vec![PageLayoutRegionSidecar {
                    class: "figure".into(),
                    source: "l1_paint".into(),
                    bbox_pdf: BBoxPdf {
                        x0: 61.2,
                        y0: 396.0,
                        x1: 306.0,
                        y1: 594.0,
                    },
                    confidence: Some(0.9),
                    reading_order: Some(1),
                    asset_path: Some("assets/page-0003-fig-01.png".into()),
                    extra: serde_json::json!({}),
                }],
            }],
        };
        std::fs::write(
            dir.path().join(PAGE_LAYOUT_SIDECAR),
            serde_json::to_string(&sidecar).unwrap(),
        )
        .unwrap();
        let store = MemoryPageLayoutStorage::new();
        let doc = Uuid::new_v4();
        let ws = Uuid::new_v4();
        let n = persist_page_layout_from_dir(&store, doc, ws, dir.path())
            .await
            .unwrap();
        assert_eq!(n, 1);
        let bundle = store.get_page_layout(&ws, &doc, 3).await.unwrap().unwrap();
        assert_eq!(bundle.regions.len(), 1);
        assert_eq!(bundle.page.region_count, 1);
    }

    #[tokio::test]
    async fn persist_missing_sidecar_is_zero() {
        let dir = tempfile::tempdir().unwrap();
        let store = MemoryPageLayoutStorage::new();
        let n = persist_page_layout_from_dir(&store, Uuid::new_v4(), Uuid::new_v4(), dir.path())
            .await
            .unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn persist_without_storage_is_zero() {
        let dir = tempfile::tempdir().unwrap();
        let n = persist_page_layout_with_storage(
            None,
            &Uuid::new_v4().to_string(),
            Uuid::new_v4(),
            dir.path(),
        )
        .await
        .unwrap();
        assert_eq!(n, 0);
    }

    fn write_one_page_sidecar(dir: &std::path::Path) {
        let sidecar = PageLayoutSidecar {
            pages: vec![PageLayoutPageSidecar {
                page_number: 1,
                width_pt: 612.0,
                height_pt: 792.0,
                rotation: 0,
                cropbox_pdf: None,
                layout_model: Some("l0-l1".into()),
                layout_status: "extracted".into(),
                regions: vec![],
            }],
        };
        std::fs::write(
            dir.join(PAGE_LAYOUT_SIDECAR),
            serde_json::to_string(&sidecar).unwrap(),
        )
        .unwrap();
    }

    struct FailingPageLayoutStorage;

    #[async_trait::async_trait]
    impl DocumentPageLayoutStorage for FailingPageLayoutStorage {
        async fn replace_document_pages(
            &self,
            _request: ReplaceDocumentPagesRequest,
        ) -> std::result::Result<(), edgequake_storage::StorageError> {
            Err(edgequake_storage::StorageError::Database(
                "forced persist failure".into(),
            ))
        }

        async fn list_document_pages(
            &self,
            _workspace_id: &Uuid,
            _document_id: &Uuid,
        ) -> std::result::Result<
            Vec<edgequake_storage::DocumentPage>,
            edgequake_storage::StorageError,
        > {
            Ok(vec![])
        }

        async fn get_page_layout(
            &self,
            _workspace_id: &Uuid,
            _document_id: &Uuid,
            _page_number: i32,
        ) -> std::result::Result<
            Option<edgequake_storage::PageLayoutBundle>,
            edgequake_storage::StorageError,
        > {
            Ok(None)
        }

        async fn delete_pages_for_document(
            &self,
            _workspace_id: &Uuid,
            _document_id: &Uuid,
        ) -> std::result::Result<u64, edgequake_storage::StorageError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn best_effort_ok_pages_records_without_panic() {
        let dir = tempfile::tempdir().unwrap();
        write_one_page_sidecar(dir.path());
        let store: Arc<dyn DocumentPageLayoutStorage> = Arc::new(MemoryPageLayoutStorage::new());
        persist_page_layout_best_effort(
            Some(&store),
            &Uuid::new_v4().to_string(),
            Uuid::new_v4(),
            dir.path(),
        )
        .await;
    }

    #[tokio::test]
    async fn best_effort_missing_sidecar_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let store: Arc<dyn DocumentPageLayoutStorage> = Arc::new(MemoryPageLayoutStorage::new());
        persist_page_layout_best_effort(
            Some(&store),
            &Uuid::new_v4().to_string(),
            Uuid::new_v4(),
            dir.path(),
        )
        .await;
    }

    #[tokio::test]
    async fn best_effort_storage_error_is_swallowed() {
        let dir = tempfile::tempdir().unwrap();
        write_one_page_sidecar(dir.path());
        let store: Arc<dyn DocumentPageLayoutStorage> = Arc::new(FailingPageLayoutStorage);
        persist_page_layout_best_effort(
            Some(&store),
            &Uuid::new_v4().to_string(),
            Uuid::new_v4(),
            dir.path(),
        )
        .await;
    }
}
