//! Page signal extraction for modality classification (SPEC-134).
//!
//! First principles: a scanned manuscript page is **image-primary** — meaning
//! lives in pixels; the glyph layer is absent or an OCR byproduct that often
//! lies. A born-digital page is **glyph-primary**. These signals measure that
//! split from the PDF structure itself (no VLM, no page raster, no decode):
//!
//! ```text
//!  PDF bytes
//!    └─ lopdf load (edgeparse-core loader)
//!         ├─ page_info         → page dims + orientation mix (all pages)
//!         ├─ page_walk         → image_area_frac + glyph_text_density
//!         │                      (one content-stream pass, ≤ 32 pages)
//!         └─ extract_image_data→ ink_frac (dominant image, ambiguous band)
//!                  │
//!                  ▼
//!  classify_page_heuristic → PageModality per page → document majority
//! ```
//!
//! Performance: manuscript scan (4 pages) ≈ 60 ms; dense born-digital
//! (1196 pages) ≈ 0.7 s with the 32-page sample. The decode-everything path
//! cost 41.5 s on the same 4-page scan (~700× slower).
//!
//! DRY: content-stream walk reuses [`crate::page_walk`]; ink reuses
//! [`crate::chart_crop::ink_fraction_from_bytes`].

use edgeparse_core::pdf::image_extractor::{extract_image_chunks, extract_image_data};

use crate::chart_crop::ink_fraction_from_bytes;
use crate::error::PdfConversionError;
use crate::page_modality::{
    classify_document_majority, classify_page_heuristic, PageClassification, PageModality,
};
use crate::page_walk::walk_page_signals;

/// Raw modality signals for one page (all fractions in 0.0–1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSignals {
    /// 1-indexed page number.
    pub page_num: usize,
    /// Fraction of page area covered by embedded image placements.
    pub image_area_frac: f32,
    /// Normalized glyph density: text char count / [`CHARS_PER_DENSE_PAGE`],
    /// clamped to 1.0.
    pub glyph_text_density: f32,
    /// Dark-pixel fraction of the dominant embedded image; 0.0 when the page
    /// is not in the ambiguous band or the image is undecodable (CCITT/JPX).
    pub ink_frac: f32,
}

/// Char count that maps to density 1.0. Calibration (real documents):
/// born-digital dense text ≈ 2200–4700 chars/page; scanner OCR byproduct on
/// handwriting ≈ 100–1600 chars/page.
const CHARS_PER_DENSE_PAGE: usize = 1600;

/// Max pages walked for classification — bounds worst-case cost on huge
/// documents (~21 ms/page on vector-heavy pages) while keeping the majority
/// vote representative.
const MAX_SAMPLE_PAGES: usize = 32;

/// Max ink probes per document (each decodes one image).
const MAX_INK_PROBES: usize = 4;

/// Compute per-page modality signals from PDF bytes (blocking, no raster).
///
/// Walks at most [`MAX_SAMPLE_PAGES`] pages (evenly spread, endpoints
/// included) — a document's modality is evident without parsing every page.
///
/// Note: `lopdf` load cost is proportional to document size (≈24 ms for a
/// 4-page scan, ≈11 s for a 1196-page legal text) — one-time per document,
/// negligible next to VLM conversion. Use [`analyze_modality_blocking`] when
/// the orientation mix is also needed, to avoid a second parse.
pub fn compute_page_signals_blocking(
    pdf_bytes: &[u8],
) -> Result<Vec<PageSignals>, PdfConversionError> {
    Ok(analyze_modality_blocking(pdf_bytes)?.pages)
}

/// Document-level modality analysis from a **single** lopdf parse.
#[derive(Debug, Clone)]
pub struct ModalityAnalysis {
    pub pages: Vec<PageSignals>,
    pub orientation_mixed: bool,
}

/// Single-parse entry point: page signals + orientation mix together.
pub fn analyze_modality_blocking(pdf_bytes: &[u8]) -> Result<ModalityAnalysis, PdfConversionError> {
    let raw = edgeparse_core::pdf::loader::load_pdf_from_bytes(pdf_bytes, None)
        .map_err(|e| PdfConversionError::Backend(format!("lopdf load: {e}")))?;
    let infos = edgeparse_core::pdf::page_info::extract_page_info(&raw.document);

    let mut portrait = false;
    let mut landscape = false;
    for info in &infos {
        let (w, h) = effective_page_dims(info.width, info.height, info.rotation);
        if w > h {
            landscape = true;
        } else {
            portrait = true;
        }
    }

    let sample = sample_page_numbers(infos.len(), MAX_SAMPLE_PAGES);
    let walked = walk_page_signals(&raw.document, &sample);

    let pages_map = raw.document.get_pages();
    let mut ink_budget = MAX_INK_PROBES;
    let mut pages = Vec::with_capacity(walked.len());

    for w in &walked {
        let Some(info) = infos.iter().find(|i| i.page_number == w.page_num as u32) else {
            continue;
        };
        let (pw, ph) = effective_page_dims(info.width, info.height, info.rotation);

        let image_area_frac = union_frac(w.image_bboxes.clone(), pw, ph);
        let glyph_text_density =
            (w.text_chars as f32 / CHARS_PER_DENSE_PAGE as f32).clamp(0.0, 1.0);

        // Ink probe only in the ambiguous band: image-dominant page whose
        // dense text layer could be born-digital OR an OCR byproduct. Rule 3
        // (ink > 0.02) is the decider there; elsewhere ink is unused.
        let mut ink_frac = 0.0;
        if ink_budget > 0 && image_area_frac > 0.5 && glyph_text_density >= 0.5 {
            ink_budget -= 1;
            if let Some(&page_id) = pages_map.get(&(w.page_num as u32)) {
                ink_frac = probe_dominant_image_ink(&raw.document, page_id, w.page_num as u32);
            }
        }

        pages.push(PageSignals {
            page_num: w.page_num,
            image_area_frac,
            glyph_text_density,
            ink_frac,
        });
    }
    Ok(ModalityAnalysis {
        pages,
        orientation_mixed: portrait && landscape,
    })
}

/// Async wrapper — structural parsing is CPU-bound, so hop to the blocking pool.
pub async fn compute_page_signals(
    pdf_bytes: &[u8],
) -> Result<Vec<PageSignals>, PdfConversionError> {
    let bytes = pdf_bytes.to_vec();
    tokio::task::spawn_blocking(move || compute_page_signals_blocking(&bytes))
        .await
        .map_err(|e| PdfConversionError::Backend(format!("page signals task panicked: {e}")))?
}

/// True when the document mixes portrait and landscape pages (soft prior).
///
/// Standalone variant that parses the document — prefer
/// [`analyze_modality_blocking`] when page signals are also needed, to avoid
/// paying the lopdf load twice.
pub fn orientation_mixed(pdf_bytes: &[u8]) -> bool {
    let Ok(raw) = edgeparse_core::pdf::loader::load_pdf_from_bytes(pdf_bytes, None) else {
        return false;
    };
    let mut portrait = false;
    let mut landscape = false;
    for info in edgeparse_core::pdf::page_info::extract_page_info(&raw.document) {
        let (w, h) = effective_page_dims(info.width, info.height, info.rotation);
        if w > h {
            landscape = true;
        } else {
            portrait = true;
        }
    }
    portrait && landscape
}

/// Classify a whole document from its page signals (majority gate, LAW-134).
pub fn classify_document_from_signals(
    signals: &[PageSignals],
    orientation_mixed: bool,
) -> PageModality {
    let pages: Vec<_> = signals
        .iter()
        .map(|s| {
            classify_page_heuristic(
                s.image_area_frac,
                s.glyph_text_density,
                s.ink_frac,
                orientation_mixed,
            )
        })
        .collect();
    classify_document_majority(&pages)
}

/// One-shot convenience: bytes → signals → document modality.
///
/// Fail-open to [`PageModality::Print`] on any extraction error — a
/// classification failure must never block ingestion (existing behavior is
/// the print path).
pub async fn classify_document_from_bytes(pdf_bytes: &[u8]) -> PageModality {
    let bytes = pdf_bytes.to_vec();
    let result = tokio::task::spawn_blocking(move || analyze_modality_blocking(&bytes))
        .await
        .map_err(|e| PdfConversionError::Backend(format!("modality task panicked: {e}")))
        .and_then(|r| r);
    match result {
        Ok(analysis) => classify_document_from_signals(&analysis.pages, analysis.orientation_mixed),
        Err(e) => {
            tracing::warn!(error = %e, "SPEC-134 page signal extraction failed; defaulting to Print");
            PageModality::Print
        }
    }
}

/// Per-page classifications (sampled pages only). Fail-open: empty vec.
pub async fn classify_pages_from_bytes(pdf_bytes: &[u8]) -> Vec<PageClassification> {
    let bytes = pdf_bytes.to_vec();
    let result = tokio::task::spawn_blocking(move || analyze_modality_blocking(&bytes))
        .await
        .map_err(|e| PdfConversionError::Backend(format!("modality task panicked: {e}")))
        .and_then(|r| r);
    match result {
        Ok(analysis) => analysis
            .pages
            .iter()
            .map(|s| PageClassification {
                page_num: s.page_num,
                result: classify_page_heuristic(
                    s.image_area_frac,
                    s.glyph_text_density,
                    s.ink_frac,
                    analysis.orientation_mixed,
                ),
            })
            .collect(),
        Err(e) => {
            tracing::warn!(error = %e, "SPEC-134 page signal extraction failed; no per-page classes");
            Vec::new()
        }
    }
}

/// Evenly-spaced 1-indexed page sample of at most `cap` pages, endpoints included.
fn sample_page_numbers(page_count: usize, cap: usize) -> Vec<usize> {
    if page_count <= cap {
        return (1..=page_count).collect();
    }
    let step = (page_count - 1) as f64 / (cap - 1) as f64;
    (0..cap)
        .map(|i| 1 + (i as f64 * step).round() as usize)
        .collect()
}

/// Rotation-adjusted page dimensions in points.
fn effective_page_dims(width: f64, height: f64, rotation: i64) -> (f64, f64) {
    if rotation == 90 || rotation == 270 {
        (height.abs(), width.abs())
    } else {
        (width.abs(), height.abs())
    }
}

/// Union of rect coverage as a fraction of the page, via a 64×64 occupancy
/// grid. Robust to the overlapping tiled strips scanners emit (a naive sum
/// overcounts 17× on such pages); ±1.5 % accuracy is ample for a heuristic.
fn union_frac(rects: Vec<(f64, f64, f64, f64)>, pw: f64, ph: f64) -> f32 {
    if rects.is_empty() || pw <= 0.0 || ph <= 0.0 {
        return 0.0;
    }
    const G: usize = 64;
    let mut grid = [false; G * G];
    for (x0, y0, x1, y1) in rects {
        let cx0 = ((x0 / pw).clamp(0.0, 1.0) * G as f64) as usize;
        let cx1 = (((x1 / pw).clamp(0.0, 1.0) * G as f64).ceil() as usize).min(G);
        let cy0 = ((y0 / ph).clamp(0.0, 1.0) * G as f64) as usize;
        let cy1 = (((y1 / ph).clamp(0.0, 1.0) * G as f64).ceil() as usize).min(G);
        for cy in cy0..cy1 {
            for cx in cx0..cx1 {
                grid[cy * G + cx] = true;
            }
        }
    }
    grid.iter().filter(|&&c| c).count() as f32 / (G * G) as f32
}

/// Dark-pixel fraction of the page's dominant image, best-effort.
///
/// `extract_image_chunks` and `extract_image_data` enumerate the XObject
/// dictionary in the same order, so chunk indices are stable lookup keys.
/// Returns 0.0 when undecodable (CCITT fax / JPX) — the classifier treats
/// unknown ink as absent and falls back to coverage rules.
fn probe_dominant_image_ink(doc: &lopdf::Document, page_id: lopdf::ObjectId, page_num: u32) -> f32 {
    let meta = extract_image_chunks(doc, page_num, page_id).unwrap_or_default();
    let dominant = meta
        .iter()
        .filter_map(|c| {
            c.index.map(|i| {
                (
                    i,
                    (c.bbox.right_x - c.bbox.left_x) * (c.bbox.top_y - c.bbox.bottom_y),
                )
            })
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let Some((idx, _)) = dominant else {
        return 0.0;
    };
    let Ok(Some(img)) = extract_image_data(doc, page_id, idx) else {
        return 0.0;
    };
    ink_fraction_from_bytes(&img.data).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_from_signals_pure_scan_is_manuscript() {
        let signals = vec![PageSignals {
            page_num: 1,
            image_area_frac: 0.95,
            glyph_text_density: 0.0,
            ink_frac: 0.06,
        }];
        assert_eq!(
            classify_document_from_signals(&signals, false),
            PageModality::Manuscript
        );
    }

    #[test]
    fn classify_from_signals_ocr_layer_scan_still_manuscript() {
        // EC-10: scanner OCR layer produces dense text coverage, but full-page
        // image coverage dominates (rule 1).
        let signals = vec![PageSignals {
            page_num: 1,
            image_area_frac: 1.0,
            glyph_text_density: 1.0,
            ink_frac: 0.0,
        }];
        assert_eq!(
            classify_document_from_signals(&signals, false),
            PageModality::Manuscript
        );
    }

    #[test]
    fn classify_from_signals_cropped_scan_sparse_text() {
        let signals = vec![PageSignals {
            page_num: 1,
            image_area_frac: 0.6,
            glyph_text_density: 0.3,
            ink_frac: 0.0,
        }];
        assert_eq!(
            classify_document_from_signals(&signals, false),
            PageModality::Manuscript
        );
    }

    #[test]
    fn classify_from_signals_born_digital_is_print() {
        let signals = vec![PageSignals {
            page_num: 1,
            image_area_frac: 0.1,
            glyph_text_density: 0.9,
            ink_frac: 0.0,
        }];
        assert_eq!(
            classify_document_from_signals(&signals, false),
            PageModality::Print
        );
    }

    #[test]
    fn classify_from_signals_mixed_document() {
        let signals = vec![
            PageSignals {
                page_num: 1,
                image_area_frac: 0.9,
                glyph_text_density: 0.05,
                ink_frac: 0.04,
            },
            PageSignals {
                page_num: 2,
                image_area_frac: 0.1,
                glyph_text_density: 0.8,
                ink_frac: 0.0,
            },
            PageSignals {
                page_num: 3,
                image_area_frac: 0.1,
                glyph_text_density: 0.85,
                ink_frac: 0.0,
            },
        ];
        assert_eq!(
            classify_document_from_signals(&signals, false),
            PageModality::Mixed
        );
    }

    #[test]
    fn empty_signals_is_print() {
        assert_eq!(
            classify_document_from_signals(&[], false),
            PageModality::Print
        );
    }

    #[test]
    fn sample_spreads_endpoints_included() {
        assert_eq!(sample_page_numbers(4, 8), vec![1, 2, 3, 4]);
        let s = sample_page_numbers(100, 8);
        assert_eq!(s.len(), 8);
        assert_eq!(s[0], 1);
        assert_eq!(s[7], 100);
    }

    #[test]
    fn union_frac_non_overlapping() {
        // Two quarter-page rects at opposite corners → 0.5.
        let rects = vec![(0.0, 0.0, 50.0, 50.0), (50.0, 50.0, 100.0, 100.0)];
        let f = union_frac(rects, 100.0, 100.0);
        assert!((f - 0.5).abs() < 0.05, "got {f}");
    }

    #[test]
    fn union_frac_overlap_not_double_counted() {
        // Same rect twice → still its own area, not 2×.
        let rects = vec![(0.0, 0.0, 50.0, 100.0), (0.0, 0.0, 50.0, 100.0)];
        let f = union_frac(rects, 100.0, 100.0);
        assert!((f - 0.5).abs() < 0.05, "got {f}");
    }

    #[test]
    fn union_frac_full_coverage() {
        let f = union_frac(vec![(0.0, 0.0, 100.0, 100.0)], 100.0, 100.0);
        assert_eq!(f, 1.0);
    }
}
