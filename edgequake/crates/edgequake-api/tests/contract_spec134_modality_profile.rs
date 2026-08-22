//! SPEC-134 contract: ManuscriptProfile resolution and DPI clamp bypass.

use edgequake_pdf::{ManuscriptProfile, PageModality};

#[test]
fn manuscript_modality_applies_dpi_floor() {
    let profile = ManuscriptProfile::resolve(PageModality::Manuscript, 96);
    assert!(
        profile.dpi >= 300,
        "Manuscript DPI must be ≥300, got {}",
        profile.dpi
    );
    assert!(
        profile.max_rendered_pixels >= 3600,
        "Manuscript max_rendered_pixels must be ≥3600, got {}",
        profile.max_rendered_pixels
    );
    assert!(
        profile.skip_edgeparse_fastpath,
        "Manuscript must skip EdgeParse fast-path"
    );
}

#[test]
fn mixed_modality_applies_dpi_floor() {
    let profile = ManuscriptProfile::resolve(PageModality::Mixed, 120);
    assert!(profile.dpi >= 300);
    assert!(profile.max_rendered_pixels >= 3600);
    assert!(profile.skip_edgeparse_fastpath);
}

#[test]
fn print_modality_uses_adaptive_dpi() {
    let profile = ManuscriptProfile::resolve(PageModality::Print, 150);
    assert_eq!(profile.dpi, 150);
    assert_eq!(profile.max_rendered_pixels, 2000);
    assert!(!profile.skip_edgeparse_fastpath);
}

#[test]
fn manuscript_bypasses_adaptive_clamp() {
    let profile = ManuscriptProfile::resolve(PageModality::Manuscript, 96);
    assert!(
        profile.bypasses_adaptive_clamp(PageModality::Manuscript),
        "Manuscript profile must bypass adaptive DPI clamp"
    );
}

#[test]
fn print_does_not_bypass_clamp() {
    let profile = ManuscriptProfile::resolve(PageModality::Print, 150);
    assert!(
        !profile.bypasses_adaptive_clamp(PageModality::Print),
        "Print profile must not bypass adaptive DPI clamp"
    );
}

#[test]
fn env_override_forces_manuscript() {
    // SAFETY: test-only env manipulation; restored after
    unsafe {
        std::env::set_var("EDGEQUAKE_PDF_PAGE_MODALITY", "manuscript");
    }
    let modality = PageModality::from_env().unwrap();
    assert_eq!(modality, PageModality::Manuscript);
    unsafe {
        std::env::remove_var("EDGEQUAKE_PDF_PAGE_MODALITY");
    }
}
