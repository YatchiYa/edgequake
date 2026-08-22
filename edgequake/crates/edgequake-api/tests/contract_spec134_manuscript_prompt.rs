//! SPEC-134 contract: Manuscript Pass-A prompt fidelity rules.

use edgequake_pdf::vision_prompts::{
    pass_a_system_prompt_for, RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT,
    RAG_PAGE_VISION_SYSTEM_PROMPT,
};
use edgequake_pdf::PageModality;

#[test]
fn manuscript_prompt_contains_fidelity_rules() {
    let prompt = RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT;
    // Must have [?] unreadable marker rule
    assert!(
        prompt.contains("[?]"),
        "Manuscript prompt must contain [?] unreadable marker"
    );
    // Must preserve source language (no English pin)
    assert!(
        prompt.contains("SAME LANGUAGE"),
        "Manuscript prompt must preserve source language"
    );
    // Must treat hand charts as whole units
    assert!(
        prompt.contains("SINGLE semantic unit"),
        "Manuscript prompt must treat charts as whole units"
    );
    // Must NOT contain English pin from print prompt
    assert!(
        !prompt.contains("Write all output in English"),
        "Manuscript prompt must not pin English"
    );
}

#[test]
fn print_prompt_unchanged() {
    let prompt = RAG_PAGE_VISION_SYSTEM_PROMPT;
    // Print prompt must still have English pin
    assert!(
        prompt.contains("Write all output in English"),
        "Print prompt must retain English pin"
    );
    // Print prompt must NOT have manuscript-specific rules
    assert!(
        !prompt.contains("[?]"),
        "Print prompt must not contain manuscript [?] marker"
    );
}

#[test]
fn prompt_selector_routes_correctly() {
    assert_eq!(
        pass_a_system_prompt_for(PageModality::Print),
        RAG_PAGE_VISION_SYSTEM_PROMPT
    );
    assert_eq!(
        pass_a_system_prompt_for(PageModality::Manuscript),
        RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT
    );
    assert_eq!(
        pass_a_system_prompt_for(PageModality::Mixed),
        RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT
    );
}

#[test]
fn manuscript_prompt_has_chart_kv_rules() {
    let prompt = RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT;
    assert!(
        prompt.contains("GFM Markdown table"),
        "Manuscript prompt must require GFM table for charts"
    );
    assert!(
        prompt.contains("Key values:"),
        "Manuscript prompt must require Key values bullet list"
    );
}
