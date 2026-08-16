//! Per-page PDF layout regions (SPEC-128 overlay).
//!
//! Persist `bbox_pdf` only; `bbox_norm` is derived at read (LAW-128-4).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

/// Canonical overlay class (one mapper in `edgequake_pdf::page_layout`).
pub const LAYOUT_CLASS_FIGURE: &str = "figure";
pub const LAYOUT_CLASS_CHART: &str = "chart";
pub const LAYOUT_CLASS_TABLE: &str = "table";
pub const LAYOUT_CLASS_PARAGRAPH: &str = "paragraph";
pub const LAYOUT_CLASS_COLUMN: &str = "column";
pub const LAYOUT_CLASS_ABANDON: &str = "abandon";

/// PDF user-space box stored as JSON `{x0,y0,x1,y1}`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LayoutBBoxPdf {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// Overlay unit square (top-left origin).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LayoutBBoxNorm {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentPage {
    pub page_id: Uuid,
    pub document_id: Uuid,
    pub workspace_id: Uuid,
    pub page_number: i32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub rotation: i16,
    pub cropbox_pdf: Option<LayoutBBoxPdf>,
    pub raster_width_px: Option<i32>,
    pub raster_height_px: Option<i32>,
    pub layout_model: Option<String>,
    pub layout_status: String,
    /// Filled on read (list/get). Not stored as its own column.
    #[serde(default)]
    pub region_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayoutRegion {
    pub region_id: Uuid,
    pub page_id: Uuid,
    pub document_id: Uuid,
    pub workspace_id: Uuid,
    pub class: String,
    pub source: String,
    pub bbox_pdf: LayoutBBoxPdf,
    pub confidence: Option<f32>,
    pub reading_order: Option<i32>,
    pub asset_path: Option<String>,
    pub extra: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplaceDocumentPagesRequest {
    pub document_id: Uuid,
    pub workspace_id: Uuid,
    pub pages: Vec<UpsertDocumentPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpsertDocumentPage {
    pub page_number: i32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub rotation: i16,
    pub cropbox_pdf: Option<LayoutBBoxPdf>,
    pub raster_width_px: Option<i32>,
    pub raster_height_px: Option<i32>,
    pub layout_model: Option<String>,
    pub layout_status: String,
    pub regions: Vec<UpsertPageLayoutRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpsertPageLayoutRegion {
    pub class: String,
    pub source: String,
    pub bbox_pdf: LayoutBBoxPdf,
    pub confidence: Option<f32>,
    pub reading_order: Option<i32>,
    pub asset_path: Option<String>,
    pub extra: serde_json::Value,
}

/// Page + regions for GET `/pages/{n}/layout`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayoutBundle {
    pub page: DocumentPage,
    pub regions: Vec<PageLayoutRegion>,
}

#[async_trait]
pub trait DocumentPageLayoutStorage: Send + Sync {
    /// Replace all pages/regions for a document (reprocess-safe).
    async fn replace_document_pages(&self, request: ReplaceDocumentPagesRequest) -> Result<()>;

    async fn list_document_pages(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentPage>>;

    async fn get_page_layout(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        page_number: i32,
    ) -> Result<Option<PageLayoutBundle>>;

    async fn delete_pages_for_document(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<u64>;
}

/// LAW-128-4: derive overlay unit square from stored PDF box + page metrics.
pub fn bbox_norm_from_pdf(
    bbox: LayoutBBoxPdf,
    width_pt: f64,
    height_pt: f64,
    rotation: i16,
) -> LayoutBBoxNorm {
    let w = width_pt.max(1.0);
    let h = height_pt.max(1.0);
    let (x0, y0, x1, y1, dw, dh) = match rotation.rem_euclid(360) {
        90 => {
            let nx0 = bbox.y0;
            let ny0 = w - bbox.x1;
            let nx1 = bbox.y1;
            let ny1 = w - bbox.x0;
            (nx0.min(nx1), ny0.min(ny1), nx0.max(nx1), ny0.max(ny1), h, w)
        }
        180 => {
            let nx0 = w - bbox.x1;
            let ny0 = h - bbox.y1;
            let nx1 = w - bbox.x0;
            let ny1 = h - bbox.y0;
            (nx0.min(nx1), ny0.min(ny1), nx0.max(nx1), ny0.max(ny1), w, h)
        }
        270 => {
            let nx0 = h - bbox.y1;
            let ny0 = bbox.x0;
            let nx1 = h - bbox.y0;
            let ny1 = bbox.x1;
            (nx0.min(nx1), ny0.min(ny1), nx0.max(nx1), ny0.max(ny1), h, w)
        }
        _ => (bbox.x0, bbox.y0, bbox.x1, bbox.y1, w, h),
    };
    LayoutBBoxNorm {
        x: (x0 / dw).clamp(0.0, 1.0),
        y: (1.0 - (y1 / dh)).clamp(0.0, 1.0),
        w: ((x1 - x0) / dw).clamp(0.0, 1.0),
        h: ((y1 - y0) / dh).clamp(0.0, 1.0),
    }
}

/// Intersection-over-union of two overlay unit squares (G-layout-coord / G-overlay).
pub fn bbox_norm_iou(a: LayoutBBoxNorm, b: LayoutBBoxNorm) -> f64 {
    let ax1 = a.x + a.w;
    let ay1 = a.y + a.h;
    let bx1 = b.x + b.w;
    let by1 = b.y + b.h;
    let ix0 = a.x.max(b.x);
    let iy0 = a.y.max(b.y);
    let ix1 = ax1.min(bx1);
    let iy1 = ay1.min(by1);
    let inter = (ix1 - ix0).max(0.0) * (iy1 - iy0).max(0.0);
    let union = a.w * a.h + b.w * b.h - inter;
    if union <= f64::EPSILON {
        0.0
    } else {
        inter / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_norm_unrotated_matches_spec() {
        let b = LayoutBBoxPdf {
            x0: 61.2,
            y0: 396.0,
            x1: 306.0,
            y1: 594.0,
        };
        let n = bbox_norm_from_pdf(b, 612.0, 792.0, 0);
        assert!((n.x - 0.1).abs() < 1e-3);
        assert!((n.w - 0.4).abs() < 1e-3);
        assert!((n.y - 0.25).abs() < 1e-3);
        assert!((n.h - 0.25).abs() < 1e-3);
        let expected = LayoutBBoxNorm {
            x: 0.1,
            y: 0.25,
            w: 0.4,
            h: 0.25,
        };
        assert!(bbox_norm_iou(n, expected) >= 0.99);
    }

    fn letter_figure() -> LayoutBBoxPdf {
        LayoutBBoxPdf {
            x0: 61.2,
            y0: 396.0,
            x1: 306.0,
            y1: 594.0,
        }
    }

    #[test]
    fn bbox_norm_rotated_90_golden() {
        let n = bbox_norm_from_pdf(letter_figure(), 612.0, 792.0, 90);
        let expected = LayoutBBoxNorm {
            x: 0.5,
            y: 0.1,
            w: 0.25,
            h: 0.4,
        };
        assert!(
            bbox_norm_iou(n, expected) >= 0.99,
            "90° got {n:?} expected {expected:?}"
        );
    }

    #[test]
    fn bbox_norm_rotated_180_golden() {
        let n = bbox_norm_from_pdf(letter_figure(), 612.0, 792.0, 180);
        let expected = LayoutBBoxNorm {
            x: 0.5,
            y: 0.5,
            w: 0.4,
            h: 0.25,
        };
        assert!(
            bbox_norm_iou(n, expected) >= 0.99,
            "180° got {n:?} expected {expected:?}"
        );
    }

    #[test]
    fn bbox_norm_rotated_270_golden() {
        let n = bbox_norm_from_pdf(letter_figure(), 612.0, 792.0, 270);
        let expected = LayoutBBoxNorm {
            x: 0.25,
            y: 0.5,
            w: 0.25,
            h: 0.4,
        };
        assert!(
            bbox_norm_iou(n, expected) >= 0.99,
            "270° got {n:?} expected {expected:?}"
        );
    }

    #[test]
    fn bbox_norm_iou_partial_overlap() {
        let a = LayoutBBoxNorm {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        };
        let b = LayoutBBoxNorm {
            x: 0.5,
            y: 0.5,
            w: 0.5,
            h: 0.5,
        };
        assert!((bbox_norm_iou(a, b) - 0.25).abs() < 1e-6);
    }
}
