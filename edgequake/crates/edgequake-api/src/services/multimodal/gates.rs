//! Multimodal analyze gates (LightRAG `VLM_PROCESS_ENABLE` parity).

use super::super::vision_content::MultimodalProcessOptions;

/// Global kill-switch for inline image VLM analysis (LightRAG `VLM_PROCESS_ENABLE`).
///
/// Default **true** when unset — chart/figure Vision analyze runs with `process_options=i`
/// unless explicitly disabled (`VLM_PROCESS_ENABLE=false`).
pub fn vlm_process_enabled() -> bool {
    match std::env::var("VLM_PROCESS_ENABLE")
        .ok()
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        None => true,
        Some("0") | Some("false") | Some("no") | Some("off") => false,
        Some(_) => true,
    }
}

/// Failure handling when required analyze fails (LightRAG hard-fail vs EdgeQuake degraded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultimodalFailMode {
    /// Skip analysis, keep placeholders (EdgeQuake ops extension).
    Degraded,
    /// Propagate failure to document status (LightRAG default semantics).
    Strict,
}

impl MultimodalFailMode {
    /// Default **strict** when unset (LightRAG parity). Set `EDGEQUAKE_MULTIMODAL_FAIL_MODE=degraded` for ops-friendly mode.
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_MULTIMODAL_FAIL_MODE")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
        {
            Some(ref s) if s == "degraded" => Self::Degraded,
            _ => Self::Strict,
        }
    }

    /// SPEC-047 P1c: single policy for PDF stage + reanalyze (SOLID-O / DRY).
    pub fn should_abort_on_hard_error(self) -> bool {
        matches!(self, Self::Strict)
    }
}

/// Whether a multimodal hard_error should fail the caller (SSOT).
pub fn should_abort_multimodal_hard_error(hard_error: Option<&str>) -> bool {
    hard_error.is_some() && MultimodalFailMode::from_env().should_abort_on_hard_error()
}

/// Whether inline image analysis should run for the given per-document flags.
pub fn should_run_image_analysis(opts: &MultimodalProcessOptions) -> bool {
    vlm_process_enabled() && opts.images
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn vlm_process_enable_defaults_on() {
        std::env::remove_var("VLM_PROCESS_ENABLE");
        assert!(vlm_process_enabled());
    }

    #[test]
    #[serial_test::serial]
    fn vlm_process_enable_respects_true() {
        std::env::set_var("VLM_PROCESS_ENABLE", "true");
        assert!(vlm_process_enabled());
        std::env::remove_var("VLM_PROCESS_ENABLE");
    }

    #[test]
    #[serial_test::serial]
    fn vlm_process_enable_respects_false() {
        std::env::set_var("VLM_PROCESS_ENABLE", "false");
        assert!(!vlm_process_enabled());
        std::env::remove_var("VLM_PROCESS_ENABLE");
    }

    #[test]
    #[serial_test::serial]
    fn fail_mode_defaults_strict() {
        std::env::remove_var("EDGEQUAKE_MULTIMODAL_FAIL_MODE");
        assert_eq!(MultimodalFailMode::from_env(), MultimodalFailMode::Strict);
    }

    #[test]
    #[serial_test::serial]
    fn fail_mode_degraded_opt_in() {
        std::env::set_var("EDGEQUAKE_MULTIMODAL_FAIL_MODE", "degraded");
        assert_eq!(MultimodalFailMode::from_env(), MultimodalFailMode::Degraded);
        std::env::remove_var("EDGEQUAKE_MULTIMODAL_FAIL_MODE");
    }

    #[test]
    #[serial_test::serial]
    fn should_run_requires_i_flag_and_global_enable() {
        std::env::set_var("VLM_PROCESS_ENABLE", "true");
        let mut opts = MultimodalProcessOptions::default();
        assert!(!should_run_image_analysis(&opts));
        opts.images = true;
        assert!(should_run_image_analysis(&opts));
        std::env::remove_var("VLM_PROCESS_ENABLE");
    }
}
