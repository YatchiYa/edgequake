//! SPEC-023 I5 — Reciprocal Rank Fusion contract tests.

use edgequake_query::fusion::{reciprocal_rank_fusion, MixFusionMode, RRF_K};

#[test]
fn contract_rrf_respects_zero_weight_arm() {
    let lists = vec![
        vec!["local_only".to_string()],
        vec!["global_only".to_string()],
        vec!["naive_only".to_string()],
    ];
    let weights = [1.0, 0.0, 0.0];
    let fused = reciprocal_rank_fusion(&lists, &weights, RRF_K);
    assert_eq!(fused.len(), 1);
    assert_eq!(fused[0].0, "local_only");
}

#[test]
fn contract_rrf_equal_weights_ties_on_swap() {
    let lists = vec![
        vec!["x".to_string(), "y".to_string()],
        vec!["y".to_string(), "x".to_string()],
    ];
    let weights = [1.0, 1.0];
    let fused = reciprocal_rank_fusion(&lists, &weights, RRF_K);
    assert_eq!(fused.len(), 2);
    assert!(
        (fused[0].1 - fused[1].1).abs() < f32::EPSILON,
        "symmetric ranks must produce equal RRF scores"
    );
}

#[test]
fn contract_mix_fusion_env_modes() {
    std::env::remove_var("EDGEQUAKE_MIX_FUSION");
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_from_env(),
        MixFusionMode::Rrf
    );

    std::env::set_var("EDGEQUAKE_MIX_FUSION", "weighted");
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_from_env(),
        MixFusionMode::MaxAfterMinMax,
        "D-35: legacy weighted alias → MaxAfterMinMax"
    );
    std::env::set_var("EDGEQUAKE_MIX_FUSION", "max_after_minmax");
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_from_env(),
        MixFusionMode::MaxAfterMinMax
    );
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_label(MixFusionMode::MaxAfterMinMax),
        "max_after_minmax"
    );

    std::env::set_var("EDGEQUAKE_MIX_FUSION", "round_robin");
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_from_env(),
        MixFusionMode::RoundRobin
    );

    std::env::remove_var("EDGEQUAKE_MIX_FUSION");
    assert_eq!(
        edgequake_query::fusion::mix_fusion_mode_from_env(),
        MixFusionMode::Rrf
    );
}
