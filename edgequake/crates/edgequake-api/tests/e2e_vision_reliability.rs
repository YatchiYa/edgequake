//! E2E-style reliability gates for Vision PDF ingestion edge cases.
//!
//! These tests exercise the fail-closed + progress-aware breaker contract
//! end-to-end through TaskFailureInfo → Task::mark_failed_with_details without
//! requiring a live VLM (deterministic, not flaky).

use edgequake_api::services::{
    annotate_timeout_progress, evaluate_vision_watchdog, VisionWatchdogAbort,
};
use edgequake_pdf::{should_fallback_to_edgeparse, PdfParserBackend, VisionFailureKind};
use edgequake_tasks::{Task, TaskFailureInfo, TaskType};
use std::time::Duration;
use uuid::Uuid;

fn sample_task() -> Task {
    Task::new(
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        TaskType::PdfProcessing,
        serde_json::json!({ "pdf_id": "d0aa6d34-95c8-4d1c-bdab-66ce66d52e34" }),
    )
}

#[test]
fn e2e_progressing_stall_does_not_permanently_fail() {
    let mut task = sample_task();
    task.max_retries = 10;

    // Simulate three progressing stalls (pages completed each attempt).
    for i in 1..=5 {
        let raw = VisionWatchdogAbort::Stall {
            stall_secs: 300,
            idle_secs: 301,
        }
        .as_timeout_message("pdf-x", "mistral");
        let annotated = annotate_timeout_progress(raw, true);
        let err = TaskFailureInfo::from_processing_error(annotated);
        assert!(err.made_progress, "attempt {i} must parse progress marker");
        task.mark_failed_with_details(err);
        assert!(
            !task.circuit_breaker_tripped,
            "progressing stalls must never trip breaker (attempt {i})"
        );
        assert!(task.can_retry() || task.retry_count < task.max_retries);
        // Re-arm as Pending like the worker does before retry.
        task.status = edgequake_tasks::TaskStatus::Pending;
        task.completed_at = None;
    }
    assert_eq!(task.consecutive_timeout_failures, 0);
}

#[test]
fn e2e_hung_provider_no_progress_trips_breaker_fail_closed() {
    let mut task = sample_task();
    task.max_retries = 10;

    for _ in 0..3 {
        let raw = VisionWatchdogAbort::Stall {
            stall_secs: 300,
            idle_secs: 301,
        }
        .as_timeout_message("pdf-x", "mistral");
        let annotated = annotate_timeout_progress(raw, false);
        let err = TaskFailureInfo::from_processing_error(annotated);
        assert!(!err.made_progress);
        task.mark_failed_with_details(err);
        task.status = edgequake_tasks::TaskStatus::Pending;
        task.completed_at = None;
    }

    assert!(task.circuit_breaker_tripped);
    assert!(!task.can_retry());

    // Explicit Vision must stay fail-closed (no silent EdgeParse).
    assert!(!should_fallback_to_edgeparse(
        PdfParserBackend::Vision,
        VisionFailureKind::Timeout,
        true,
    ));
}

#[test]
fn e2e_watchdog_policy_matches_production_defaults() {
    // Progressing for hours is fine; idle 5 minutes is not.
    assert!(evaluate_vision_watchdog(
        Duration::from_secs(10_000),
        Duration::from_secs(86_400),
        Duration::from_secs(60),
        Duration::from_secs(300),
    )
    .is_none());

    assert!(matches!(
        evaluate_vision_watchdog(
            Duration::from_secs(400),
            Duration::from_secs(86_400),
            Duration::from_secs(300),
            Duration::from_secs(300),
        ),
        Some(VisionWatchdogAbort::Stall { .. })
    ));
}
