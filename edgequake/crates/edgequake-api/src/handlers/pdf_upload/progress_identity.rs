//! PDF progress identity contract (SPEC-054 / GitHub #300).
//!
//! # First principles
//!
//! An admitted PDF job has **one** progress-store key: the server-generated
//! task id (`pdf-<uuid>`). Optional client `track_id` is batch/request
//! correlation only and MUST NOT seed or update the progress store.
//!
//! This module is the SSOT for that invariant (DRY + SRP).
//!
//! Progress entries are in-memory. After a backend restart the entry is gone
//! while the durable task row may still be Pending/Processing — GET progress
//! must rehydrate a skeleton so the UI does not hard-404.

use edgequake_tasks::progress::{PdfUploadProgress, PhaseError, PipelinePhase};
use edgequake_tasks::{Task, TaskStatus};
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

/// Build a progress skeleton from a durable task when the in-memory map missed
/// (typical after `make stop` / deploy restart while convert is still live).
pub(crate) fn pdf_progress_from_task(task: &Task) -> Option<PdfUploadProgress> {
    let pdf_id = task
        .pdf_id()
        .map(|u| u.to_string())
        .or_else(|| {
            task.metadata
                .as_ref()
                .and_then(|m| m.get("pdf_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            task.task_data
                .get("pdf_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })?;
    let filename = task
        .task_data
        .get("filename")
        .and_then(|v| v.as_str())
        .or_else(|| {
            task.metadata
                .as_ref()
                .and_then(|m| m.get("filename"))
                .and_then(|v| v.as_str())
        })
        .or_else(|| task.task_data.get("file_source").and_then(|v| v.as_str()))
        .unwrap_or("document.pdf")
        .to_string();

    let mut progress = PdfUploadProgress::new(task.track_id.clone(), pdf_id, filename);
    progress.started_at = task.started_at.unwrap_or(task.created_at);
    progress.updated_at = task.updated_at;

    // Upload is always done once a durable task exists.
    progress.complete_phase(PipelinePhase::Upload);

    match task.status {
        TaskStatus::Pending | TaskStatus::Processing => {
            let step = task
                .progress
                .as_ref()
                .map(|p| p.current_step.as_str())
                .unwrap_or("pdf_conversion");
            let phase = infer_pipeline_phase(step);
            let total = task
                .progress
                .as_ref()
                .and_then(|p| p.chunk_progress.as_ref())
                .map(|c| c.total_chunks as usize)
                .unwrap_or(0)
                .max(1);
            let current = task
                .progress
                .as_ref()
                .and_then(|p| p.chunk_progress.as_ref())
                .map(|c| c.processed_chunks as usize)
                .unwrap_or(0);
            for p in PipelinePhase::all() {
                if p.index() < phase.index() {
                    progress.complete_phase(*p);
                }
            }
            progress.start_phase(phase, total);
            let msg = task
                .progress
                .as_ref()
                .map(|p| format!("Resuming — {}", p.current_step))
                .unwrap_or_else(|| "Resuming PDF processing after server restart…".to_string());
            progress.update_phase(phase, current, msg);
            if let Some(p) = &task.progress {
                progress.overall_percentage = f32::from(p.percent_complete);
            }
        }
        TaskStatus::Indexed => {
            for p in PipelinePhase::all() {
                progress.complete_phase(*p);
            }
            progress.is_complete = true;
            progress.completed_at = task.completed_at;
            progress.overall_percentage = 100.0;
        }
        TaskStatus::Failed | TaskStatus::Cancelled => {
            let phase = task
                .progress
                .as_ref()
                .map(|p| infer_pipeline_phase(&p.current_step))
                .unwrap_or(PipelinePhase::PdfConversion);
            let err = PhaseError::new(
                task.error_message
                    .clone()
                    .unwrap_or_else(|| format!("Task {}", task.status)),
                "TASK_FAILED",
                false,
                "Check the document status or retry the upload.",
            );
            progress.fail_phase(phase, err);
            progress.is_failed = true;
            progress.completed_at = task.completed_at;
        }
    }

    Some(progress)
}

fn infer_pipeline_phase(step: &str) -> PipelinePhase {
    let s = step.to_ascii_lowercase();
    if s.contains("chunk") {
        PipelinePhase::Chunking
    } else if s.contains("embed") {
        PipelinePhase::Embedding
    } else if s.contains("extract") || s.contains("entity") || s.contains("glean") {
        PipelinePhase::Extraction
    } else if s.contains("graph") || s.contains("merge") || s.contains("persist") {
        PipelinePhase::GraphStorage
    } else if s.contains("upload") {
        PipelinePhase::Upload
    } else {
        PipelinePhase::PdfConversion
    }
}

/// Return live progress, rehydrating + reseeding from the durable task when the
/// in-memory map is empty (post-restart).
pub(crate) async fn get_or_rehydrate_pdf_progress(
    state: &AppState,
    track_id: &str,
    task: &Task,
) -> Option<PdfUploadProgress> {
    if let Some(existing) = state.tasks.pipeline_state.get_pdf_progress(track_id).await {
        return Some(existing);
    }

    let skeleton = pdf_progress_from_task(task)?;
    state
        .tasks
        .pipeline_state
        .put_pdf_progress(track_id, skeleton.clone())
        .await;
    info!(
        track_id = %track_id,
        status = %task.status,
        "PDF progress rehydrated from durable task after in-memory miss"
    );
    Some(skeleton)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_tasks::TaskType;
    use uuid::Uuid;

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

    #[test]
    fn rehydrate_processing_task_lands_on_conversion() {
        let pdf_id = Uuid::new_v4();
        let mut task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::PdfProcessing,
            serde_json::json!({
                "pdf_id": pdf_id.to_string(),
                "filename": "demo.pdf",
                "tenant_id": Uuid::nil().to_string(),
                "workspace_id": Uuid::nil().to_string(),
                "enable_vision": false,
                "vision_provider": "mock",
            }),
        );
        task.status = TaskStatus::Processing;
        task.progress = Some(edgequake_tasks::TaskProgress {
            current_step: "pdf_conversion".into(),
            total_steps: 5,
            percent_complete: 40,
            chunk_progress: None,
        });

        let p = pdf_progress_from_task(&task).expect("skeleton");
        assert!(!p.is_complete);
        assert!(!p.is_failed);
        assert_eq!(p.filename, "demo.pdf");
        assert!(p.phase(PipelinePhase::PdfConversion).is_some());
        assert!((p.overall_percentage - 40.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn rehydrate_seeds_empty_progress_map() {
        use crate::state::AppState;

        let state = AppState::test_state();
        let pdf_id = Uuid::new_v4();
        let mut task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::PdfProcessing,
            serde_json::json!({
                "pdf_id": pdf_id.to_string(),
                "filename": "resume.pdf",
            }),
        );
        task.status = TaskStatus::Processing;
        task.progress = Some(edgequake_tasks::TaskProgress {
            current_step: "pdf_conversion".into(),
            total_steps: 5,
            percent_complete: 25,
            chunk_progress: None,
        });
        let track_id = task.track_id.clone();

        assert!(
            state
                .tasks
                .pipeline_state
                .get_pdf_progress(&track_id)
                .await
                .is_none(),
            "map must start empty"
        );

        let first = get_or_rehydrate_pdf_progress(&state, &track_id, &task)
            .await
            .expect("rehydrate");
        assert_eq!(first.filename, "resume.pdf");
        assert!(!first.is_complete);

        let second = get_or_rehydrate_pdf_progress(&state, &track_id, &task)
            .await
            .expect("cached");
        assert_eq!(second.filename, "resume.pdf");
        assert!(state
            .tasks
            .pipeline_state
            .get_pdf_progress(&track_id)
            .await
            .is_some());
    }

    #[test]
    fn rehydrate_cancelled_task_is_failed_skeleton() {
        let pdf_id = Uuid::new_v4();
        let mut task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::PdfProcessing,
            serde_json::json!({
                "pdf_id": pdf_id.to_string(),
                "filename": "dead.pdf",
            }),
        );
        task.status = TaskStatus::Cancelled;
        task.error_message = Some("Task cancelled by user".into());

        let p = pdf_progress_from_task(&task).expect("skeleton");
        assert!(p.is_failed);
        assert!(!p.is_complete);
    }
}
