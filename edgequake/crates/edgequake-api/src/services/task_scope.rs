//! Scoped task access — DRY workspace isolation for v1 tasks and v2 jobs.

use edgequake_tasks::Task;

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

/// Load a task when it belongs to the requester's workspace (404 if cross-tenant).
pub async fn get_task_for_context(
    state: &AppState,
    track_id: &str,
    tenant_ctx: &TenantContext,
) -> ApiResult<Task> {
    let task = state
        .tasks
        .storage
        .get_task(track_id)
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to get task: {e}")))?
        .ok_or_else(|| ApiError::NotFound(format!("Task not found: {track_id}")))?;

    if let Some(ctx_workspace_id) = tenant_ctx.workspace_id_uuid() {
        if task.workspace_id != ctx_workspace_id {
            return Err(ApiError::NotFound(format!("Task not found: {track_id}")));
        }
    }

    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_tasks::{Task, TaskType};
    use uuid::Uuid;

    #[tokio::test]
    async fn e2e_cancel_foreign_track_id_404() {
        let state = AppState::test_state();
        let ws_owner = Uuid::new_v4();
        let ws_other = Uuid::new_v4();
        let task = Task::new(
            Uuid::new_v4(),
            ws_owner,
            TaskType::Insert,
            serde_json::json!({}),
        );
        let track_id = task.track_id.clone();
        state
            .tasks
            .storage
            .create_task(&task)
            .await
            .expect("create task");

        let foreign = TenantContext {
            tenant_id: Some(Uuid::new_v4().to_string()),
            workspace_id: Some(ws_other.to_string()),
            user_id: None,
        };
        let err = get_task_for_context(&state, &track_id, &foreign)
            .await
            .expect_err("foreign workspace must 404");
        assert!(
            matches!(err, ApiError::NotFound(_)),
            "expected NotFound, got {err:?}"
        );
    }

    #[tokio::test]
    async fn e2e_pdf_progress_foreign_404() {
        // Same ownership gate used by PDF progress / WS filtered progress.
        let state = AppState::test_state();
        let ws_owner = Uuid::new_v4();
        let task = Task::new(
            Uuid::new_v4(),
            ws_owner,
            TaskType::Upload,
            serde_json::json!({"pdf_id": Uuid::new_v4().to_string()}),
        );
        let track_id = task.track_id.clone();
        state
            .tasks
            .storage
            .create_task(&task)
            .await
            .expect("create task");

        let foreign = TenantContext {
            tenant_id: None,
            workspace_id: Some(Uuid::new_v4().to_string()),
            user_id: None,
        };
        let err = get_task_for_context(&state, &track_id, &foreign)
            .await
            .expect_err("foreign PDF progress must 404");
        assert!(matches!(err, ApiError::NotFound(_)));
    }
}
