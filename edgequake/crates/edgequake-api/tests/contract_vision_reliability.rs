//! Contract + unit tests for reliable Vision PDF ingestion.
//!
//! Covers: page-count SSOT wiring, stall-watchdog policy, durable checkpoints,
//! progress-aware timeout markers, and fail-closed explicit Vision (no silent
//! EdgeParse on stall when backend_explicit).

use edgequake_api::services::{
    annotate_timeout_progress, durable_vision_checkpoint_dir, evaluate_vision_watchdog,
    vision_stall_timeout_secs, VisionWatchdogAbort, DEFAULT_VISION_STALL_TIMEOUT_SECS,
};
use edgequake_pdf::{should_fallback_to_edgeparse, PdfParserBackend, VisionFailureKind};
use std::time::Duration;

#[test]
fn stall_watchdog_allows_long_progressing_runs() {
    // 2 hours wall clock with recent progress → continue
    assert!(evaluate_vision_watchdog(
        Duration::from_secs(7_200),
        Duration::from_secs(86_400),
        Duration::from_secs(5),
        Duration::from_secs(300),
    )
    .is_none());
}

#[test]
fn stall_watchdog_trips_only_when_idle() {
    let abort = evaluate_vision_watchdog(
        Duration::from_secs(400),
        Duration::from_secs(86_400),
        Duration::from_secs(301),
        Duration::from_secs(300),
    )
    .expect("must abort");
    assert!(matches!(abort, VisionWatchdogAbort::Stall { .. }));
}

#[test]
fn progress_marker_roundtrip() {
    let msg = annotate_timeout_progress(
        "Vision extraction stalled: no progress for 300s".into(),
        true,
    );
    assert!(msg.contains("[vision_progress=1]"));
    let msg0 = annotate_timeout_progress("hung".into(), false);
    assert!(msg0.contains("[vision_progress=0]"));
}

#[test]
fn durable_checkpoint_not_under_tmp_by_default() {
    let prev = std::env::var("EDGEQUAKE_CHECKPOINT_DIR").ok();
    let prev_data = std::env::var("EDGEQUAKE_DATA_DIR").ok();
    std::env::remove_var("EDGEQUAKE_CHECKPOINT_DIR");
    std::env::remove_var("EDGEQUAKE_DATA_DIR");
    let dir = durable_vision_checkpoint_dir("pdf-uuid-test");
    assert!(
        !dir.starts_with("/tmp/") && !dir.starts_with("/var/folders/"),
        "default checkpoint must be durable, got {dir}"
    );
    assert!(dir.contains("pdf-uuid-test"));
    if let Some(v) = prev {
        std::env::set_var("EDGEQUAKE_CHECKPOINT_DIR", v);
    }
    if let Some(v) = prev_data {
        std::env::set_var("EDGEQUAKE_DATA_DIR", v);
    }
}

#[test]
fn explicit_vision_stall_stays_fail_closed() {
    // Decision A: never silently degrade explicit Vision to EdgeParse.
    assert!(!should_fallback_to_edgeparse(
        PdfParserBackend::Vision,
        VisionFailureKind::Timeout,
        true, // backend_explicit
    ));
}

#[test]
fn implicit_vision_timeout_still_may_fallback() {
    assert!(should_fallback_to_edgeparse(
        PdfParserBackend::Vision,
        VisionFailureKind::Timeout,
        false,
    ));
}

#[test]
fn stall_timeout_env_floor() {
    let prev = std::env::var("EDGEQUAKE_VISION_STALL_TIMEOUT_SECS").ok();
    std::env::set_var("EDGEQUAKE_VISION_STALL_TIMEOUT_SECS", "5");
    assert_eq!(vision_stall_timeout_secs(), 30); // floor
    std::env::remove_var("EDGEQUAKE_VISION_STALL_TIMEOUT_SECS");
    assert_eq!(
        vision_stall_timeout_secs(),
        DEFAULT_VISION_STALL_TIMEOUT_SECS
    );
    if let Some(v) = prev {
        std::env::set_var("EDGEQUAKE_VISION_STALL_TIMEOUT_SECS", v);
    }
}

#[test]
fn watchdog_abort_message_mentions_resume() {
    let msg = VisionWatchdogAbort::Stall {
        stall_secs: 300,
        idle_secs: 301,
    }
    .as_timeout_message("pdf-1", "mistral");
    assert!(msg.contains("preserved for resume"));
    assert!(msg.contains("mistral"));
}
