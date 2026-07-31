//! Capacity block SSOT — named binding layer when fairness parks a task.
//!
//! ## First principles
//!
//! Default product admission: provider/model pool is the ingest hard gate
//! (tenant ingest lane off). Opt-in tenant fair-share and lifecycle lanes still
//! stamp their own layer. The park reason must name **which** layer blocked.

use serde::{Deserialize, Serialize};

/// Which capacity layer blocked (or would block) admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "layer", rename_all = "snake_case")]
pub enum CapacityLayer {
    /// Noisy-neighbor / tenant fair-share lane.
    TenantFairShare { in_use: usize, max: usize },
    /// Global provider (and optional model) pool.
    ProviderModel {
        provider: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        in_use: usize,
        max: usize,
    },
}

impl CapacityLayer {
    /// Human-readable badge / stage message (INV-Q10 presentation SSOT).
    pub fn wait_message(&self) -> String {
        match self {
            Self::TenantFairShare { in_use, max } => {
                format!("Waiting for tenant fair-share ({in_use} of {max})")
            }
            Self::ProviderModel {
                provider,
                model,
                in_use,
                max,
            } => {
                let label = match model {
                    Some(m) if !m.is_empty() => format!("{provider}/{m}"),
                    _ => provider.clone(),
                };
                // Capitalize common local provider names for UI.
                let display = if label.eq_ignore_ascii_case("ollama")
                    || label.to_ascii_lowercase().starts_with("ollama/")
                {
                    capitalize_provider_label(&label)
                } else {
                    label
                };
                format!("Waiting for {display} capacity ({in_use} of {max})")
            }
        }
    }

    /// Compact reason string for pipeline `capacity_wait_reason`.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::TenantFairShare { .. } => "tenant_fair_share",
            Self::ProviderModel { .. } => "provider_model",
        }
    }
}

fn capitalize_provider_label(label: &str) -> String {
    let lower = label.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("ollama/") {
        format!("Ollama/{rest}")
    } else if lower == "ollama" {
        "Ollama".into()
    } else if let Some(rest) = lower.strip_prefix("lmstudio/") {
        format!("LM Studio/{rest}")
    } else if lower == "lmstudio" || lower == "lm-studio" || lower == "lm_studio" {
        "LM Studio".into()
    } else {
        label.to_string()
    }
}

/// Merge/clear helpers for [`crate::types::TaskProgress`].
pub mod progress_keys {
    /// JSON object key under task.progress (and flattened merge).
    pub const CAPACITY_BLOCK: &str = "capacity_block";
}

/// Stamp `capacity_block` onto task progress (park SSOT).
pub async fn stamp_capacity_block(
    storage: &dyn crate::storage::TaskStorage,
    track_id: &str,
    layer: &CapacityLayer,
) -> crate::error::TaskResult<()> {
    use crate::types::TaskProgress;

    let Some(task) = storage.get_task(track_id).await? else {
        return Ok(());
    };
    let wait = layer.wait_message();
    let mut progress = task.progress.unwrap_or(TaskProgress {
        current_step: wait.clone(),
        total_steps: 0,
        percent_complete: 0,
        chunk_progress: None,
        capacity_block: None,
    });
    progress.capacity_block = Some(layer.clone());
    // Prefer named wait copy when step is empty or still admission-idle.
    let step_l = progress.current_step.to_ascii_lowercase();
    if progress.current_step.is_empty()
        || matches!(
            step_l.as_str(),
            "queued" | "pending" | "waiting" | "uploading"
        )
        || step_l.contains("waiting for")
    {
        progress.current_step = wait;
    }
    storage.update_task_progress(track_id, &progress).await
}

/// Clear `capacity_block` from task progress (reclaim / clear hold).
pub async fn clear_capacity_block(
    storage: &dyn crate::storage::TaskStorage,
    track_id: &str,
) -> crate::error::TaskResult<()> {
    let Some(task) = storage.get_task(track_id).await? else {
        return Ok(());
    };
    let Some(mut progress) = task.progress else {
        return Ok(());
    };
    if progress.capacity_block.take().is_none() {
        return Ok(());
    }
    storage.update_task_progress(track_id, &progress).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_wait_message_names_layer() {
        let msg = CapacityLayer::TenantFairShare {
            in_use: 1,
            max: 1,
        }
        .wait_message();
        assert!(msg.contains("tenant fair-share"));
        assert!(msg.contains("1 of 1"));
    }

    #[test]
    fn provider_wait_message_includes_model() {
        let msg = CapacityLayer::ProviderModel {
            provider: "ollama".into(),
            model: Some("gemma3:latest".into()),
            in_use: 1,
            max: 1,
        }
        .wait_message();
        assert!(msg.contains("Ollama/gemma3:latest") || msg.contains("ollama/gemma3"));
        assert!(msg.contains("capacity"));
    }

    #[test]
    fn serde_roundtrip_tagged() {
        let layer = CapacityLayer::TenantFairShare {
            in_use: 2,
            max: 3,
        };
        let v = serde_json::to_value(&layer).unwrap();
        assert_eq!(v["layer"], "tenant_fair_share");
        let back: CapacityLayer = serde_json::from_value(v).unwrap();
        assert_eq!(back, layer);
    }

    #[tokio::test]
    async fn stamp_and_clear_capacity_block_on_progress() {
        use crate::memory::MemoryTaskStorage;
        use crate::storage::TaskStorage;
        use crate::types::{Task, TaskType};
        use std::sync::Arc;
        use uuid::Uuid;

        let storage = Arc::new(MemoryTaskStorage::new());
        let task = Task::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();

        let layer = CapacityLayer::TenantFairShare {
            in_use: 1,
            max: 1,
        };
        stamp_capacity_block(storage.as_ref(), &track_id, &layer)
            .await
            .unwrap();
        let stamped = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(
            stamped.progress.as_ref().and_then(|p| p.capacity_block.clone()),
            Some(layer.clone())
        );

        clear_capacity_block(storage.as_ref(), &track_id)
            .await
            .unwrap();
        let cleared = storage.get_task(&track_id).await.unwrap().unwrap();
        assert!(cleared
            .progress
            .as_ref()
            .and_then(|p| p.capacity_block.as_ref())
            .is_none());

        // clear_fairness_hold also clears capacity_block
        stamp_capacity_block(storage.as_ref(), &track_id, &layer)
            .await
            .unwrap();
        storage
            .mark_fairness_hold(&track_id, std::time::Duration::from_secs(30))
            .await
            .unwrap();
        storage.clear_fairness_hold(&track_id).await.unwrap();
        let after_hold = storage.get_task(&track_id).await.unwrap().unwrap();
        assert!(after_hold
            .progress
            .as_ref()
            .and_then(|p| p.capacity_block.as_ref())
            .is_none());
    }
}
