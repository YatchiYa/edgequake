//! Process-local + durable single-flight for workspace wipe-all (issue #309).
//!
//! Closes the race where two concurrent DELETE /documents pass the DB active-task
//! check before either `WorkspaceWipe` row exists. Registry entries self-heal when
//! storage shows no active wipe task.

use std::collections::HashMap;
use std::sync::Mutex;

use edgequake_tasks::{Pagination, TaskFilter, TaskStatus, TaskType, WorkspaceWipeTaskData};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Default)]
pub struct WorkspaceWipeAdmissionRegistry {
    slots: Mutex<HashMap<Uuid, String>>,
}

impl WorkspaceWipeAdmissionRegistry {
    /// Register `wipe_track_id` for `workspace_id` or return an existing holder.
    pub fn try_register(&self, workspace_id: Uuid, wipe_track_id: &str) -> Option<String> {
        let mut map = self
            .slots
            .lock()
            .expect("workspace wipe admission registry lock");
        if let Some(existing) = map.get(&workspace_id) {
            if existing != wipe_track_id {
                return Some(existing.clone());
            }
            return None;
        }
        map.insert(workspace_id, wipe_track_id.to_string());
        None
    }

    pub fn get(&self, workspace_id: Uuid) -> Option<String> {
        self.slots
            .lock()
            .expect("workspace wipe admission registry lock")
            .get(&workspace_id)
            .cloned()
    }

    pub fn release(&self, workspace_id: Uuid) {
        self.slots
            .lock()
            .expect("workspace wipe admission registry lock")
            .remove(&workspace_id);
    }
}

/// Look up an active (Pending/Processing) WorkspaceWipe for the workspace.
pub async fn find_active_workspace_wipe_track_id(
    state: &AppState,
    workspace_id: Uuid,
) -> Option<String> {
    for status in [TaskStatus::Pending, TaskStatus::Processing] {
        let list = state
            .tasks
            .storage
            .list_tasks(
                TaskFilter {
                    workspace_id: Some(workspace_id),
                    status: Some(status),
                    task_type: Some(TaskType::WorkspaceWipe),
                    ..Default::default()
                },
                Pagination {
                    page: 1,
                    page_size: 20,
                    ..Default::default()
                },
            )
            .await
            .ok()?;
        for task in list.tasks {
            if let Ok(data) =
                serde_json::from_value::<WorkspaceWipeTaskData>(task.task_data.clone())
            {
                return Some(data.wipe_track_id);
            }
            return Some(task.track_id);
        }
    }
    None
}

/// True when a durable wipe is Pending/Processing for this workspace.
pub async fn workspace_wipe_in_flight(state: &AppState, workspace_id: Uuid) -> bool {
    let active = find_active_workspace_wipe_track_id(state, workspace_id).await;
    if active.is_some() {
        return true;
    }
    // Self-heal stale process-local slot when durable storage has no active wipe.
    if state.tasks.wipe_admission.get(workspace_id).is_some() {
        state.tasks.wipe_admission.release(workspace_id);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_register_returns_existing_on_conflict() {
        let reg = WorkspaceWipeAdmissionRegistry::default();
        let ws = Uuid::new_v4();
        assert!(reg.try_register(ws, "wipe-a").is_none());
        assert_eq!(reg.try_register(ws, "wipe-b").as_deref(), Some("wipe-a"));
    }

    #[test]
    fn release_allows_new_registration() {
        let reg = WorkspaceWipeAdmissionRegistry::default();
        let ws = Uuid::new_v4();
        reg.try_register(ws, "wipe-a");
        reg.release(ws);
        assert!(reg.try_register(ws, "wipe-b").is_none());
    }
}
