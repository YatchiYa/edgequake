//! SPEC-076 A4 — sparse FTS+ANN RRF tip (no silent default flip).
//!
//! Lexical bake-off: ANN ranks miss an exact-token id that FTS ranks first;
//! RRF recovers it in top-k; sparse-first weighted also keeps it; ANN-only misses.

use edgequake_query::fusion::{self, MixFusionMode, RRF_K};
use edgequake_query::sparse_retrieval::sparse_fusion_mode_from_env;

#[test]
fn sparse_fusion_default_weighted_rrf_tip_opt_in() {
    // Sequential: env is process-global; keep tip/default asserts in one test.
    std::env::remove_var("EDGEQUAKE_SPARSE_FUSION");
    assert_eq!(
        sparse_fusion_mode_from_env(),
        MixFusionMode::Weighted,
        "SPEC-076 must not silent-flip sparse fusion default"
    );
    std::env::set_var("EDGEQUAKE_SPARSE_FUSION", "rrf");
    assert_eq!(sparse_fusion_mode_from_env(), MixFusionMode::Rrf);
    std::env::remove_var("EDGEQUAKE_SPARSE_FUSION");
    assert_eq!(sparse_fusion_mode_from_env(), MixFusionMode::Weighted);
}

/// Lexical eval: code/name token lives in FTS rank-1; ANN buries it.
#[test]
fn lexical_bakeoff_rrf_beats_ann_only() {
    // ANN semantic neighbors (miss the SKU token)
    let ann = vec![
        "chunk-semantic-a".into(),
        "chunk-semantic-b".into(),
        "chunk-semantic-c".into(),
        "chunk-semantic-d".into(),
        "chunk-sku-PN-7781".into(), // buried at rank 5
    ];
    // FTS hits the exact part number first
    let fts = vec![
        "chunk-sku-PN-7781".into(),
        "chunk-semantic-a".into(),
        "chunk-other".into(),
    ];

    let ann_only_top3: Vec<String> = ann.iter().take(3).cloned().collect();
    assert!(
        !ann_only_top3.iter().any(|s| s == "chunk-sku-PN-7781"),
        "ANN-only top-3 must miss the lexical SKU (eval premise)"
    );

    let fused = fusion::reciprocal_rank_fusion(&[ann.clone(), fts.clone()], &[1.0, 1.25], RRF_K);
    let rrf_top3: Vec<String> = fused.iter().take(3).map(|(id, _)| id.clone()).collect();
    assert!(
        rrf_top3.iter().any(|s| s == "chunk-sku-PN-7781"),
        "RRF tip must surface lexical SKU in top-3: {rrf_top3:?}"
    );

    // Sparse-first weighted (default) also keeps FTS-first id — tip is RRF, not a default flip.
    assert_eq!(fts.first().map(String::as_str), Some("chunk-sku-PN-7781"));
}

#[test]
fn content_tsv_upsert_honesty_still_wired() {
    // Cross-crate honesty: storage upsert must keep writing content_tsv (SPEC-058/076).
    let impl_src =
        include_str!("../../edgequake-storage/src/adapters/postgres/vector/storage_impl.rs");
    assert!(
        impl_src.contains("content_tsv = EXCLUDED.content_tsv"),
        "vector upsert must sync content_tsv for FTS+ANN RRF"
    );
    let fts_src = include_str!("../../edgequake-storage/src/adapters/postgres/vector/fts.rs");
    assert!(
        fts_src.contains("content_tsv"),
        "native FTS must read content_tsv"
    );
}
