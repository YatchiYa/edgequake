//! External task delivery for horizontal worker scale (SPEC-026 Phase 4 P-12).
//!
//! SPEC-057 P1: Postgres `TaskStorage` is the **delivery SSOT** (`claim_next`).
//! Channel / NOTIFY payloads are **wake-only** — workers ignore the body and
//! always authorize work via claim + lease.

mod bridge;
mod mode;
mod notifier;
mod storage_hydrating;

pub use bridge::BridgedTaskQueue;
pub use mode::{
    delivery_mode_from_env, is_multi_replica_deployment, parse_delivery_mode, replicas_from_env,
    validate_delivery_for_replicas, TaskDeliveryMode, REPLICAS_ENV,
};
pub use notifier::{ChannelTaskNotifier, NoopTaskNotifier, SharedTaskNotifier, TaskNotifier};
pub use storage_hydrating::StorageHydratingTaskQueue;

use crate::error::{TaskError, TaskResult};
use crate::queue::SharedTaskQueue;
use crate::storage::SharedTaskStorage;
use crate::types::Task;
use tracing::warn;

/// Enqueue path: persist then **wake** per mode (channel payload is not authoritative).
///
/// SPEC-132 / LAW-132-2 / F-091-19: after durable persist, wake uses `try_send` so a
/// full in-process channel never hangs the HTTP admit handler. Missed wakes are
/// recovered by claim_next + startup/periodic hydrate (SPEC-057).
pub async fn enqueue_with_delivery(
    storage: &SharedTaskStorage,
    queue: &SharedTaskQueue,
    notifier: &dyn TaskNotifier,
    mode: TaskDeliveryMode,
    task: Task,
) -> TaskResult<()> {
    let track_id = task.track_id.clone();
    storage.create_task(&task).await?;
    match mode {
        TaskDeliveryMode::Local | TaskDeliveryMode::Bridged => match queue.try_send(task).await {
            Ok(()) => {}
            Err(TaskError::QueueFull) => {
                warn!(
                    track_id = %track_id,
                    "SPEC-132: wake channel full after durable persist; \
                     task remains pending for claim_next/hydrate"
                );
            }
            Err(e) => return Err(e),
        },
        TaskDeliveryMode::NotifyOnly => notifier.notify(&track_id).await?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryTaskStorage;
    use crate::queue::ChannelTaskQueue;
    use crate::types::TaskType;
    use std::sync::Arc;
    use uuid::Uuid;

    const TEST_TENANT: &str = "00000000-0000-0000-0000-000000000001";
    const TEST_WORKSPACE: &str = "00000000-0000-0000-0000-000000000002";

    fn test_ids() -> (Uuid, Uuid) {
        (
            Uuid::parse_str(TEST_TENANT).unwrap(),
            Uuid::parse_str(TEST_WORKSPACE).unwrap(),
        )
    }

    #[tokio::test]
    async fn local_delivery_sends_to_channel() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let queue: SharedTaskQueue = Arc::new(ChannelTaskQueue::new(8));
        let notifier = Arc::new(NoopTaskNotifier);
        let (tenant, workspace) = test_ids();
        let task = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));

        enqueue_with_delivery(
            &storage,
            &queue,
            notifier.as_ref(),
            TaskDeliveryMode::Local,
            task.clone(),
        )
        .await
        .unwrap();

        let received = queue.receive().await.unwrap();
        assert_eq!(received.track_id, task.track_id);
    }

    #[tokio::test]
    async fn bridged_delivery_notifies_and_queues() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let inner: SharedTaskQueue = Arc::new(ChannelTaskQueue::new(8));
        let notifier = Arc::new(ChannelTaskNotifier::new(8));
        let bridged: SharedTaskQueue = Arc::new(BridgedTaskQueue::new(
            Arc::clone(&inner),
            Arc::clone(&notifier) as SharedTaskNotifier,
        ));
        let (tenant, workspace) = test_ids();
        let task = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));

        let mut sub = notifier.subscribe();
        enqueue_with_delivery(
            &storage,
            &bridged,
            notifier.as_ref(),
            TaskDeliveryMode::Bridged,
            task.clone(),
        )
        .await
        .unwrap();

        let notified = sub.recv().await.unwrap();
        assert_eq!(notified, task.track_id);
        let received = inner.receive().await.unwrap();
        assert_eq!(received.track_id, task.track_id);
    }

    #[tokio::test]
    async fn notify_only_skips_local_send() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let queue: SharedTaskQueue = Arc::new(ChannelTaskQueue::new(8));
        let notifier = Arc::new(ChannelTaskNotifier::new(8));
        let (tenant, workspace) = test_ids();
        let task = Task::new(tenant, workspace, TaskType::Insert, serde_json::json!({}));
        let track_id = task.track_id.clone();

        let mut hydrating = StorageHydratingTaskQueue::new(Arc::clone(&storage), &notifier);
        enqueue_with_delivery(
            &storage,
            &queue,
            notifier.as_ref(),
            TaskDeliveryMode::NotifyOnly,
            task,
        )
        .await
        .unwrap();

        assert!(queue.try_receive().await.unwrap().is_none());
        let loaded = hydrating.receive_hydrated().await.unwrap();
        assert_eq!(loaded.track_id, track_id);
    }

    /// SPEC-132 EC-3: full wake channel must not fail admit after durable persist.
    #[tokio::test]
    async fn local_delivery_ok_when_wake_channel_full() {
        let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
        let queue: SharedTaskQueue = Arc::new(ChannelTaskQueue::new(1));
        let notifier = Arc::new(NoopTaskNotifier);
        let (tenant, workspace) = test_ids();

        let first = Task::new(
            tenant,
            workspace,
            TaskType::Insert,
            serde_json::json!({"n": 1}),
        );
        enqueue_with_delivery(
            &storage,
            &queue,
            notifier.as_ref(),
            TaskDeliveryMode::Local,
            first,
        )
        .await
        .unwrap();

        let second = Task::new(
            tenant,
            workspace,
            TaskType::Insert,
            serde_json::json!({"n": 2}),
        );
        let second_id = second.track_id.clone();
        enqueue_with_delivery(
            &storage,
            &queue,
            notifier.as_ref(),
            TaskDeliveryMode::Local,
            second,
        )
        .await
        .expect("admit must succeed when wake is full");

        let stored = storage.get_task(&second_id).await.unwrap();
        assert!(
            stored.is_some(),
            "second task must be durable after soft wake miss"
        );
        assert_eq!(queue.size().await.unwrap(), 1);
    }
}
