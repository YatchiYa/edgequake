//! SPEC-047 / 020 B2 — Hybrid arm honesty e2e.
//!
//! Law: `mode=hybrid` must not collapse Factual→naive-only (smoke showed ~96%
//! `naive_only_rate`). Mix keeps the aggressive cost gate; Hybrid uses
//! [`intent_arm_mask_hybrid`].

use edgequake_query::mix_weights::{
    intent_arm_mask, intent_arm_mask_hybrid, resolve_arm_plan, resolve_hybrid_arm_plan,
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
fn e2e_b2_mix_factual_stays_naive_only_for_cost() {
    let plan = resolve_arm_plan(
        &QueryEngineConfig::default(),
        None,
        QueryIntent::Factual,
        true,
    );
    assert!(!plan.run_local && !plan.run_global && plan.run_naive);
    assert_eq!(intent_arm_mask(QueryIntent::Factual), (false, false, true));
    assert_eq!(
        intent_arm_mask_hybrid(QueryIntent::Factual),
        (true, false, true),
        "hybrid mask must diverge from Mix on Factual"
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
