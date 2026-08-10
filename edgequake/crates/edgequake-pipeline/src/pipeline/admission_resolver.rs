//! SPEC-091 QW2 — Admission resolver (LAW-Q1: capacity is derived, not guessed).
//!
//! One [`ProviderProfile`] (kind + budget B) derives **every** concurrency
//! number in the ingestion system — worker threads, extraction/embed/merge/vision
//! fan-out, queue soft bound, tenant lane weight — via [`resolve`]. The five
//! legacy resolvers (`pipeline/config.rs`, `local_inference_gate.rs`,
//! `core/resource/budget.rs`, `tasks/admission.rs`) keep their env vars as
//! overrides, but their *defaults* are pinned to this plan by the drift test
//! below (SSOT: one derivation, one drift guard).
//!
//! Model: local providers (Ollama / LM Studio) are **compute-bound** — fan-out
//! beyond B just queues at the provider gate; cloud providers are
//! **latency-bound** — 2× oversubscription hides network latency. Hence the
//! kind-aware formulas.
//!
//! Spec: `specs/091-simplify-data-layer/13-queue-admission-target-spec.md`.

use crate::pipeline::config::is_local_extraction_provider;

/// Env key: target wait used to derive the queue soft bound (seconds).
pub const QUEUE_TARGET_WAIT_SECS_ENV: &str = "EDGEQUAKE_QUEUE_TARGET_WAIT_SECS";
/// Default queue target wait (10 minutes).
pub const DEFAULT_QUEUE_TARGET_WAIT_SECS: u64 = 600;

/// Provider capacity class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Compute-bound local inference (Ollama / LM Studio).
    Local,
    /// Latency-bound cloud API (OpenAI, Azure, ...) or in-process mock.
    Cloud,
}

/// The one hand-set input: which provider and how much in-flight capacity (B).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderProfile {
    pub kind: ProviderKind,
    /// Cluster-wide in-flight budget B (LAW-Q3 ledger seeds from the same value).
    pub budget: u16,
}

impl ProviderProfile {
    /// Default local profile: single-GPU Ollama (serial execution).
    pub const LOCAL_DEFAULT: Self = Self {
        kind: ProviderKind::Local,
        budget: 1,
    };
    /// Default cloud profile: moderate API account (latency-bound).
    pub const CLOUD_DEFAULT: Self = Self {
        kind: ProviderKind::Cloud,
        budget: 8,
    };

    /// Profile for a provider name (local detection is the existing SSOT:
    /// `is_local_extraction_provider`). Budget defaults per kind; explicit
    /// `budget` overrides via [`ProviderProfile::with_budget`].
    pub fn for_provider(provider_name: &str) -> Self {
        if is_local_extraction_provider(provider_name) {
            Self::LOCAL_DEFAULT
        } else {
            Self::CLOUD_DEFAULT
        }
    }

    pub fn with_budget(self, budget: u16) -> Self {
        Self { budget, ..self }
    }
}

/// Every derived concurrency figure (LAW-Q1 SSOT). All fields are `f(B)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionPlan {
    /// Worker threads: `2B` clamped — workers wait on provider slots.
    pub worker_threads: usize,
    /// Extraction fan-out per task: `B` local (compute-bound), `2B` cloud
    /// (latency-hiding), capped.
    pub extraction_concurrency: usize,
    /// Embedding batch fan-out: `B/2` local, `B` cloud.
    pub embed_max_async: usize,
    /// Merge fan-out: `B` both kinds (cheap calls).
    pub merge_max_async: usize,
    /// Vision conversion jobs: `B` local (GPU-bound), `B` cloud.
    pub vision_jobs: usize,
    /// Queue soft bound: `ceil(λ̂ × target_wait)` — derived from measured drain
    /// rate, never a guessed constant (LAW-Q4).
    pub queue_soft_bound: u32,
    /// Tenant lane weight for DRR fair-share (LAW-Q5): equal shares.
    pub tenant_lane_weight: u32,
}

/// Resolve the plan (pure — testable SSOT).
///
/// `drain_rate_per_min` is the measured completion rate (EWMA seed: recent
/// completions per minute); `target_wait_secs` bounds honest queueing time.
pub fn resolve(
    profile: &ProviderProfile,
    drain_rate_per_min: f64,
    target_wait_secs: u64,
) -> AdmissionPlan {
    let b = usize::from(profile.budget.max(1));
    let (threads, extraction, embed, merge, vision) = match profile.kind {
        ProviderKind::Local => (
            (2 * b).clamp(1, 4),
            // Compute-bound: raise B (and ALLOW_LOCAL_HIGH_CONCURRENCY) for multi-GPU.
            // Default B=1 → extraction 1 (serial Ollama).
            b.clamp(1, 2),
            (b / 2).clamp(1, 2),
            b.clamp(1, 4),
            b.clamp(1, 2),
        ),
        ProviderKind::Cloud => (
            (2 * b).clamp(4, 32),
            (2 * b).clamp(4, 16),
            b.clamp(1, 8),
            b.clamp(1, 8),
            b.clamp(1, 8),
        ),
    };
    let soft_bound = if drain_rate_per_min > 0.0 {
        (drain_rate_per_min * (target_wait_secs as f64 / 60.0)).ceil() as u32
    } else {
        // No measured drain yet: bound by one target-window of provider
        // capacity — the honest pre-measurement statement.
        (profile.budget as u64 * target_wait_secs.max(60) / 60) as u32
    };
    AdmissionPlan {
        worker_threads: threads,
        extraction_concurrency: extraction,
        embed_max_async: embed,
        merge_max_async: merge,
        vision_jobs: vision,
        queue_soft_bound: soft_bound.max(1),
        tenant_lane_weight: 1,
    }
}

/// Queue target wait from env (default 600s).
pub fn queue_target_wait_secs_from_env() -> u64 {
    std::env::var(QUEUE_TARGET_WAIT_SECS_ENV)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_QUEUE_TARGET_WAIT_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::config::{
        DEFAULT_MAX_CONCURRENT_EXTRACTIONS, LOCAL_MAX_CONCURRENT_EXTRACTIONS,
        LOCAL_MAX_INGEST_TASKS_PER_TENANT_CAP, LOCAL_WORKER_THREADS_CAP,
    };

    /// LAW-Q1: the plan reproduces today's verified defaults exactly — the
    /// legacy resolvers' constants cannot drift from the SSOT without failing.
    #[test]
    fn contract_spec091_admission_resolver_derives_all() {
        // Local (B=1): extraction 1, threads 2, embed 1, merge 1, vision 1.
        let local = resolve(&ProviderProfile::LOCAL_DEFAULT, 2.0, 600);
        assert_eq!(
            local.extraction_concurrency,
            LOCAL_MAX_CONCURRENT_EXTRACTIONS
        );
        assert_eq!(local.worker_threads, 2);
        assert!(local.worker_threads <= LOCAL_WORKER_THREADS_CAP);
        assert_eq!(local.embed_max_async, 1);
        assert_eq!(local.merge_max_async, 1);
        assert_eq!(local.vision_jobs, 1);

        // Cloud (B=8): extraction 16, threads 16, embed 8, merge 8, vision 8.
        let cloud = resolve(&ProviderProfile::CLOUD_DEFAULT, 30.0, 600);
        assert_eq!(
            cloud.extraction_concurrency,
            DEFAULT_MAX_CONCURRENT_EXTRACTIONS
        );
        assert_eq!(cloud.worker_threads, 16);
        assert_eq!(cloud.embed_max_async, 8);
        assert_eq!(cloud.merge_max_async, 8);
        assert_eq!(cloud.vision_jobs, 8);

        // Tenant lane weight is equal-share DRR.
        assert_eq!(local.tenant_lane_weight, 1);
        assert_eq!(cloud.tenant_lane_weight, 1);
        // Local ingest cap constant agrees with the local plan's extraction.
        assert_eq!(
            LOCAL_MAX_INGEST_TASKS_PER_TENANT_CAP,
            local.extraction_concurrency
        );
    }

    /// Queue soft bound is Little's Law: λ̂ × target wait — never a constant.
    #[test]
    fn queue_soft_bound_is_littles_law() {
        // λ̂ = 2 tasks/min, target 600s = 10 min → bound 20.
        let plan = resolve(&ProviderProfile::LOCAL_DEFAULT, 2.0, 600);
        assert_eq!(plan.queue_soft_bound, 20);
        // λ̂ = 30/min, same window → 300.
        let plan = resolve(&ProviderProfile::CLOUD_DEFAULT, 30.0, 600);
        assert_eq!(plan.queue_soft_bound, 300);
        // No history → capacity × window fallback, never zero.
        let plan = resolve(&ProviderProfile::LOCAL_DEFAULT, 0.0, 600);
        assert!(plan.queue_soft_bound >= 1);
    }

    /// Scaling: a bigger local budget (multi-GPU opt-out) raises local numbers
    /// but stays under local caps; cloud oversubscribes for latency-hiding.
    #[test]
    fn scaling_respects_kind() {
        let local_big = resolve(&ProviderProfile::LOCAL_DEFAULT.with_budget(4), 4.0, 600);
        assert_eq!(local_big.extraction_concurrency, 2); // compute-bound cap
        assert_eq!(local_big.worker_threads, 4);
        let cloud_big = resolve(&ProviderProfile::CLOUD_DEFAULT.with_budget(16), 60.0, 600);
        assert_eq!(cloud_big.extraction_concurrency, 16); // latency cap
        assert_eq!(cloud_big.worker_threads, 32);
    }
}
