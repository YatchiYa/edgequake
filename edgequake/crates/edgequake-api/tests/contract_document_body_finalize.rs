//! Contract: text-insert finalize must dual-write the full body, not a summary.

use std::fs;
use std::path::PathBuf;

fn read_finalize() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/processor/text_insert/finalize.rs");
    fs::read_to_string(path).expect("finalize.rs")
}

#[test]
fn finalize_ensure_document_record_uses_full_text_content() {
    let src = read_finalize();
    assert!(
        src.contains("ensure_document_record"),
        "finalize must dual-write via ensure_document_record"
    );
    assert!(
        src.contains("&text_content"),
        "finalize must pass full text_content into ensure_document_record"
    );
    // Guard against regression of the 500-char summary overwrite bug.
    assert!(
        !src.contains("chars().take(500)"),
        "finalize must not truncate body to 500 chars before ensure_document_record"
    );
    assert!(
        !src.contains("content_summary"),
        "finalize must not pass content_summary into ensure_document_record"
    );
}
