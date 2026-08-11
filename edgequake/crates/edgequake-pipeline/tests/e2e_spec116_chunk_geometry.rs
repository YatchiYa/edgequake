//! SPEC-116 — Geometry e2e: Fixed Acc-fair vs Adaptive chunk counts on mid-size text.
//!
//! Run:
//!   cargo test -p edgequake-pipeline --test e2e_spec116_chunk_geometry -- --nocapture

use edgequake_pipeline::{
    build_chunker_config_with_policy, resolve_chunker, ChunkStrategy, ChunkingPolicy,
};

fn chunk_count_for(policy: ChunkingPolicy, text: &str) -> usize {
    let config =
        build_chunker_config_with_policy(text.len(), ChunkStrategy::Recursive, Some(&policy), None);
    let chunker = resolve_chunker(ChunkStrategy::Recursive, config);
    chunker.chunk(text, "spec116").expect("chunk").len()
}

#[test]
fn spec116_adaptive_yields_more_chunks_than_fixed_acc_fair() {
    let unit = "The quick brown fox jumps over the lazy dog. ";
    let reps = 61_000 / unit.len() + 1;
    let text = unit.repeat(reps);
    assert!(text.len() > 50_000 && text.len() < 100_000);

    let fixed_n = chunk_count_for(ChunkingPolicy::acc_fair(), &text);
    let adaptive_n = chunk_count_for(ChunkingPolicy::Adaptive, &text);

    assert!(
        adaptive_n > fixed_n,
        "Adaptive ({adaptive_n}) should exceed Fixed Acc-fair ({fixed_n}) on ~61KB text"
    );
    assert!(
        (8..=40).contains(&fixed_n),
        "unexpected Fixed count {fixed_n}"
    );
    assert!(
        (10..=50).contains(&adaptive_n),
        "unexpected Adaptive count {adaptive_n}"
    );
}

#[test]
fn spec116_workspace_isolation_policies_independent() {
    let text = "alpha beta gamma. ".repeat(4_000);
    let a = chunk_count_for(
        ChunkingPolicy::Fixed {
            size: 1200,
            overlap: 100,
        },
        &text,
    );
    let b = chunk_count_for(ChunkingPolicy::Adaptive, &text);
    assert_ne!(a, b, "two policies on same text must differ in geometry");
}
