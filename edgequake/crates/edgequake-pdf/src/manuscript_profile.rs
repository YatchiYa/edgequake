//! Manuscript render profile for Vision Pass-A (SPEC-134).
//!
//! First principles: handwritten / MFD-scanned pages need higher resolution than
//! print-optimized adaptive DPI. Thin ink, tick marks, and color series blur at
//! 96–150 DPI. This profile couples DPI floor with `max_rendered_pixels` floor
//! so effective resolution is not silently capped.
//!
//! DRY: single `ManuscriptProfile::resolve` — no ad-hoc DPI math elsewhere.

use crate::page_modality::PageModality;

/// Default DPI floor for manuscript pages (SOTA HTR guidance: ≥300 DPI).
pub const DEFAULT_MANUSCRIPT_DPI: u32 = 300;

/// Default max_rendered_pixels floor for manuscript pages.
/// A4 @ 300 DPI ≈ 2480×3508; floor 3600 covers long edge.
pub const DEFAULT_MANUSCRIPT_MAX_PIXELS: u32 = 3600;

/// Manuscript render profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManuscriptProfile {
    /// DPI to use for pdf2md convert (Pass-A OCR).
    pub dpi: u32,
    /// Max rendered pixels for page PNG assets (viewer + Pass-B).
    pub max_rendered_pixels: u32,
    /// Whether to block EdgeParse Auto fast-path (LAW-134-12).
    pub skip_edgeparse_fastpath: bool,
}

impl Default for ManuscriptProfile {
    fn default() -> Self {
        Self {
            dpi: DEFAULT_MANUSCRIPT_DPI,
            max_rendered_pixels: DEFAULT_MANUSCRIPT_MAX_PIXELS,
            skip_edgeparse_fastpath: true,
        }
    }
}

impl ManuscriptProfile {
    /// Resolve profile for a given modality.
    ///
    /// - `adaptive_dpi`: the DPI computed by `compute_safe_pdf_resource_profile`
    ///   (96–150 based on page count / file size).
    /// - Env overrides: `EDGEQUAKE_PDF_MANUSCRIPT_DPI`, `EDGEQUAKE_PDF_MANUSCRIPT_MAX_PIXELS`,
    ///   `EDGEQUAKE_PDF_MANUSCRIPT_SKIP_EDGEPARSE`.
    ///
    /// For Print modality, returns profile with `dpi = adaptive_dpi` (no floor).
    /// For Manuscript/Mixed, applies floors.
    pub fn resolve(modality: PageModality, adaptive_dpi: u32) -> Self {
        if !modality.is_manuscript_like() {
            return Self {
                dpi: adaptive_dpi,
                max_rendered_pixels: 2000, // existing default
                skip_edgeparse_fastpath: false,
            };
        }

        let dpi = std::env::var("EDGEQUAKE_PDF_MANUSCRIPT_DPI")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .map(|d| d.clamp(200, 400))
            .unwrap_or(DEFAULT_MANUSCRIPT_DPI)
            .max(adaptive_dpi); // never go below adaptive

        let max_pixels = std::env::var("EDGEQUAKE_PDF_MANUSCRIPT_MAX_PIXELS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MANUSCRIPT_MAX_PIXELS)
            .max(2000); // never below existing default

        let skip_edgeparse = std::env::var("EDGEQUAKE_PDF_MANUSCRIPT_SKIP_EDGEPARSE")
            .ok()
            .map(|v| !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no"))
            .unwrap_or(true);

        Self {
            dpi,
            max_rendered_pixels: max_pixels,
            skip_edgeparse_fastpath: skip_edgeparse,
        }
    }

    /// Whether this profile should bypass the adaptive DPI clamp in pdf_processing.
    ///
    /// The existing `.clamp(96, safe_dpi.max(96))` caps DPI at the adaptive profile.
    /// For manuscript, we need to exceed that cap.
    pub fn bypasses_adaptive_clamp(&self, modality: PageModality) -> bool {
        modality.is_manuscript_like() && self.dpi > 150
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_modality_uses_adaptive_dpi() {
        let p = ManuscriptProfile::resolve(PageModality::Print, 150);
        assert_eq!(p.dpi, 150);
        assert_eq!(p.max_rendered_pixels, 2000);
        assert!(!p.skip_edgeparse_fastpath);
    }

    #[test]
    fn manuscript_modality_applies_floors() {
        let p = ManuscriptProfile::resolve(PageModality::Manuscript, 96);
        assert_eq!(p.dpi, 300);
        assert_eq!(p.max_rendered_pixels, 3600);
        assert!(p.skip_edgeparse_fastpath);
    }

    #[test]
    fn mixed_modality_applies_floors() {
        let p = ManuscriptProfile::resolve(PageModality::Mixed, 120);
        assert_eq!(p.dpi, 300);
        assert!(p.skip_edgeparse_fastpath);
    }

    #[test]
    fn manuscript_never_below_adaptive() {
        // If adaptive somehow computed 400 (future), MS floor doesn't lower it
        let p = ManuscriptProfile::resolve(PageModality::Manuscript, 400);
        assert_eq!(p.dpi, 400);
    }

    #[test]
    fn bypasses_clamp_only_for_ms_above_150() {
        let ms = ManuscriptProfile::resolve(PageModality::Manuscript, 96);
        assert!(ms.bypasses_adaptive_clamp(PageModality::Manuscript));

        let print = ManuscriptProfile::resolve(PageModality::Print, 150);
        assert!(!print.bypasses_adaptive_clamp(PageModality::Print));
    }

    #[test]
    fn default_profile_is_manuscript_ready() {
        let p = ManuscriptProfile::default();
        assert_eq!(p.dpi, 300);
        assert_eq!(p.max_rendered_pixels, 3600);
        assert!(p.skip_edgeparse_fastpath);
    }
}
