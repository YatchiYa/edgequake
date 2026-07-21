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

    /// Atomically swap a provisional track id for the durable task track id.
    ///
    /// Returns `Some(existing)` when another wipe already owns the workspace.
    /// Never leaves the slot empty between release and re-register.
    pub fn replace_track_id(
        &self,
        workspace_id: Uuid,
        expected_provisional: &str,
        new_track_id: &str,
    ) -> Option<String> {
        let mut map = self
            .slots
            .lock()
            .expect("workspace wipe admission registry lock");
        match map.get(&workspace_id) {
            Some(current) if current == expected_provisional || current == new_track_id => {
                map.insert(workspace_id, new_track_id.to_string());
                None
            }
            Some(current) => Some(current.clone()),
            None => {
                map.insert(workspace_id, new_track_id.to_string());
                None
            }
        }
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
        // Prefer the first Pending/Processing wipe; parse wipe_track_id when present.
        if let Some(track_id) = list.tasks.into_iter().find_map(|task| {
            serde_json::from_value::<WorkspaceWipeTaskData>(task.task_data.clone())
                .ok()
                .map(|data| data.wipe_track_id)
                .or(Some(task.track_id))
        }) {
            return Some(track_id);
        }
    }
    None
}

/// True when a durable wipe is Pending/Processing, or a process-local admit slot is held.
///
/// Process-local slots must count as in-flight so concurrent upload/reprocess cannot
/// race the window between `try_register` and durable `enqueue_task`.
pub async fn workspace_wipe_in_flight(state: &AppState, workspace_id: Uuid) -> bool {
    if find_active_workspace_wipe_track_id(state, workspace_id)
        .await
        .is_some()
    {
        return true;
    }
    // Local admit slot is authoritative until release (success or enqueue failure).
    state.tasks.wipe_admission.get(workspace_id).is_some()
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

    #[test]
    fn replace_track_id_swaps_without_gap() {
        let reg = WorkspaceWipeAdmissionRegistry::default();
        let ws = Uuid::new_v4();
        assert!(reg.try_register(ws, "provisional").is_none());
        assert!(reg
            .replace_track_id(ws, "provisional", "durable-track")
            .is_none());
        assert_eq!(reg.get(ws).as_deref(), Some("durable-track"));
    }

    #[test]
    fn replace_track_id_returns_conflict() {
        let reg = WorkspaceWipeAdmissionRegistry::default();
        let ws = Uuid::new_v4();
        assert!(reg.try_register(ws, "other-wipe").is_none());
        assert_eq!(
            reg.replace_track_id(ws, "provisional", "durable-track")
                .as_deref(),
            Some("other-wipe")
        );
    }
}
