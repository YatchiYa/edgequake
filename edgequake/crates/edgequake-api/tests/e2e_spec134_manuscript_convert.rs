//! SPEC-134 E2E: Manuscript modality → full pipeline with profile (mock).

use edgequake_pdf::{
    pass_a_system_prompt_for, ManuscriptProfile, PageModality,
    RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT,
};
use serial_test::serial;

/// E2E mock: forced manuscript modality applies profile + prompt correctly.
#[test]
#[serial]
fn forced_manuscript_modality_applies_profile() {
    // SAFETY: test-only env manipulation; restored after
    unsafe {
        std::env::set_var("EDGEQUAKE_PDF_PAGE_MODALITY", "manuscript");
    }

    let modality = PageModality::from_env().unwrap();
    assert_eq!(modality, PageModality::Manuscript);

    let profile = ManuscriptProfile::resolve(modality, 96);
    assert!(profile.dpi >= 300, "MS DPI floor must apply");
    assert!(
        profile.max_rendered_pixels >= 3600,
        "MS max_pixels floor must apply"
    );
    assert!(profile.skip_edgeparse_fastpath, "MS must skip EdgeParse");

    let prompt = pass_a_system_prompt_for(modality);
    assert_eq!(prompt, RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT);
    assert!(prompt.contains("[?]"), "MS prompt must have [?] marker");
    assert!(
        prompt.contains("SAME LANGUAGE"),
        "MS prompt must preserve language"
    );

    unsafe {
        std::env::remove_var("EDGEQUAKE_PDF_PAGE_MODALITY");
    }
}

/// E2E mock: forced mixed modality also applies manuscript profile.
#[test]
#[serial]
fn forced_mixed_modality_applies_profile() {
    unsafe {
        std::env::set_var("EDGEQUAKE_PDF_PAGE_MODALITY", "mixed");
    }

    let modality = PageModality::from_env().unwrap();
    assert_eq!(modality, PageModality::Mixed);

    let profile = ManuscriptProfile::resolve(modality, 120);
    assert!(profile.dpi >= 300);
    assert!(profile.skip_edgeparse_fastpath);

    let prompt = pass_a_system_prompt_for(modality);
    assert_eq!(prompt, RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT);

    unsafe {
        std::env::remove_var("EDGEQUAKE_PDF_PAGE_MODALITY");
    }
}

/// E2E mock: print modality (default) uses existing behavior.
#[test]
#[serial]
fn print_modality_uses_existing_behavior() {
    // Ensure no env override
    unsafe {
        std::env::remove_var("EDGEQUAKE_PDF_PAGE_MODALITY");
    }

    let modality = PageModality::from_env().unwrap_or(PageModality::Print);
    assert_eq!(modality, PageModality::Print);

    let profile = ManuscriptProfile::resolve(modality, 150);
    assert_eq!(profile.dpi, 150, "Print must use adaptive DPI");
    assert_eq!(
        profile.max_rendered_pixels, 2000,
        "Print must use default max_pixels"
    );
    assert!(
        !profile.skip_edgeparse_fastpath,
        "Print must not skip EdgeParse"
    );

    let prompt = pass_a_system_prompt_for(modality);
    assert!(
        prompt.contains("Write all output in English"),
        "Print prompt must keep EN pin"
    );
    assert!(
        !prompt.contains("[?]"),
        "Print prompt must not have MS marker"
    );
}
