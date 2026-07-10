//! Mix-mode weight resolution + intent arm gating (SPEC-022 / SPEC-046 OPS-P1).
//!
//! SOLID: single responsibility — resolve which Mix/Hybrid arms run and at what weight.
//! DRY: both Mix and Hybrid call [`resolve_arm_plan`].

use serde::{Deserialize, Serialize};

use crate::engine_impl::QueryEngineConfig;
use crate::keywords::QueryIntent;

/// Metadata keys written onto [`crate::context::QueryContext`] for Mix/Hybrid arms.
pub const META_ARM_LOCAL_MS: &str = "arm_local_ms";
pub const META_ARM_GLOBAL_MS: &str = "arm_global_ms";
pub const META_ARM_NAIVE_MS: &str = "arm_naive_ms";
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
/// Default **true** (gate on). Set `false`/`off`/`0` to force all arms.
/// `force_all` / `all` also disables gating.
pub fn parse_mix_arm_gate(raw: &str) -> bool {
    !matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "no" | "force_all" | "all"
    )
}

pub fn mix_arm_gate_enabled() -> bool {
    parse_mix_arm_gate(&std::env::var("EDGEQUAKE_MIX_ARM_GATE").unwrap_or_default())
}

/// Intent → preferred arm mask when gating is on (SPEC-046 OPS-P1.3).
///
/// Even when the client forces `mode=mix|hybrid`, L1 factual queries should
/// not pay the full 3-arm tax unless the operator disables the gate.
pub fn intent_arm_mask(intent: QueryIntent) -> (bool, bool, bool) {
    match intent {
        QueryIntent::Factual => (false, false, true), // naive only
        QueryIntent::Relational => (true, true, false), // local + global
        QueryIntent::Exploratory => (false, true, false), // global
        QueryIntent::Comparative | QueryIntent::Procedural => (true, true, true),
    }
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
/// Hybrid does not take Mix weight overrides — gating is pure intent / env.
pub fn resolve_hybrid_arm_plan(intent: QueryIntent, gate_enabled: bool) -> ArmPlan {
    let (ml, mg, mn) = if gate_enabled {
        intent_arm_mask(intent)
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
    fn factual_gate_runs_naive_only() {
        let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Factual, true);
        assert!(!plan.run_local && !plan.run_global && plan.run_naive);
        assert!((plan.w_naive - 1.0).abs() < 1e-5);
    }

    #[test]
    fn relational_gate_skips_naive() {
        let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Relational, true);
        assert!(plan.run_local && plan.run_global && !plan.run_naive);
    }

    #[test]
    fn gate_off_runs_all_arms() {
        let plan = resolve_arm_plan(&cfg(), None, QueryIntent::Factual, false);
        assert!(plan.run_local && plan.run_global && plan.run_naive);
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
        assert!(parse_mix_arm_gate(""));
        assert!(parse_mix_arm_gate("true"));
        assert!(!parse_mix_arm_gate("false"));
        assert!(!parse_mix_arm_gate("force_all"));
        assert!(!parse_mix_arm_gate("0"));
    }

    #[test]
    fn intent_masks_cover_all_variants() {
        assert_eq!(
            intent_arm_mask(QueryIntent::Exploratory),
            (false, true, false)
        );
        assert_eq!(
            intent_arm_mask(QueryIntent::Comparative),
            (true, true, true)
        );
        assert_eq!(intent_arm_mask(QueryIntent::Procedural), (true, true, true));
    }

    #[test]
    fn hybrid_plan_equal_weights_among_survivors() {
        let plan = resolve_hybrid_arm_plan(QueryIntent::Relational, true);
        assert!(plan.run_local && plan.run_global && !plan.run_naive);
        assert!((plan.w_local - 0.5).abs() < 1e-5);
        assert!((plan.w_global - 0.5).abs() < 1e-5);
    }
}
