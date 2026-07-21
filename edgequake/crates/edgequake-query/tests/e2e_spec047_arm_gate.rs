//! SPEC-047 / 020 B2 / 065 — Mix & Hybrid arm honesty e2e.
//!
//! Law:
//! - Product Smart (`mode=mix`) = LightRAG mix → always local ∥ global ∥ naive.
//! - Linked (`mode=hybrid`) keeps [`intent_arm_mask_hybrid`] so Factual is
//!   local+naive (not naive-only, not full Mix tax).

use edgequake_query::mix_weights::{
    intent_arm_mask, intent_arm_mask_hybrid, parse_mix_arm_gate, resolve_arm_plan,
    resolve_hybrid_arm_plan,
};
use edgequake_query::{QueryEngineConfig, QueryIntent};

#[test]
fn e2e_b2_hybrid_factual_is_not_naive_only() {
    let plan = resolve_hybrid_arm_plan(QueryIntent::Factual, true);
    assert!(
        plan.run_local && plan.run_naive,
        "hybrid Factual must run local+naive; got {plan:?}"
    );
    assert!(
        !plan.run_global,
        "hybrid Factual skips global community tax; got {plan:?}"
    );
    assert!(
        !(plan.run_naive && !plan.run_local && !plan.run_global),
        "must not be naive-only under hybrid"
    );
}

#[test]
fn e2e_065_mix_factual_runs_all_arms_lightrag() {
    // Gate on or off — Mix mask is always-on LightRAG arms.
    for gate in [true, false] {
        let plan = resolve_arm_plan(
            &QueryEngineConfig::default(),
            None,
            QueryIntent::Factual,
            gate,
        );
        assert!(
            plan.run_local && plan.run_global && plan.run_naive,
            "Mix Factual must run all three arms (gate={gate}); got {plan:?}"
        );
    }
    assert_eq!(intent_arm_mask(QueryIntent::Factual), (true, true, true));
    assert_eq!(
        intent_arm_mask_hybrid(QueryIntent::Factual),
        (true, false, true),
        "hybrid mask must diverge from Mix on Factual (skips global)"
    );
}

#[test]
fn e2e_065_mix_arm_gate_defaults_off() {
    assert!(
        !parse_mix_arm_gate(""),
        "unset EDGEQUAKE_MIX_ARM_GATE must default off (product Smart = LR mix)"
    );
}

#[test]
fn e2e_b2_hybrid_relational_keeps_page_arm() {
    let plan = resolve_hybrid_arm_plan(QueryIntent::Relational, true);
    assert!(
        plan.run_naive,
        "hybrid Relational must keep naive for page evidence; got {plan:?}"
    );
    assert!(plan.run_local && plan.run_global);
}

#[test]
fn e2e_065_mix_all_intents_keep_three_arms() {
    for intent in [
        QueryIntent::Factual,
        QueryIntent::Relational,
        QueryIntent::Exploratory,
        QueryIntent::Comparative,
        QueryIntent::Procedural,
    ] {
        let plan = resolve_arm_plan(&QueryEngineConfig::default(), None, intent, true);
        assert!(
            plan.run_local && plan.run_global && plan.run_naive,
            "Mix {intent:?} must keep LightRAG three-arm set; got {plan:?}"
        );
    }
}
