//! SPEC-132 — Admit wake must not hang when the in-process channel is full.
//!
//! LAW-132-2 / F-091-19 / EC-3: durable persist succeeds; `try_send` soft-misses.

use std::sync::Arc;
use std::time::Duration;

use edgequake_tasks::delivery::NoopTaskNotifier;
use edgequake_tasks::memory::MemoryTaskStorage;
use edgequake_tasks::queue::{ChannelTaskQueue, SharedTaskQueue};
use edgequake_tasks::storage::SharedTaskStorage;
use edgequake_tasks::{enqueue_with_delivery, Task, TaskDeliveryMode, TaskType};
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec132_admit_wake_non_block() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let queue: SharedTaskQueue = Arc::new(ChannelTaskQueue::new(1));
    let notifier = NoopTaskNotifier;
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();

    let filler = Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        serde_json::json!({"role": "filler"}),
    );
    enqueue_with_delivery(&storage, &queue, &notifier, TaskDeliveryMode::Local, filler)
        .await
        .expect("first enqueue");

    let blocked = Task::new(
        tenant,
        workspace,
        TaskType::PdfProcessing,
        serde_json::json!({"role": "would_block_on_send_await"}),
    );
    let track_id = blocked.track_id.clone();

    let enqueue = enqueue_with_delivery(
        &storage,
        &queue,
        &notifier,
        TaskDeliveryMode::Local,
        blocked,
    );
    tokio::time::timeout(Duration::from_millis(500), enqueue)
        .await
        .expect("SPEC-132: enqueue must not hang on full wake channel")
        .expect("SPEC-132: enqueue must return Ok after durable persist");

    let stored = storage
        .get_task(&track_id)
        .await
        .expect("storage")
        .expect("task durable");
    assert_eq!(stored.track_id, track_id);
}
