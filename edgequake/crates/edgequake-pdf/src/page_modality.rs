//! Page modality classification for PDF pages (SPEC-134).
//!
//! First principles: a handwritten / MFD-scanned page is **not** a born-digital
//! PDF with a trustworthy glyph stream. Meaning lives in pixels. This module
//! classifies pages into modalities so downstream render / prompt / asset policy
//! can route correctly.
//!
//! DRY: single `PageModality` enum; single heuristic classifier; no VLM calls here.

use serde::{Deserialize, Serialize};

/// Page modality for vision pipeline routing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageModality {
    /// Born-digital or clean print scan — default Pass-A print prompt.
    #[default]
    Print,
    /// Handwritten / MFD-scanned technical page — manuscript profile.
    Manuscript,
    /// Mixed print + handwriting — apply manuscript render floor + prompt.
    Mixed,
}

impl PageModality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Print => "print",
            Self::Manuscript => "manuscript",
            Self::Mixed => "mixed",
        }
    }

    pub fn from_env_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "print" => Some(Self::Print),
            "manuscript" | "ms" | "handwritten" => Some(Self::Manuscript),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }

    /// Force override from env for tests / operator control.
    pub fn from_env() -> Option<Self> {
        std::env::var("EDGEQUAKE_PDF_PAGE_MODALITY")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .and_then(|v| Self::from_env_str(&v))
    }

    /// Whether manuscript profile should apply.
    pub fn is_manuscript_like(self) -> bool {
        matches!(self, Self::Manuscript | Self::Mixed)
    }
}

/// Result of heuristic page classification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageClassResult {
    pub modality: PageModality,
    /// Confidence 0.0–1.0 (heuristic, not calibrated).
    pub score: f32,
}

/// One classified page (1-indexed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageClassification {
    pub page_num: usize,
    pub result: PageClassResult,
}

impl PageClassification {
    pub fn modality(self) -> PageModality {
        self.result.modality
    }
}

/// Heuristic page classifier (deterministic, no VLM).
///
/// Signals:
/// - `image_area_frac`: fraction of page area covered by image XObjects
///   (union of CTM placement bboxes — tiled scanner strips sum to ~1.0).
/// - `glyph_text_density`: extractable text coverage, normalized 0–1.
/// - `ink_frac`: fraction of dark pixels in the dominant image (proxy for
///   handwriting; 0.0 when the image is undecodable, e.g. CCITT/JPX).
/// - `orientation_mixed`: document has mixed portrait/landscape pages.
///
/// Rules (first principles: **image dominance is the primary fact** — on a
/// scanned page, any glyph layer is an OCR byproduct hint that often lies):
/// - image_area_frac ≥ 0.85 → Manuscript (full-page scan; EC-10 OCR layer
///   may be dense — coverage wins over text)
/// - image_area_frac > 0.7 AND glyph_text_density < 0.1 → Manuscript
/// - image_area_frac > 0.5 AND ink_frac > 0.02 → Manuscript (ink-confirmed)
/// - image_area_frac > 0.5 AND glyph_text_density < 0.5 → Manuscript
///   (cropped scan with sparse OCR)
/// - image_area_frac > 0.3 AND glyph_text_density < 0.3 → Mixed
/// - orientation_mixed AND image_area_frac > 0.4 → Mixed (soft prior)
/// - else → Print
///
/// Known limit: a cropped scan (image 0.5–0.85) with a dense OCR layer
/// (density ≥ 0.5) and an undecodable image falls to Print — escape hatch is
/// the `EDGEQUAKE_PDF_PAGE_MODALITY` override.
pub fn classify_page_heuristic(
    image_area_frac: f32,
    glyph_text_density: f32,
    ink_frac: f32,
    orientation_mixed: bool,
) -> PageClassResult {
    // Force override wins
    if let Some(forced) = PageModality::from_env() {
        return PageClassResult {
            modality: forced,
            score: 1.0,
        };
    }

    let (modality, score) =
        if image_area_frac >= 0.85 || (image_area_frac > 0.7 && glyph_text_density < 0.1) {
            // Full-page scan (any text layer is an OCR byproduct — EC-10: coverage
            // wins over text) or strongly image-dominant page with negligible text.
            (PageModality::Manuscript, 0.9)
        } else if image_area_frac > 0.5 && ink_frac > 0.02 {
            (PageModality::Manuscript, 0.7)
        } else if image_area_frac > 0.5 && glyph_text_density < 0.5 {
            (PageModality::Manuscript, 0.55)
        } else if image_area_frac > 0.3 && glyph_text_density < 0.3 {
            (PageModality::Mixed, 0.6)
        } else if orientation_mixed && image_area_frac > 0.4 {
            (PageModality::Mixed, 0.5)
        } else {
            (PageModality::Print, 0.8)
        };

    PageClassResult { modality, score }
}

/// Document-level majority gate (v1 simplification).
///
/// If ≥50% of pages are manuscript-like, classify whole document as Manuscript.
/// This avoids per-page DPI switches inside pdf2md (which would double render).
pub fn classify_document_majority(pages: &[PageClassResult]) -> PageModality {
    if pages.is_empty() {
        return PageModality::Print;
    }

    // Force override wins
    if let Some(forced) = PageModality::from_env() {
        return forced;
    }

    let ms_count = pages
        .iter()
        .filter(|p| p.modality.is_manuscript_like())
        .count();
    let ms_frac = ms_count as f32 / pages.len() as f32;

    if ms_frac >= 0.5 {
        PageModality::Manuscript
    } else if ms_frac > 0.0 {
        PageModality::Mixed
    } else {
        PageModality::Print
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_page_classified_print() {
        let r = classify_page_heuristic(0.1, 0.8, 0.005, false);
        assert_eq!(r.modality, PageModality::Print);
    }

    #[test]
    fn image_primary_low_text_is_manuscript() {
        let r = classify_page_heuristic(0.85, 0.05, 0.03, false);
        assert_eq!(r.modality, PageModality::Manuscript);
        assert!(r.score > 0.8);
    }

    #[test]
    fn mid_image_mid_ink_is_manuscript() {
        let r = classify_page_heuristic(0.6, 0.2, 0.04, false);
        assert_eq!(r.modality, PageModality::Manuscript);
    }

    #[test]
    fn low_image_low_text_is_mixed() {
        let r = classify_page_heuristic(0.4, 0.2, 0.01, false);
        assert_eq!(r.modality, PageModality::Mixed);
    }

    #[test]
    fn orientation_mixed_soft_prior() {
        let r = classify_page_heuristic(0.45, 0.5, 0.01, true);
        assert_eq!(r.modality, PageModality::Mixed);
    }

    #[test]
    fn document_majority_manuscript() {
        let pages = vec![
            PageClassResult {
                modality: PageModality::Manuscript,
                score: 0.9,
            },
            PageClassResult {
                modality: PageModality::Manuscript,
                score: 0.8,
            },
            PageClassResult {
                modality: PageModality::Print,
                score: 0.9,
            },
        ];
        assert_eq!(classify_document_majority(&pages), PageModality::Manuscript);
    }

    #[test]
    fn document_majority_mixed() {
        let pages = vec![
            PageClassResult {
                modality: PageModality::Manuscript,
                score: 0.9,
            },
            PageClassResult {
                modality: PageModality::Print,
                score: 0.9,
            },
            PageClassResult {
                modality: PageModality::Print,
                score: 0.9,
            },
        ];
        assert_eq!(classify_document_majority(&pages), PageModality::Mixed);
    }

    #[test]
    fn document_all_print() {
        let pages = vec![
            PageClassResult {
                modality: PageModality::Print,
                score: 0.9,
            },
            PageClassResult {
                modality: PageModality::Print,
                score: 0.9,
            },
        ];
        assert_eq!(classify_document_majority(&pages), PageModality::Print);
    }

    #[test]
    fn empty_document_is_print() {
        assert_eq!(classify_document_majority(&[]), PageModality::Print);
    }

    #[test]
    fn modality_str_roundtrip() {
        assert_eq!(PageModality::Print.as_str(), "print");
        assert_eq!(PageModality::Manuscript.as_str(), "manuscript");
        assert_eq!(PageModality::Mixed.as_str(), "mixed");
        assert_eq!(
            PageModality::from_env_str("print"),
            Some(PageModality::Print)
        );
        assert_eq!(
            PageModality::from_env_str("manuscript"),
            Some(PageModality::Manuscript)
        );
        assert_eq!(
            PageModality::from_env_str("mixed"),
            Some(PageModality::Mixed)
        );
        assert_eq!(PageModality::from_env_str("invalid"), None);
    }

    #[test]
    fn is_manuscript_like() {
        assert!(!PageModality::Print.is_manuscript_like());
        assert!(PageModality::Manuscript.is_manuscript_like());
        assert!(PageModality::Mixed.is_manuscript_like());
    }
}
