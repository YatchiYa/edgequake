//! Local multimodal Pass B profile (never-stuck ingest budgets).
//!
//! Cloud / mock keep full specialize quality. Local VLMs (Ollama, LM Studio)
//! get classify-only, figure caps, wall budgets, and fail-open defaults so
//! PDF ingest cannot stall on Converting for hours.

use std::time::Duration;

/// Default max figures analyzed on local VLM (rest keep Pass A placeholders).
pub const LOCAL_MM_MAX_FIGURES_DEFAULT: usize = 12;

/// Default Pass B wall-clock budget for local VLM (seconds).
pub const LOCAL_MM_PASS_B_TIMEOUT_SECS_DEFAULT: u64 = 600;

/// Soft per-figure estimate used in worker task timeout (seconds).
/// Matches [`crate::safety_limits::LOCAL_PASS_B_VISION_TIMEOUT_SECS`] (90s).
pub const LOCAL_MM_PER_FIGURE_SECS_DEFAULT: u64 = 90;

/// Resolved multimodal Pass B policy for the active vision provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalMmProfile {
    /// True for Ollama / LM Studio (not mock/cloud).
    pub is_local: bool,
    /// Skip specialize + dense chart retry (classify JSON → caption only).
    pub classify_only: bool,
    /// Max figures to analyze (`None` = unlimited).
    pub max_figures: Option<usize>,
    /// Wall-clock budget for the entire Pass B loop (`None` = no wall).
    pub pass_b_timeout: Option<Duration>,
    /// Prefer every-figure progress emissions.
    pub emit_every_figure: bool,
}

impl LocalMmProfile {
    /// Resolve Pass B profile from a vision provider name (+ env overrides).
    pub fn resolve(vision_provider: &str) -> Self {
        let is_local = is_local_vlm(vision_provider);
        if !is_local {
            return Self::cloud();
        }
        Self {
            is_local: true,
            classify_only: local_classify_only_enabled(),
            max_figures: max_figures_for_local(),
            pass_b_timeout: Some(Duration::from_secs(pass_b_timeout_secs_for_local())),
            emit_every_figure: true,
        }
    }

    /// Resolve using `EDGEQUAKE_VISION_PROVIDER` / `EDGEQUAKE_LLM_PROVIDER`.
    pub fn resolve_from_env() -> Self {
        let provider = std::env::var("EDGEQUAKE_VISION_PROVIDER")
            .or_else(|_| std::env::var("EDGEQUAKE_LLM_PROVIDER"))
            .unwrap_or_default();
        Self::resolve(&provider)
    }

    /// Cloud / mock defaults — unlimited figures, full specialize, no wall.
    pub fn cloud() -> Self {
        let max_figures = max_figures_override_unlimited_ok();
        Self {
            is_local: false,
            classify_only: false,
            max_figures,
            pass_b_timeout: pass_b_timeout_override_optional(),
            emit_every_figure: false,
        }
    }

    /// Figures that will actually be analyzed given discovered count.
    pub fn figures_to_analyze(self, discovered: usize) -> usize {
        match self.max_figures {
            Some(cap) => discovered.min(cap),
            None => discovered,
        }
    }

    /// Budget to add into worker task timeout for Pass B (local only).
    pub fn pass_b_task_budget_secs(self) -> u64 {
        if !self.is_local {
            return 0;
        }
        let max_f = self.max_figures.unwrap_or(LOCAL_MM_MAX_FIGURES_DEFAULT) as u64;
        let soft = max_f.saturating_mul(LOCAL_MM_PER_FIGURE_SECS_DEFAULT);
        let wall = self
            .pass_b_timeout
            .map(|d| d.as_secs())
            .unwrap_or(LOCAL_MM_PASS_B_TIMEOUT_SECS_DEFAULT);
        soft.min(wall)
    }
}

/// Local VLM for Pass B budgets (excludes mock — tests / cloud-parity).
pub fn is_local_vlm(provider_name: &str) -> bool {
    matches!(
        provider_name.trim().to_ascii_lowercase().as_str(),
        "ollama" | "lmstudio" | "lm-studio" | "lm_studio"
    )
}

fn local_classify_only_enabled() -> bool {
    match std::env::var("EDGEQUAKE_MM_LOCAL_CLASSIFY_ONLY") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            !matches!(t.as_str(), "0" | "false" | "no" | "off")
        }
        // Default on for local runtime; off under `cfg(test)` so specialize
        // contracts keep covering the cloud path when make exports ollama.
        Err(_) => !cfg!(test),
    }
}

fn max_figures_for_local() -> Option<usize> {
    match std::env::var("EDGEQUAKE_MM_MAX_FIGURES") {
        Ok(v) => {
            let n: usize = v.trim().parse().unwrap_or(LOCAL_MM_MAX_FIGURES_DEFAULT);
            if n == 0 {
                // 0 = unlimited even on local (ops escape hatch).
                None
            } else {
                Some(n)
            }
        }
        // Runtime default 12; tests must opt into the cap via env.
        Err(_) if cfg!(test) => None,
        Err(_) => Some(LOCAL_MM_MAX_FIGURES_DEFAULT),
    }
}

fn pass_b_timeout_secs_for_local() -> u64 {
    std::env::var("EDGEQUAKE_MM_PASS_B_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(LOCAL_MM_PASS_B_TIMEOUT_SECS_DEFAULT)
        .max(30)
}

/// Cloud: only apply cap when env explicitly sets a positive limit.
fn max_figures_override_unlimited_ok() -> Option<usize> {
    match std::env::var("EDGEQUAKE_MM_MAX_FIGURES") {
        Ok(v) => {
            let n: usize = v.trim().parse().unwrap_or(0);
            if n == 0 {
                None
            } else {
                Some(n)
            }
        }
        Err(_) => None,
    }
}

fn pass_b_timeout_override_optional() -> Option<Duration> {
    match std::env::var("EDGEQUAKE_MM_PASS_B_TIMEOUT_SECS") {
        Ok(v) => {
            let n: u64 = v.trim().parse().unwrap_or(0);
            if n == 0 {
                None
            } else {
                Some(Duration::from_secs(n.max(30)))
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn local_ollama_defaults() {
        std::env::remove_var("EDGEQUAKE_MM_MAX_FIGURES");
        std::env::remove_var("EDGEQUAKE_MM_PASS_B_TIMEOUT_SECS");
        std::env::set_var("EDGEQUAKE_MM_LOCAL_CLASSIFY_ONLY", "1");
        std::env::set_var("EDGEQUAKE_MM_MAX_FIGURES", "12");
        let p = LocalMmProfile::resolve("ollama");
        assert!(p.is_local);
        assert!(p.classify_only);
        assert_eq!(p.max_figures, Some(12));
        assert_eq!(
            p.pass_b_timeout,
            Some(Duration::from_secs(LOCAL_MM_PASS_B_TIMEOUT_SECS_DEFAULT))
        );
        assert!(p.emit_every_figure);
        assert_eq!(p.figures_to_analyze(46), 12);
        assert!(p.pass_b_task_budget_secs() <= LOCAL_MM_PASS_B_TIMEOUT_SECS_DEFAULT);
        std::env::remove_var("EDGEQUAKE_MM_LOCAL_CLASSIFY_ONLY");
        std::env::remove_var("EDGEQUAKE_MM_MAX_FIGURES");
    }

    #[test]
    #[serial_test::serial]
    fn cloud_and_mock_unlimited_full_quality() {
        std::env::remove_var("EDGEQUAKE_MM_MAX_FIGURES");
        std::env::remove_var("EDGEQUAKE_MM_PASS_B_TIMEOUT_SECS");
        for provider in ["openai", "mistral", "mock", ""] {
            let p = LocalMmProfile::resolve(provider);
            assert!(!p.is_local, "provider={provider}");
            assert!(!p.classify_only);
            assert_eq!(p.max_figures, None);
            assert_eq!(p.pass_b_timeout, None);
        }
    }

    #[test]
    #[serial_test::serial]
    fn env_can_disable_classify_only() {
        std::env::set_var("EDGEQUAKE_MM_LOCAL_CLASSIFY_ONLY", "0");
        let p = LocalMmProfile::resolve("ollama");
        assert!(!p.classify_only);
        std::env::remove_var("EDGEQUAKE_MM_LOCAL_CLASSIFY_ONLY");
    }

    #[test]
    #[serial_test::serial]
    fn env_zero_max_figures_means_unlimited_local() {
        std::env::set_var("EDGEQUAKE_MM_MAX_FIGURES", "0");
        let p = LocalMmProfile::resolve("ollama");
        assert_eq!(p.max_figures, None);
        assert_eq!(p.figures_to_analyze(1000), 1000);
        std::env::remove_var("EDGEQUAKE_MM_MAX_FIGURES");
    }
}
