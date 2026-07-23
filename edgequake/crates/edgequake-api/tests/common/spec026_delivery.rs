//! SPEC-026 Phase 4 task delivery E2E helpers (DRY SSOT).
//!
//! SPEC-057 P3: hydrating workers must authorize work via `claim_next` (never
//! bare `mark_processing`). Notify payloads are wake-only.

use edgequake_tasks::{
    delivery::StorageHydratingTaskQueue, task_lease_ttl_from_env, CancellationRegistry,
    ChannelTaskNotifier, SharedTaskProcessor, SharedTaskStorage,
};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Background workers for `notify_only` mode: wake from notifier, claim from SSOT.
pub fn spawn_hydrating_workers(
    storage: SharedTaskStorage,
    notifier: Arc<ChannelTaskNotifier>,
    processor: SharedTaskProcessor,
    cancellation_registry: CancellationRegistry,
    num_workers: usize,
) -> Vec<JoinHandle<()>> {
    (0..num_workers.max(1))
        .map(|worker_id| {
            let storage = Arc::clone(&storage);
            let notifier = Arc::clone(&notifier);
            let processor = Arc::clone(&processor);
            let cancel_registry = cancellation_registry.clone();
            tokio::spawn(async move {
                let mut hydrating =
                    StorageHydratingTaskQueue::new(storage.clone(), notifier.as_ref());
                let worker_name = format!("hydrating-{worker_id}");
                let lease_ttl = task_lease_ttl_from_env();
                loop {
                    // Wake only — ignore hydrated body; claim authorizes work.
                    let Ok(_wake) = hydrating.receive_hydrated().await else {
                        break;
                    };
                    let mut task = match storage.claim_next(&worker_name, lease_ttl).await {
                        Ok(Some(t)) => t,
                        Ok(None) => continue,
                        Err(_) => continue,
                    };
                    let cancel = cancel_registry.register(&task.track_id).await;
                    match processor.process(&mut task, cancel).await {
                        Ok(result) => {
                            let _ = task.mark_success(result);
                        }
                        Err(e) => task.mark_failed(e.to_string()),
                    }
                    cancel_registry.deregister(&task.track_id).await;
                    let _ = storage.update_task(&task).await;
                    tracing::debug!(
                        worker_id,
                        track_id = %task.track_id,
                        "hydrating worker finished task"
                    );
                }
            })
        })
        .collect()
}

/// Allow spawned hydrating workers to subscribe before the test enqueues work.
pub async fn wait_for_hydrating_workers_ready() {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}
