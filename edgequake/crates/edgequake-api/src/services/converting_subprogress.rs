//! Post-OCR converting sub-step progress (SPEC-048 / Pass B VLM analyze).
//!
//! SSOT for human-readable `stage_message` strings and `stage_progress` fractions
//! during figure/chart Vision LLM analysis. Keeps pdf_processing, multimodal
//! analyzer, and progress_facade parsing aligned (DRY).

use std::sync::Arc;

/// Callback invoked when a converting sub-step advances (typically KV metadata patch).
pub type ConvertingSubstepReporter = Arc<dyn Fn(String, f64) + Send + Sync>;

/// User-visible message while Vision LLM analyzes extracted figure/chart images.
pub fn vision_figure_analyze_message(completed: usize, total: usize) -> String {
    if total == 0 {
        return "Analyzing figures with Vision LLM…".to_string();
    }
    format!("Analyzing figures with Vision LLM — figure {completed}/{total}")
}

/// Map figure analyze completion into the converting stage band (0.98–0.995).
pub fn vision_figure_analyze_progress_01(completed: usize, total: usize) -> f64 {
    const BASE: f64 = 0.98;
    const SPAN: f64 = 0.015;
    if total == 0 {
        return BASE;
    }
    (BASE + SPAN * (completed as f64 / total as f64)).min(0.995)
}

/// Throttle KV writes: first, every 3rd, and last — or every step when total ≤ 5.
pub fn should_emit_substep_milestone(completed: usize, total: usize) -> bool {
    if total == 0 {
        return completed == 1;
    }
    completed == 1 || completed == total || completed.is_multiple_of(3) || total <= 5
}

/// Emit a converting sub-step if a reporter is configured.
pub fn report_vision_figure_analyze(
    reporter: Option<&ConvertingSubstepReporter>,
    completed: usize,
    total: usize,
) {
    if let Some(hook) = reporter {
        if should_emit_substep_milestone(completed, total) {
            hook(
                vision_figure_analyze_message(completed, total),
                vision_figure_analyze_progress_01(completed, total),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn vision_figure_message_includes_counts() {
        assert_eq!(
            vision_figure_analyze_message(3, 12),
            "Analyzing figures with Vision LLM — figure 3/12"
        );
    }

    #[test]
    fn vision_figure_progress_monotonic_in_band() {
        assert!(vision_figure_analyze_progress_01(0, 10) >= 0.98);
        assert!(vision_figure_analyze_progress_01(10, 10) <= 0.995);
        assert!(
            vision_figure_analyze_progress_01(5, 10) > vision_figure_analyze_progress_01(2, 10)
        );
    }

    #[test]
    fn milestone_throttles_large_runs() {
        assert!(should_emit_substep_milestone(1, 20));
        assert!(should_emit_substep_milestone(3, 20));
        assert!(!should_emit_substep_milestone(2, 20));
        assert!(should_emit_substep_milestone(20, 20));
    }

    #[test]
    fn reporter_called_on_milestones() {
        let calls = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&calls);
        let reporter: ConvertingSubstepReporter = Arc::new(move |msg, p| {
            c.fetch_add(1, Ordering::Relaxed);
            assert!(msg.contains("figure 3/9"));
            assert!(p > 0.98);
        });
        report_vision_figure_analyze(Some(&reporter), 3, 9);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
