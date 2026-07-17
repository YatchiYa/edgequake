//! SPEC-057 P3: dual WorkerPool never double-processes a single Pending task.
//!
//! Two pools share one `TaskStorage` + one wake queue; a counting processor
//! proves `process` runs exactly once (claim_next + lease is SSOT).

use async_trait::async_trait;
use edgequake_tasks::{
    memory::MemoryTaskStorage, queue::ChannelTaskQueue, Task, TaskProcessor, TaskStatus, TaskType,
    WorkerPool, WorkerPoolConfig,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct CountingProcessor {
    calls: AtomicU64,
}

#[async_trait]
impl TaskProcessor for CountingProcessor {
    async fn process(
        &self,
        _task: &mut Task,
        _cancel_token: CancellationToken,
    ) -> edgequake_tasks::TaskResult<serde_json::Value> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        // Hold briefly so the second pool's claim races against a live lease.
        tokio::time::sleep(Duration::from_millis(80)).await;
        Ok(serde_json::json!({ "ok": true }))
    }
}

#[tokio::test]
async fn dual_worker_pool_processes_pending_task_exactly_once() {
    let storage: Arc<dyn edgequake_tasks::TaskStorage> = Arc::new(MemoryTaskStorage::new());
    let queue: Arc<dyn edgequake_tasks::TaskQueue> = Arc::new(ChannelTaskQueue::new(32));
    let counter = Arc::new(CountingProcessor {
        calls: AtomicU64::new(0),
    });
    let processor: edgequake_tasks::SharedTaskProcessor = counter.clone();

    let config = WorkerPoolConfig {
        num_workers: 2,
        auto_retry: false,
        initial_retry_delay_ms: 50,
        max_retry_delay_ms: 200,
        backoff_multiplier: 2.0,
        max_tasks_per_tenant: 0,
        processing_timeout_secs: 30,
    };

    let mut pool_a = WorkerPool::new(
        config.clone(),
        Arc::clone(&queue),
        Arc::clone(&storage),
        Arc::clone(&processor),
    );
    let mut pool_b = WorkerPool::new(
        config,
        Arc::clone(&queue),
        Arc::clone(&storage),
        Arc::clone(&processor),
    );
    pool_a.start();
    pool_b.start();

    let task = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "dual-pool-once" }),
    );
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();
    // Dual wake — both pools may observe the channel; claim must serialize.
    queue.send(task.clone()).await.unwrap();
    queue.send(task).await.unwrap();

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        if stored.status == TaskStatus::Indexed || stored.status == TaskStatus::Failed {
            break;
        }
        if tokio::time::Instant::now() > deadline {
            panic!(
                "task did not reach terminal status; status={:?}",
                stored.status
            );
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    assert_eq!(
        counter.calls.load(Ordering::SeqCst),
        1,
        "exactly one process() invocation across dual WorkerPools"
    );
    let final_task = storage.get_task(&track_id).await.unwrap().unwrap();
    assert_eq!(final_task.status, TaskStatus::Indexed);

    pool_a.shutdown().await;
    pool_b.shutdown().await;
}

#[test]
fn delivery_gate_rejects_local_with_replicas_gt_one() {
    use edgequake_tasks::{validate_delivery_for_replicas, TaskDeliveryMode};
    assert!(validate_delivery_for_replicas(TaskDeliveryMode::Local, 2).is_err());
    assert!(validate_delivery_for_replicas(TaskDeliveryMode::Bridged, 2).is_ok());
}
