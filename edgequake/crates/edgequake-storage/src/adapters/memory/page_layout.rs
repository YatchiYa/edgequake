//! In-memory page layout storage (SPEC-128 tests).

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;
use crate::page_layout_storage::*;

use super::lock::map_lock_err;

#[derive(Debug, Default)]
pub struct MemoryPageLayoutStorage {
    pages: RwLock<HashMap<(Uuid, i32), DocumentPage>>,
    regions: RwLock<HashMap<Uuid, Vec<PageLayoutRegion>>>,
}

impl MemoryPageLayoutStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DocumentPageLayoutStorage for MemoryPageLayoutStorage {
    async fn replace_document_pages(&self, request: ReplaceDocumentPagesRequest) -> Result<()> {
        let _ = self
            .delete_pages_for_document(&request.workspace_id, &request.document_id)
            .await?;
        let now = Utc::now();
        let mut pages = self.pages.write().map_err(map_lock_err)?;
        let mut regions = self.regions.write().map_err(map_lock_err)?;
        for p in request.pages {
            let page_id = Uuid::new_v4();
            let regs: Vec<PageLayoutRegion> = p
                .regions
                .into_iter()
                .map(|r| PageLayoutRegion {
                    region_id: Uuid::new_v4(),
                    page_id,
                    document_id: request.document_id,
                    workspace_id: request.workspace_id,
                    class: r.class,
                    source: r.source,
                    bbox_pdf: r.bbox_pdf,
                    confidence: r.confidence,
                    reading_order: r.reading_order,
                    asset_path: r.asset_path,
                    extra: r.extra,
                    created_at: now,
                })
                .collect();
            let page = DocumentPage {
                page_id,
                document_id: request.document_id,
                workspace_id: request.workspace_id,
                page_number: p.page_number,
                width_pt: p.width_pt,
                height_pt: p.height_pt,
                rotation: p.rotation,
                cropbox_pdf: p.cropbox_pdf,
                raster_width_px: p.raster_width_px,
                raster_height_px: p.raster_height_px,
                layout_model: p.layout_model,
                layout_status: p.layout_status,
                region_count: regs.len() as i32,
                created_at: now,
                updated_at: now,
            };
            pages.insert((request.document_id, p.page_number), page);
            regions.insert(page_id, regs);
        }
        Ok(())
    }

    async fn list_document_pages(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentPage>> {
        let guard = self.pages.read().map_err(map_lock_err)?;
        let mut out: Vec<DocumentPage> = guard
            .values()
            .filter(|p| p.document_id == *document_id && p.workspace_id == *workspace_id)
            .cloned()
            .collect();
        out.sort_by_key(|p| p.page_number);
        let regions = self.regions.read().map_err(map_lock_err)?;
        for p in &mut out {
            p.region_count = regions.get(&p.page_id).map(|r| r.len() as i32).unwrap_or(0);
        }
        Ok(out)
    }

    async fn get_page_layout(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        page_number: i32,
    ) -> Result<Option<PageLayoutBundle>> {
        let pages = self.pages.read().map_err(map_lock_err)?;
        let Some(page) = pages.get(&(*document_id, page_number)).cloned() else {
            return Ok(None);
        };
        let mut page = page;
        if page.workspace_id != *workspace_id {
            return Ok(None);
        }
        let regions = self.regions.read().map_err(map_lock_err)?;
        let regs = regions.get(&page.page_id).cloned().unwrap_or_default();
        page.region_count = regs.len() as i32;
        Ok(Some(PageLayoutBundle {
            page,
            regions: regs,
        }))
    }

    async fn delete_pages_for_document(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<u64> {
        let mut pages = self.pages.write().map_err(map_lock_err)?;
        let mut regions = self.regions.write().map_err(map_lock_err)?;
        let keys: Vec<(Uuid, i32)> = pages
            .iter()
            .filter(|(_, p)| p.document_id == *document_id && p.workspace_id == *workspace_id)
            .map(|(k, _)| *k)
            .collect();
        let n = keys.len() as u64;
        for k in keys {
            if let Some(p) = pages.remove(&k) {
                regions.remove(&p.page_id);
            }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn replace_and_get_layout() {
        let store = MemoryPageLayoutStorage::new();
        let doc = Uuid::new_v4();
        let ws = Uuid::new_v4();
        store
            .replace_document_pages(ReplaceDocumentPagesRequest {
                document_id: doc,
                workspace_id: ws,
                pages: vec![UpsertDocumentPage {
                    page_number: 1,
                    width_pt: 612.0,
                    height_pt: 792.0,
                    rotation: 0,
                    cropbox_pdf: None,
                    raster_width_px: None,
                    raster_height_px: None,
                    layout_model: Some("l0-l1".into()),
                    layout_status: "extracted".into(),
                    regions: vec![UpsertPageLayoutRegion {
                        class: "figure".into(),
                        source: "l1_paint".into(),
                        bbox_pdf: LayoutBBoxPdf {
                            x0: 10.0,
                            y0: 20.0,
                            x1: 100.0,
                            y1: 200.0,
                        },
                        confidence: None,
                        reading_order: Some(1),
                        asset_path: Some("assets/page-0001-fig-01.png".into()),
                        extra: serde_json::json!({}),
                    }],
                }],
            })
            .await
            .unwrap();
        let bundle = store
            .get_page_layout(&ws, &doc, 1)
            .await
            .unwrap()
            .expect("page");
        assert_eq!(bundle.regions.len(), 1);
        assert_eq!(bundle.page.width_pt, 612.0);
        let listed = store.list_document_pages(&ws, &doc).await.unwrap();
        assert_eq!(listed.len(), 1);
    }
}
