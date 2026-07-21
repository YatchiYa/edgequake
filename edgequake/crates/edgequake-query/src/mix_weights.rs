//! Mix-mode weight resolution + intent arm gating (SPEC-022 / SPEC-046 OPS-P1 / 065).
//!
//! SOLID: single responsibility — resolve which Mix/Hybrid arms run and at what weight.
//! DRY: Mix and Hybrid share [`ArmPlan`] / weight helpers; Hybrid uses
//! [`intent_arm_mask_hybrid`] for Linked cost routing.
//!
//! **Law (LightRAG mix / product Smart):** Mix always runs local ∥ global ∥ naive.
//! Intent arm collapse belongs on Hybrid (Linked), not Mix.

use serde::{Deserialize, Serialize};

use crate::engine_impl::QueryEngineConfig;
use crate::keywords::QueryIntent;

/// Metadata keys written onto [`crate::context::QueryContext`] for Mix/Hybrid arms.
pub const META_ARM_LOCAL_MS: &str = "arm_local_ms";
pub const META_ARM_GLOBAL_MS: &str = "arm_global_ms";
pub const META_ARM_NAIVE_MS: &str = "arm_naive_ms";
pub const META_ARM_LOCAL_CHUNKS: &str = "arm_local_chunks";
pub const META_ARM_GLOBAL_CHUNKS: &str = "arm_global_chunks";
pub const META_ARM_NAIVE_CHUNKS: &str = "arm_naive_chunks";
pub const META_ARMS_RUN: &str = "arms_run";
pub const META_ARMS_GATED: &str = "arms_gated";

/// Optional per-request Mix weight override (unset fields use engine config defaults).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MixWeightOverride {
    #[serde(default)]
    pub local: Option<f32>,
    #[serde(default)]
    pub global: Option<f32>,
    #[serde(default)]
    pub naive: Option<f32>,
}

impl MixWeightOverride {
    pub fn is_set(&self) -> bool {
        self.local.is_some() || self.global.is_some() || self.naive.is_some()
    }
}

/// Which retrieval arms to execute and their fusion weights (already normalized).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmPlan {
    pub run_local: bool,
    pub run_global: bool,
    pub run_naive: bool,
    pub w_local: f32,
    pub w_global: f32,
    pub w_naive: f32,
}

impl ArmPlan {
    pub fn empty_context_ok(&self) -> bool {
        !self.run_local && !self.run_global && !self.run_naive
    }
}

/// Parse `EDGEQUAKE_MIX_ARM_GATE` (pure — pass raw for tests).
///
/// Default **false** (gate off = LightRAG-like Mix arms). Opt in with
/// `true`/`1`/`on`/`yes`. Mix [`intent_arm_mask`] is always all arms, so enabling
/// the gate is a no-op for Mix today; Hybrid still uses its own mask when gated.
pub fn parse_mix_arm_gate(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "on" | "yes"
    )
}

pub fn mix_arm_gate_enabled() -> bool {
    parse_mix_arm_gate(&std::env::var("EDGEQUAKE_MIX_ARM_GATE").unwrap_or_default())
}

/// Mix arm mask — LightRAG `mix` law (product Smart).
///
/// Always `(local, global, naive)` for every intent. Cost-aware arm collapse lives
/// on Hybrid via [`intent_arm_mask_hybrid`], not Mix.
pub fn intent_arm_mask(_intent: QueryIntent) -> (bool, bool, bool) {
    (true, true, true)
}

/// Hybrid arm mask (SPEC-047 / 020 B2).
///
/// Law: requesting `mode=hybrid` means multi-arm fusion. Collapsing Factual→naive-only
/// made hybrid a lie on MMLongBench (≈96% `naive_only_rate`). Mix always runs all
/// three arms (LightRAG); Hybrid retains naive plus at least one graph arm when gated.
pub fn intent_arm_mask_hybrid(intent: QueryIntent) -> (bool, bool, bool) {
    match intent {
        // Local + naive: entity neighborhood + page chunks (skip global community tax).
        QueryIntent::Factual => (true, false, true),
        // Relational: keep graph arms and add naive for page-grounded evidence.
        QueryIntent::Relational => (true, true, true),
        // Exploratory: global + naive (page evidence still required for Acc).
        QueryIntent::Exploratory => (false, true, true),
        QueryIntent::Comparative | QueryIntent::Procedural => (true, true, true),
    }
}

/// Parse a Mix arm weight from env (`EDGEQUAKE_MIX_{LOCAL,GLOBAL,NAIVE}_WEIGHT`).
///
/// Default **1.0** when unset / invalid. Values are clamped to `[0.0, 10.0]`
/// before normalization in [`normalized_mix_weights`].
pub fn mix_arm_weight_from_env(var: &str, default: f32) -> f32 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, 10.0))
        .unwrap_or(default)
}

/// Normalize Mix weights to sum to 1 (P-G8 E24/E25).
pub fn normalized_mix_weights(
    config: &QueryEngineConfig,
    override_weights: Option<&MixWeightOverride>,
) -> (f32, f32, f32) {
    let l = override_weights
        .and_then(|o| o.local)
        .unwrap_or(config.mix_local_weight);
    let g = override_weights
        .and_then(|o| o.global)
        .unwrap_or(config.mix_global_weight);
    let n = override_weights
        .and_then(|o| o.naive)
        .unwrap_or(config.mix_naive_weight);
    let sum = l + g + n;
    if !sum.is_finite() || sum <= 0.0 {
        tracing::warn!(
            mix_local_weight = l,
            mix_global_weight = g,
            mix_naive_weight = n,
            "Mix weights sum to 0 or are non-finite; falling back to equal weights (P-G8 E24)"
        );
        (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    } else {
        (l / sum, g / sum, n / sum)
    }
}

/// Resolve arm plan: apply intent gate (unless disabled / explicit override forces arms).
///
/// Explicit `MixWeightOverride` with any field set **disables** intent gating
/// (operator/eval wants specific arms). Zero-weight arms after normalize are skipped.
pub fn resolve_arm_plan(
    config: &QueryEngineConfig,
    override_weights: Option<&MixWeightOverride>,
    intent: QueryIntent,
    gate_enabled: bool,
) -> ArmPlan {
    let (mut w_local, mut w_global, mut w_naive) = normalized_mix_weights(config, override_weights);

    let explicit = override_weights.is_some_and(|o| o.is_set());
    if gate_enabled && !explicit {
        let (ml, mg, mn) = intent_arm_mask(intent);
        if !ml {
            w_local = 0.0;
        }
        if !mg {
            w_global = 0.0;
        }
        if !mn {
            w_naive = 0.0;
        }
        // Re-normalize surviving arms
        let sum = w_local + w_global + w_naive;
        if sum > 0.0 && sum.is_finite() {
            w_local /= sum;
            w_global /= sum;
            w_naive /= sum;
        } else {
            // Degenerate — fall back to naive
            w_local = 0.0;
            w_global = 0.0;
            w_naive = 1.0;
        }
    }

    ArmPlan {
        run_local: w_local > 0.0,
        run_global: w_global > 0.0,
        run_naive: w_naive > 0.0,
        w_local,
        w_global,
        w_naive,
    }
}

/// Hybrid arm plan: equal weights among surviving arms (intent gate only).
///
/// Hybrid does not take Mix weight overrides — gating uses [`intent_arm_mask_hybrid`]
/// so Factual does not collapse to naive-only (020 B2).
pub fn resolve_hybrid_arm_plan(intent: QueryIntent, gate_enabled: bool) -> ArmPlan {
    let (ml, mg, mn) = if gate_enabled {
        intent_arm_mask_hybrid(intent)
    } else {
        (true, true, true)
    };
    let n = [ml, mg, mn].iter().filter(|&&x| x).count().max(1) as f32;
    ArmPlan {
        run_local: ml,
        run_global: mg,
        run_naive: mn,
        w_local: if ml { 1.0 / n } else { 0.0 },
        w_global: if mg { 1.0 / n } else { 0.0 },
        w_naive: if mn { 1.0 / n } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> QueryEngineConfig {
        QueryEngineConfig::default()
    }

    #[test]
    fn mix_factual_always_runs_all_arms() {
        // LightRAG mix / product Smart: gate on or off, Mix keeps local+global+naive.
        for gate in [true, false] {
            let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Factual, gate);
            assert!(
                plan.run_local && plan.run_global && plan.run_naive,
                "Mix Factual must run all arms (gate={gate}); got {plan:?}"
            );
        }
        assert!(
            (resolve_arm_plan(&cfg(), None, QueryIntent::Factual, true).w_local - 1.0 / 3.0).abs()
                < 1e-5
        );
    }

    #[test]
    fn hybrid_factual_keeps_local_and_naive_b2() {
        let plan = resolve_hybrid_arm_plan(QueryIntent::Factual, true);
        assert!(
            plan.run_local && !plan.run_global && plan.run_naive,
            "020 B2: hybrid Factual must not collapse to naive-only; got {plan:?}"
        );
        assert!((plan.w_local - 0.5).abs() < 1e-5);
        assert!((plan.w_naive - 0.5).abs() < 1e-5);
    }

    #[test]
    fn hybrid_mask_differs_from_mix_on_factual() {
        assert_eq!(intent_arm_mask(QueryIntent::Factual), (true, true, true));
        assert_eq!(
            intent_arm_mask_hybrid(QueryIntent::Factual),
            (true, false, true),
            "Hybrid Factual skips global; Mix keeps all three"
        );
    }

    #[test]
    fn mix_relational_keeps_naive() {
        let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Relational, true);
        assert!(plan.run_local && plan.run_global && plan.run_naive);
    }

    #[test]
    fn hybrid_relational_includes_naive() {
        let plan = resolve_hybrid_arm_plan(QueryIntent::Relational, true);
        assert!(plan.run_local && plan.run_global && plan.run_naive);
        assert!((plan.w_local - 1.0 / 3.0).abs() < 1e-5);
        assert!((plan.w_global - 1.0 / 3.0).abs() < 1e-5);
        assert!((plan.w_naive - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn gate_off_runs_all_arms() {
        let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Factual, false);
        assert!(plan.run_local && plan.run_global && plan.run_naive);
        let h = resolve_hybrid_arm_plan(QueryIntent::Factual, false);
        assert!(h.run_local && h.run_global && h.run_naive);
    }

    #[test]
    fn explicit_override_bypasses_gate() {
        let ov = MixWeightOverride {
            local: Some(1.0),
            global: Some(0.0),
            naive: Some(0.0),
        };
        let plan = resolve_arm_plan(&cfg(), Some(&ov), QueryIntent::Factual, true);
        assert!(plan.run_local && !plan.run_global && !plan.run_naive);
    }

    #[test]
    fn parse_mix_arm_gate_edge_cases() {
        // Default off (LightRAG-like product Smart).
        assert!(!parse_mix_arm_gate(""));
        assert!(!parse_mix_arm_gate("false"));
        assert!(!parse_mix_arm_gate("0"));
        assert!(!parse_mix_arm_gate("force_all"));
        assert!(!parse_mix_arm_gate("all"));
        assert!(parse_mix_arm_gate("true"));
        assert!(parse_mix_arm_gate("1"));
        assert!(parse_mix_arm_gate("on"));
        assert!(parse_mix_arm_gate("yes"));
    }

    #[test]
    fn intent_masks_cover_all_variants() {
        for intent in [
            QueryIntent::Factual,
            QueryIntent::Relational,
            QueryIntent::Exploratory,
            QueryIntent::Comparative,
            QueryIntent::Procedural,
        ] {
            assert_eq!(
                intent_arm_mask(intent),
                (true, true, true),
                "Mix mask must be LightRAG always-on for {intent:?}"
            );
        }
        assert_eq!(
            intent_arm_mask_hybrid(QueryIntent::Exploratory),
            (false, true, true)
        );
    }

    #[test]
    fn hybrid_plan_equal_weights_among_survivors() {
        // Factual hybrid: local+naive survivors → equal 0.5
        let plan = resolve_hybrid_arm_plan(QueryIntent::Factual, true);
        assert!(plan.run_local && !plan.run_global && plan.run_naive);
        assert!((plan.w_local - 0.5).abs() < 1e-5);
        assert!((plan.w_naive - 0.5).abs() < 1e-5);
    }

    #[test]
    fn naive_weight_boost_normalizes_to_half() {
        // Acc-win E3b: local=1, global=1, naive=2 → 0.25 / 0.25 / 0.50
        let mut c = cfg();
        c.mix_local_weight = 1.0;
        c.mix_global_weight = 1.0;
        c.mix_naive_weight = 2.0;
        let (l, g, n) = normalized_mix_weights(&c, None);
        assert!((l - 0.25).abs() < 1e-5);
        assert!((g - 0.25).abs() < 1e-5);
        assert!((n - 0.5).abs() < 1e-5);
        let plan = resolve_arm_plan(&c, None, QueryIntent::Comparative, false);
        assert!((plan.w_naive - 0.5).abs() < 1e-5);
    }

    #[test]
    fn mix_arm_weight_from_env_clamps() {
        assert!(
            (mix_arm_weight_from_env("EDGEQUAKE_MIX_NAIVE_WEIGHT_UNSET_XYZ", 1.0) - 1.0).abs()
                < 1e-5
        );
    }
}
