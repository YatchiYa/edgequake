//! Per-page layout sidecar (SPEC-128) — PDF user space SSOT, no SQL.
//!
//! Written under `{assets_root}/page_layout.json` during vision convert.
//! The API crate persists rows after `document_id` is known.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::embedded_images::WrittenFigureAsset;
use crate::figure_filter::{FigureFilterResult, FigureKind};
use crate::region_assets::WrittenTableAsset;

/// Sidecar filename under the document mm-assets root.
pub const PAGE_LAYOUT_SIDECAR: &str = "page_layout.json";

/// PDF user-space box `{x0,y0,x1,y1}` (min/max, y-up).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BBoxPdf {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl BBoxPdf {
    pub fn from_tuple(t: (f32, f32, f32, f32)) -> Self {
        Self {
            x0: t.0.min(t.2),
            y0: t.1.min(t.3),
            x1: t.0.max(t.2),
            y1: t.1.max(t.3),
        }
    }
}

/// One overlay region in the sidecar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayoutRegionSidecar {
    pub class: String,
    pub source: String,
    pub bbox_pdf: BBoxPdf,
    pub confidence: Option<f32>,
    pub reading_order: Option<i32>,
    pub asset_path: Option<String>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// One page in the sidecar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayoutPageSidecar {
    pub page_number: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub rotation: i16,
    pub cropbox_pdf: Option<BBoxPdf>,
    pub layout_model: Option<String>,
    pub layout_status: String,
    pub regions: Vec<PageLayoutRegionSidecar>,
}

/// Root sidecar document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageLayoutSidecar {
    pub pages: Vec<PageLayoutPageSidecar>,
}

pub fn sidecar_path(assets_root: &Path) -> std::path::PathBuf {
    assets_root.join(PAGE_LAYOUT_SIDECAR)
}

pub fn sidecar_exists(assets_root: &Path) -> bool {
    sidecar_path(assets_root).is_file()
}

pub fn load_page_layout_sidecar(assets_root: &Path) -> Option<PageLayoutSidecar> {
    let path = sidecar_path(assets_root);
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn overlay_class_from_figure_kind(kind: &FigureKind, is_figure: bool) -> &'static str {
    if !is_figure {
        return "abandon";
    }
    match kind {
        FigureKind::BarChart
        | FigureKind::LineChart
        | FigureKind::ScatterPlot
        | FigureKind::Heatmap
        | FigureKind::Histogram
        | FigureKind::PieChart
        | FigureKind::RadarChart => "chart",
        FigureKind::TableVisual => "table",
        _ => "figure",
    }
}

/// Write sidecar from already-extracted figure/table assets (persist before bbox drop).
pub fn write_sidecar_from_assets(
    assets_root: &Path,
    pdf_bytes: &[u8],
    figure_map: &HashMap<usize, Vec<WrittenFigureAsset>>,
    table_map: &HashMap<usize, Vec<WrittenTableAsset>>,
    filter_results: Option<&[FigureFilterResult]>,
) {
    let media =
        edgequake_pdf2md::extract_page_media_boxes_from_bytes(pdf_bytes, None).unwrap_or_default();
    let mut pages_by_num: HashMap<usize, PageLayoutPageSidecar> = HashMap::new();
    for box_info in &media {
        pages_by_num.insert(
            box_info.page_num,
            PageLayoutPageSidecar {
                page_number: box_info.page_num,
                width_pt: box_info.width_pt,
                height_pt: box_info.height_pt,
                rotation: box_info.rotation,
                cropbox_pdf: box_info.cropbox.map(BBoxPdf::from_tuple),
                layout_model: Some("l0-l1".into()),
                layout_status: "extracted".into(),
                regions: Vec::new(),
            },
        );
    }

    let filter_by_path: HashMap<&str, &FigureFilterResult> = filter_results
        .unwrap_or(&[])
        .iter()
        .map(|r| (r.rel_path.as_str(), r))
        .collect();

    for (page, figs) in figure_map {
        let page_entry = pages_by_num
            .entry(*page)
            .or_insert_with(|| empty_page(*page));
        for fig in figs {
            let Some(bb) = fig.bbox else {
                continue;
            };
            let (class, extra) = if let Some(fr) = filter_by_path.get(fig.rel_path.as_str()) {
                let class = overlay_class_from_figure_kind(&fr.kind, fr.is_figure);
                (
                    class.to_string(),
                    serde_json::json!({ "figure_kind": fr.kind, "is_figure": fr.is_figure }),
                )
            } else {
                (
                    "figure".to_string(),
                    serde_json::json!({ "asset": "embedded_or_region" }),
                )
            };
            page_entry.regions.push(PageLayoutRegionSidecar {
                class,
                source: "l1_paint".into(),
                bbox_pdf: BBoxPdf::from_tuple(bb),
                confidence: None,
                reading_order: Some(fig.index as i32),
                asset_path: Some(fig.rel_path.clone()),
                extra,
            });
        }
    }

    for (page, tables) in table_map {
        let page_entry = pages_by_num
            .entry(*page)
            .or_insert_with(|| empty_page(*page));
        for t in tables {
            let Some(bb) = t.bbox else {
                continue;
            };
            page_entry.regions.push(PageLayoutRegionSidecar {
                class: "table".into(),
                source: "l1_paint".into(),
                bbox_pdf: BBoxPdf::from_tuple(bb),
                confidence: None,
                reading_order: Some(t.index as i32),
                asset_path: Some(t.rel_path.clone()),
                extra: serde_json::json!({ "label": t.label }),
            });
        }
    }

    if let Ok(text_regions) = edgequake_pdf2md::extract_text_layout_from_bytes(pdf_bytes, None) {
        for tr in text_regions {
            let page_entry = pages_by_num
                .entry(tr.page_num)
                .or_insert_with(|| empty_page(tr.page_num));
            page_entry.regions.push(PageLayoutRegionSidecar {
                class: tr.class.into(),
                source: tr.source.into(),
                bbox_pdf: BBoxPdf::from_tuple(tr.bbox),
                confidence: None,
                reading_order: Some(tr.reading_order),
                asset_path: None,
                extra: serde_json::json!({}),
            });
        }
    }

    let mut pages: Vec<PageLayoutPageSidecar> = pages_by_num.into_values().collect();
    pages.sort_by_key(|p| p.page_number);
    let sidecar = PageLayoutSidecar { pages };
    let path = sidecar_path(assets_root);
    if let Err(e) = std::fs::create_dir_all(assets_root) {
        warn!(path = %assets_root.display(), error = %e, "failed to create page_layout sidecar dir");
        return;
    }
    match serde_json::to_string_pretty(&sidecar) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                warn!(path = %path.display(), error = %e, "failed to write page_layout sidecar");
            } else {
                debug!(path = %path.display(), "wrote SPEC-128 page_layout sidecar");
            }
        }
        Err(e) => warn!(error = %e, "failed to serialize page_layout sidecar"),
    }
}

fn empty_page(page_number: usize) -> PageLayoutPageSidecar {
    PageLayoutPageSidecar {
        page_number,
        width_pt: 612.0,
        height_pt: 792.0,
        rotation: 0,
        cropbox_pdf: None,
        layout_model: Some("l0-l1".into()),
        layout_status: "extracted".into(),
        regions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_class_maps_charts_and_noise() {
        assert_eq!(
            overlay_class_from_figure_kind(&FigureKind::BarChart, true),
            "chart"
        );
        assert_eq!(
            overlay_class_from_figure_kind(&FigureKind::Logo, false),
            "abandon"
        );
        assert_eq!(
            overlay_class_from_figure_kind(&FigureKind::Stamp, false),
            "abandon"
        );
        assert_eq!(
            overlay_class_from_figure_kind(&FigureKind::Diagram, true),
            "figure"
        );
    }

    fn minimal_pdf_bytes() -> Vec<u8> {
        br#"%PDF-1.4
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>endobj
xref
0 4
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
trailer<< /Size 4 /Root 1 0 R >>
startxref
190
%%EOF"#
            .to_vec()
    }

    #[test]
    fn sidecar_keeps_abandon_when_filter_discards() {
        use crate::figure_filter::{FigureFilterResult, FigureKind};
        use crate::WrittenFigureAsset;
        use std::collections::HashMap;

        let dir = tempfile::tempdir().unwrap();
        let mut map = HashMap::new();
        map.insert(
            1usize,
            vec![WrittenFigureAsset {
                page_num: 1,
                index: 1,
                rel_path: "assets/logo.png".into(),
                width: 10,
                height: 10,
                bbox: Some((10.0, 700.0, 80.0, 770.0)),
            }],
        );
        let results = vec![FigureFilterResult {
            rel_path: "assets/logo.png".into(),
            page_num: 1,
            label: String::new(),
            kind: FigureKind::Logo,
            is_figure: false,
            description: String::new(),
        }];
        write_sidecar_from_assets(
            dir.path(),
            &minimal_pdf_bytes(),
            &map,
            &HashMap::new(),
            Some(&results),
        );
        let sidecar = load_page_layout_sidecar(dir.path()).expect("sidecar");
        let classes: Vec<_> = sidecar
            .pages
            .iter()
            .flat_map(|p| p.regions.iter().map(|r| r.class.as_str()))
            .collect();
        assert!(
            classes.contains(&"abandon"),
            "LAW-128-2: discarded crops remain on overlay as abandon, got {classes:?}"
        );
    }
}
