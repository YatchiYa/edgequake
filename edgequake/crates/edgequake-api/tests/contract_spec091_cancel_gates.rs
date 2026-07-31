//! SPEC-091 WP1 / WP-AC-04: CancelGate SSOT is exhaustive and wired.
//!
//! Run: `cargo test -p edgequake-api --test contract_spec091_cancel_gates`

use edgequake_api::processor::cancel_gates::CancelGate;

#[test]
fn cancel_gate_all_unique_and_roundtrip() {
    let mut seen = std::collections::HashSet::new();
    for g in CancelGate::ALL {
        assert!(seen.insert(g.as_str()), "duplicate gate id {}", g.as_str());
        assert_eq!(CancelGate::parse(g.as_str()).unwrap(), *g);
    }
    assert!(
        CancelGate::ALL.len() >= 10,
        "expected full WP1 gate set, got {}",
        CancelGate::ALL.len()
    );
}

#[test]
fn cancel_gate_required_wp1_ids_present() {
    let ids: Vec<&str> = CancelGate::ALL.iter().map(|g| g.as_str()).collect();
    for required in [
        "claim",
        "park",
        "pre-prepare",
        "pre-extraction",
        "pre-embed",
        "pre-graph-storage",
        "pre-promote",
        "pre-vision-extraction",
        "pre-ingest-enqueue",
    ] {
        assert!(
            ids.contains(&required),
            "missing CancelGate id {required} in {:?}",
            ids
        );
    }
}

#[test]
fn processor_sources_call_required_text_insert_gates() {
    let prepare = include_str!("../src/processor/text_insert/prepare.rs");
    let extraction = include_str!("../src/processor/text_insert/extraction.rs");
    let persist = include_str!("../src/processor/text_insert/persist.rs");
    let finalize = include_str!("../src/processor/text_insert/finalize.rs");
    assert!(
        prepare.contains("pre-prepare"),
        "prepare must check pre-prepare"
    );
    assert!(
        extraction.contains("pre-extraction"),
        "extraction must check pre-extraction"
    );
    assert!(
        extraction.contains("pre-embed"),
        "extraction must check pre-embed"
    );
    assert!(
        persist.contains("pre-graph-storage"),
        "persist must check pre-graph-storage (PreMaterialize)"
    );
    assert!(
        finalize.contains("pre-promote"),
        "finalize must check pre-promote"
    );
}
