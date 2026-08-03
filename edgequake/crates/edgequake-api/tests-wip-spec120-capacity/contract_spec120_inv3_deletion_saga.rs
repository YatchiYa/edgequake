//! SPEC-120 INV-3: deletion waits for live ingest lease before purge/fence.

use std::sync::Arc;
use std::time::Duration;

use edgequake_tasks::{memory::MemoryTaskStorage, SharedTaskStorage, Task, TaskStatus, TaskType};
use uuid::Uuid;

/// Synthetic drain wait used by INV-3 (mirrors document_task_cleanup logic).
async fn wait_until_drained(
    storage: &SharedTaskStorage,
    track_ids: &[String],
    timeout: Duration,
) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let now = chrono::Utc::now();
        let mut blocking = Vec::new();
        for track_id in track_ids {
            let Some(task) = storage
                .get_task(track_id)
                .await
                .map_err(|e| e.to_string())?
            else {
                continue;
            };
            let terminal = matches!(
                task.status,
                TaskStatus::Cancelled
                    | TaskStatus::Indexed
                    | TaskStatus::Failed
                    | TaskStatus::DeadLetter
            );
            if !terminal && !task.lease_is_expired(now) {
                blocking.push(track_id.clone());
            }
        }
        if blocking.is_empty() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(format!("still blocking: {}", blocking.join(",")));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn inv3_purge_blocked_while_lease_live() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let mut ingest = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "inv3-doc" }),
    );
    ingest.mark_processing();
    // Fresh lease — not expired.
    ingest.lease_owner = Some("worker-1".into());
    ingest.lease_token = Some(Uuid::new_v4());
    ingest.lease_expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(120));
    let track_id = ingest.track_id.clone();
    storage.create_task(&ingest).await.unwrap();

    let err = wait_until_drained(
        &storage,
        std::slice::from_ref(&track_id),
        Duration::from_millis(80),
    )
    .await
    .expect_err("live lease must block purge");
    assert!(err.contains(&track_id));
}

#[tokio::test]
async fn inv3_expired_lease_allows_purge() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let mut ingest = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "inv3-doc-expired" }),
    );
    ingest.mark_processing();
    ingest.lease_owner = Some("worker-1".into());
    ingest.lease_token = Some(Uuid::new_v4());
    ingest.lease_expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(5));
    let track_id = ingest.track_id.clone();
    storage.create_task(&ingest).await.unwrap();

    wait_until_drained(&storage, &[track_id], Duration::from_secs(1))
        .await
        .expect("expired lease must be treated as drained");
}

#[tokio::test]
async fn inv3_cancelled_dependent_allows_purge() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let mut ingest = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": "inv3-doc-cancelled" }),
    );
    ingest.mark_processing();
    ingest.lease_owner = Some("worker-1".into());
    ingest.lease_token = Some(Uuid::new_v4());
    ingest.lease_expires_at = Some(chrono::Utc::now() + chrono::Duration::seconds(120));
    ingest.mark_cancelled();
    let track_id = ingest.track_id.clone();
    storage.create_task(&ingest).await.unwrap();

    wait_until_drained(&storage, &[track_id], Duration::from_secs(1))
        .await
        .expect("terminal cancelled dependent must not block purge");
}
