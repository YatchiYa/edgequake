//! SPEC-091 WP1: fairness permit released before materialize (LAW-WP3).
//!
//! A processor that drops its fairness permit mid-flight must free the lane
//! so a second task for the same tenant can acquire while the first is still
//! "processing" (simulating DB materialize).
//!
//! Run: `cargo test -p edgequake-tasks --test contract_spec091_fairness_release_before_materialize`

use async_trait::async_trait;
use edgequake_tasks::{
    FairnessClass, FairnessPermit, Task, TaskProcessor, TaskProviderClass, TaskResult,
    TenantConcurrencyLimiter, TryAcquireOutcome,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct EarlyReleaseProcessor {
    /// Set when fairness was dropped before the long materialize sleep.
    released_before_materialize: AtomicBool,
    /// Signalled after fairness drop so the waiter can try_acquire.
    materialize_gate: Arc<Notify>,
    /// Held until test allows completion.
    finish: Arc<Notify>,
}

#[async_trait]
impl TaskProcessor for EarlyReleaseProcessor {
    async fn process(
        &self,
        _task: &mut Task,
        _cancel_token: CancellationToken,
    ) -> TaskResult<serde_json::Value> {
        unreachable!("use process_with_fairness")
    }

    async fn process_with_fairness(
        &self,
        _task: &mut Task,
        _cancel_token: CancellationToken,
        fairness: Option<FairnessPermit>,
    ) -> TaskResult<serde_json::Value> {
        // Simulate extract holding the permit, then materialize without it.
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(fairness);
        self.released_before_materialize
            .store(true, Ordering::SeqCst);
        self.materialize_gate.notify_one();
        self.finish.notified().await;
        Ok(serde_json::json!({ "ok": true }))
    }
}

#[tokio::test]
async fn fairness_lane_frees_when_permit_dropped_before_materialize() {
    let limiter = TenantConcurrencyLimiter::new_fair_share(1, 1, 0);
    let tenant = Uuid::new_v4();
    let ws = Uuid::new_v4();
    let local = TaskProviderClass::Local("local".to_string());

    let first = match limiter
        .try_acquire(tenant, ws, FairnessClass::Ingest, &local)
        .await
    {
        TryAcquireOutcome::Acquired(p) => p,
        other => panic!("expected Acquired, got {other:?}"),
    };

    // Lane full while first holds permit.
    assert!(matches!(
        limiter
            .try_acquire(tenant, ws, FairnessClass::Ingest, &local)
            .await,
        TryAcquireOutcome::AtCapacity
    ));

    let materialize_gate = Arc::new(Notify::new());
    let finish = Arc::new(Notify::new());
    let processor = EarlyReleaseProcessor {
        released_before_materialize: AtomicBool::new(false),
        materialize_gate: Arc::clone(&materialize_gate),
        finish: Arc::clone(&finish),
    };

    let mut task = Task::new(
        tenant,
        ws,
        edgequake_tasks::TaskType::Insert,
        serde_json::json!({}),
    );
    let cancel = CancellationToken::new();
    let proc_handle = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            processor
                .process_with_fairness(&mut task, cancel, Some(first))
                .await
        }
    });

    // Wait until processor dropped fairness (materialize phase).
    tokio::time::timeout(Duration::from_secs(2), materialize_gate.notified())
        .await
        .expect("processor should release fairness");

    // Second ingest must now acquire (lane freed mid-flight).
    let second = match limiter
        .try_acquire(tenant, ws, FairnessClass::Ingest, &local)
        .await
    {
        TryAcquireOutcome::Acquired(p) => p,
        other => panic!("expected second Acquired after early release, got {other:?}"),
    };
    drop(second);

    finish.notify_one();
    proc_handle
        .await
        .expect("join")
        .expect("process ok");
}
