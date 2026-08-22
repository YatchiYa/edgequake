//! SPEC-134 Slice B contract: modality-adaptive strategy is wired on the
//! production path in `pdf_processing.rs` — prompt routing (WP-3), figure
//! filter gating (WP-4), and manuscript env model routing (WP-10).
//!
//! Behavioral coverage of the policy helpers lives in the crate-local
//! `spec134_strategy` unit-test module; these tests pin the production wiring
//! so a future refactor cannot silently drop the routing.

/// Production source of the PDF task processor (the SSOT for the wiring).
fn prod_src() -> &'static str {
    include_str!("../src/processor/pdf_processing.rs")
}

#[test]
fn pass_a_prompt_routed_on_production_path() {
    let src = prod_src();
    assert!(
        src.contains("route_pass_a_system_prompt("),
        "production path must route the Pass-A prompt by modality"
    );
    // Precedence (LAW-134-5): explicit upload prompt wins — the routing helper
    // must only fill the prompt when none was explicitly set.
    assert!(
        src.contains("if cfg.page_system_prompt.is_none() && modality.is_manuscript_like()"),
        "prompt routing must preserve explicit upload prompts (explicit > modality > print)"
    );
}

#[test]
fn figure_filter_gated_to_print_modality() {
    let src = prod_src();
    assert!(
        src.contains("should_attach_figure_filter("),
        "SPEC-128 figure filter attach must be gated by modality"
    );
    assert!(
        src.contains("&& modality == edgequake_pdf::PageModality::Print"),
        "figure filter must attach for Print modality only (manuscript pages \
         treat embedded XObjects as scan-tiling artifacts, LAW-134-16)"
    );
}

#[test]
fn manuscript_env_model_routing_on_production_path() {
    let src = prod_src();
    assert!(
        src.contains("resolve_vision_provider_for_modality(page_modality, &data.vision_provider)"),
        "vision provider must be resolved through the manuscript routing helper"
    );
    assert!(
        src.contains("resolve_vision_model_for_modality("),
        "vision model must be resolved through the manuscript routing helper"
    );
    assert!(
        src.contains("EDGEQUAKE_VISION_PROVIDER_MANUSCRIPT"),
        "provider override env var must be honored"
    );
    assert!(
        src.contains("EDGEQUAKE_VISION_MODEL_MANUSCRIPT"),
        "model override env var must be honored"
    );
}

#[test]
fn modality_classified_before_edgeparse_auto() {
    let src = prod_src();
    let classify = src
        .find("classify_pages_from_bytes")
        .expect("per-page classify must run on the production path");
    let edgeparse = src
        .find("try_edgeparse_fast_path")
        .expect("SPEC-038 Auto still exists");
    assert!(
        classify < edgeparse,
        "LAW-134-12: classify pages before EdgeParse Auto"
    );
    assert!(
        src.contains("should_skip_edgeparse"),
        "skip_edgeparse_fastpath must be consulted via PageConvertPlan"
    );
}

#[test]
fn manuscript_prompt_ssot_distinct_from_print() {
    // The two prompts must remain distinct SSOTs — the manuscript prompt
    // carries the no-English-pin + whole-graphic rules.
    let print = edgequake_pdf::pass_a_system_prompt_for(edgequake_pdf::PageModality::Print);
    let ms = edgequake_pdf::pass_a_system_prompt_for(edgequake_pdf::PageModality::Manuscript);
    let mixed = edgequake_pdf::pass_a_system_prompt_for(edgequake_pdf::PageModality::Mixed);
    assert_ne!(print, ms, "manuscript prompt must differ from print SSOT");
    assert_eq!(ms, mixed, "mixed pages use the manuscript prompt");
    assert!(
        !ms.contains("Write ALL output in English"),
        "manuscript prompt must not carry the print English pin"
    );
}
