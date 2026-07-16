//! PDF progress identity contract (SPEC-054 / GitHub #300).
//!
//! # First principles
//!
//! An admitted PDF job has **one** progress-store key: the server-generated
//! task id (`pdf-<uuid>`). Optional client `track_id` is batch/request
//! correlation only and MUST NOT seed or update the progress store.
//!
//! This module is the SSOT for that invariant (DRY + SRP).

use tracing::info;

use crate::state::AppState;

/// Canonical progress-store key for an admitted PDF job.
///
/// Identity: `response.task_id` == queued task track_id == progress key.
#[inline]
pub(crate) fn pdf_progress_track_id(task_id: &str) -> &str {
    task_id
}

/// Whether a client batch id differs from the server task id.
#[inline]
pub(crate) fn client_batch_differs_from_task(
    task_id: &str,
    client_batch_track_id: Option<&str>,
) -> bool {
    client_batch_track_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some_and(|batch| batch != task_id)
}

/// Seed PDF progress under the server task id only.
///
/// Never creates a progress skeleton for the client batch id — that caused
/// GitHub #300 (UI subscribed to a key the worker never updates).
/// Seed PDF progress under the server task id (upload + reprocess share this SSOT).
pub(crate) async fn seed_pdf_job_progress(
    state: &AppState,
    task_id: &str,
    pdf_id: &str,
    filename: &str,
    client_batch_track_id: Option<&str>,
) {
    let progress_key = pdf_progress_track_id(task_id);
    if client_batch_differs_from_task(task_id, client_batch_track_id) {
        info!(
            task_id = %progress_key,
            client_track_id = %client_batch_track_id.unwrap_or(""),
            "SPEC-054/gh#300: PDF progress keyed by task_id (client track_id is batch correlation only)"
        );
    }
    state
        .tasks
        .pipeline_state
        .start_pdf_progress(progress_key, pdf_id, filename)
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_key_is_task_id() {
        assert_eq!(pdf_progress_track_id("pdf-abc"), "pdf-abc");
    }

    #[test]
    fn client_batch_differs_when_present_and_distinct() {
        assert!(client_batch_differs_from_task(
            "pdf-1",
            Some("upload_batch_1")
        ));
        assert!(!client_batch_differs_from_task("pdf-1", Some("pdf-1")));
        assert!(!client_batch_differs_from_task("pdf-1", Some("")));
        assert!(!client_batch_differs_from_task("pdf-1", Some("   ")));
        assert!(!client_batch_differs_from_task("pdf-1", None));
    }
}
