//! Scoped task access — DRY workspace isolation for v1 tasks and v2 jobs.

use edgequake_tasks::{Task, TaskFilter};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

/// Build the tenant/workspace portion of a task filter.
///
/// Explicit query values take precedence for admin/debug clients; normal
/// requests inherit the authenticated header context.
pub fn task_filter_for_scope(
    tenant_ctx: &TenantContext,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> TaskFilter {
    TaskFilter {
        tenant_id: tenant_id
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .or_else(|| tenant_ctx.tenant_id_uuid()),
        workspace_id: workspace_id
            .and_then(|value| uuid::Uuid::parse_str(value).ok())
            .or_else(|| tenant_ctx.workspace_id_uuid()),
        ..TaskFilter::default()
    }
}

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

    #[test]
    fn task_filter_scope_uses_context_and_explicit_override() {
        let context_tenant = Uuid::new_v4();
        let context_workspace = Uuid::new_v4();
        let explicit_workspace = Uuid::new_v4();
        let context = TenantContext {
            tenant_id: Some(context_tenant.to_string()),
            workspace_id: Some(context_workspace.to_string()),
            user_id: None,
        };

        let inherited = task_filter_for_scope(&context, None, None);
        assert_eq!(inherited.tenant_id, Some(context_tenant));
        assert_eq!(inherited.workspace_id, Some(context_workspace));

        let overridden =
            task_filter_for_scope(&context, None, Some(&explicit_workspace.to_string()));
        assert_eq!(overridden.tenant_id, Some(context_tenant));
        assert_eq!(overridden.workspace_id, Some(explicit_workspace));
    }

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
