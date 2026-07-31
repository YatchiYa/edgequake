//! SPEC-091 E2E + chaos suites — queue/admission edge cases (EC-17..EC-24).
//!
//! Hermetic (in-memory) verification of the "Code is Law" contracts:
//!
//! - **EC-17** delete-while-processing: cancel intent (the deletion cascade's
//!   task half) aborts in-flight work; no re-claim, no orphan rows.
//! - **EC-18** cancel from every non-terminal state: queued / mid-extraction /
//!   retry-backoff — never resurrected.
//! - **EC-19** duplicate-while-processing: single-flight detection via the
//!   active-task projection (queued, processing, and after-failed cases).
//! - **EC-20** provider budget never exceeded: N tasks × M tenants through a
//!   fair-share pool; `total_active ≤ B` at all times; nobody starves.
//! - **EC-22** worker crash: lease expiry reclaims the task for another worker.
//! - **EC-23** shutdown drain: graceful pool stop loses nothing — queued rows
//!   stay claimable for the next boot, workers stop inside the drain budget.
//! - **EC-24** provider stall: processing timeout fires (permanent failure),
//!   and the pool recovers to process the next task.
//!
//! Postgres ledger equivalents live in `contract_spec091_provider_budget.rs`;
//! explicit queued/ETA in `e2e_spec091_queue_admission.rs` (EC-21).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use edgequake_tasks::memory::MemoryTaskStorage;
use edgequake_tasks::queue::{ChannelTaskQueue, TaskQueue};
use edgequake_tasks::tenant_limiter::TenantConcurrencyLimiter;
use edgequake_tasks::worker::{WorkerPool, WorkerPoolConfig};
use edgequake_tasks::{
    SharedTaskProcessor, Task, TaskError, TaskProcessor, TaskResult, TaskStatus, TaskStorage,
    TaskType,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tenant() -> Uuid {
    Uuid::from_u128(0xA11CE)
}

fn workspace() -> Uuid {
    Uuid::from_u128(0xB0B)
}

fn insert_task(ws: Uuid, i: usize) -> Task {
    Task::new(tenant(), ws, TaskType::Insert, serde_json::json!({"i": i}))
}

/// Pool config for chaos tests: small timeouts, optional fair-share budget.
fn chaos_config(workers: usize, provider_budget: usize) -> WorkerPoolConfig {
    WorkerPoolConfig {
        num_workers: workers,
        auto_retry: false,
        initial_retry_delay_ms: 50,
        max_retry_delay_ms: 200,
        backoff_multiplier: 2.0,
        max_tasks_per_tenant: 0,
        max_lifecycle_tasks_per_tenant: 0,
        processing_timeout_secs: 60,
        provider_budget,
        tenant_lane_weight: 1,
    }
}

async fn wait_for(what: &str, deadline: Duration, cond: impl Fn() -> bool) {
    let start = Instant::now();
    while !cond() {
        assert!(
            start.elapsed() < deadline,
            "timed out waiting for condition: {what}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn stored_status(storage: &MemoryTaskStorage, track_id: &str) -> TaskStatus {
    storage
        .get_task(track_id)
        .await
        .unwrap()
        .expect("task row exists")
        .status
}

async fn wait_for_status(storage: &MemoryTaskStorage, track_id: &str, want: TaskStatus) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = stored_status(storage, track_id).await;
        if status == want {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "task {track_id} never reached {want:?} (stuck at {status:?})"
        );
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
}

// ---------------------------------------------------------------------------
// EC-17: delete-while-processing — cancel intent aborts in-flight work
// ---------------------------------------------------------------------------

/// Processor that blocks until its cancellation token fires (models an
/// in-flight LLM call aborted by the deletion cascade's cancel intent).
struct TokenHonoringProcessor {
    started: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TaskProcessor for TokenHonoringProcessor {
    async fn process(
        &self,
        _task: &mut Task,
        cancel_token: CancellationToken,
    ) -> TaskResult<serde_json::Value> {
        self.started.fetch_add(1, Ordering::SeqCst);
        cancel_token.cancelled().await;
        Err(TaskError::Cancelled("aborted at stage boundary".into()))
    }
}

#[tokio::test]
async fn e2e_spec091_queue_delete_while_processing() {
    let queue = Arc::new(ChannelTaskQueue::new(10));
    let storage = Arc::new(MemoryTaskStorage::new());
    let started = Arc::new(AtomicUsize::new(0));
    let processor: SharedTaskProcessor = Arc::new(TokenHonoringProcessor {
        started: Arc::clone(&started),
    });

    let mut pool = WorkerPool::new(
        chaos_config(1, 0),
        queue.clone(),
        storage.clone(),
        processor,
    );
    let registry = pool.cancellation_registry();
    pool.start();

    let task = insert_task(workspace(), 1);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();
    queue.send(task).await.unwrap();

    // Wait for processing to start, then fire the deletion cascade's task
    // half: durable cancel intent → token aborts the in-flight stage.
    let s = started.clone();
    wait_for("processing started", Duration::from_secs(5), move || {
        s.load(Ordering::SeqCst) >= 1
    })
    .await;
    registry.cancel(&track_id).await;

    // The worker persists Cancelled via TaskError::Cancelled (no retry).
    wait_for_status(&storage, &track_id, TaskStatus::Cancelled).await;

    // EC-17 invariant: nothing re-claims a Cancelled row — no orphan work.
    assert!(storage
        .claim_next("worker-x", Duration::from_secs(60))
        .await
        .unwrap()
        .is_none());

    pool.shutdown().await;
}

// ---------------------------------------------------------------------------
// EC-18: cancel from queued / mid-extraction / retry-backoff
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_spec091_queue_cancel_states() {
    // (a) Queued: cancel intent before any claim → never processed.
    let queue = Arc::new(ChannelTaskQueue::new(10));
    let storage = Arc::new(MemoryTaskStorage::new());
    let started = Arc::new(AtomicUsize::new(0));
    let processor: SharedTaskProcessor = Arc::new(TokenHonoringProcessor {
        started: Arc::clone(&started),
    });
    let mut pool = WorkerPool::new(
        chaos_config(1, 0),
        queue.clone(),
        storage.clone(),
        processor,
    );
    let registry = pool.cancellation_registry();

    let queued = insert_task(workspace(), 1);
    let queued_id = queued.track_id.clone();
    storage.create_task(&queued).await.unwrap();
    // Cancel intent BEFORE the pool starts: durable, pre-dates any token.
    registry.cancel(&queued_id).await;
    let mut row = storage.get_task(&queued_id).await.unwrap().unwrap();
    row.mark_cancelled();
    storage.update_task(&row).await.unwrap();

    pool.start();
    queue.send(queued).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        started.load(Ordering::SeqCst),
        0,
        "queued cancel never runs"
    );
    assert_eq!(
        stored_status(&storage, &queued_id).await,
        TaskStatus::Cancelled
    );
    pool.shutdown().await;

    // (b) Mid-extraction is covered by EC-17 above (token abort path).
    //
    // (c) Retry-backoff: a retryable failure requeues through the state
    // machine; a cancel intent during the backoff window must prevent the
    // retry from ever running again.
    let queue = Arc::new(ChannelTaskQueue::new(10));
    let storage = Arc::new(MemoryTaskStorage::new());

    struct AlwaysTransient {
        attempts: Arc<AtomicUsize>,
    }
    #[async_trait::async_trait]
    impl TaskProcessor for AlwaysTransient {
        async fn process(
            &self,
            _task: &mut Task,
            _cancel_token: CancellationToken,
        ) -> TaskResult<serde_json::Value> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            Err(TaskError::Processing("transient blip".into()))
        }
    }

    let attempts = Arc::new(AtomicUsize::new(0));
    let processor: SharedTaskProcessor = Arc::new(AlwaysTransient {
        attempts: Arc::clone(&attempts),
    });
    let mut config = chaos_config(1, 0);
    config.auto_retry = true;
    config.initial_retry_delay_ms = 400; // wide backoff window for the cancel
    let mut pool = WorkerPool::new(config, queue.clone(), storage.clone(), processor);
    let registry = pool.cancellation_registry();
    pool.start();

    let task = insert_task(workspace(), 2);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();
    queue.send(task).await.unwrap();

    // First attempt fails retryable → requeue_for_retry persists Pending.
    let a = attempts.clone();
    wait_for("first failure", Duration::from_secs(5), move || {
        a.load(Ordering::SeqCst) >= 1
    })
    .await;
    wait_for_status(&storage, &track_id, TaskStatus::Pending).await;

    // Cancel intent lands inside the backoff window (row now Pending).
    registry.cancel(&track_id).await;
    let mut row = storage.get_task(&track_id).await.unwrap().unwrap();
    row.mark_cancelled();
    storage.update_task(&row).await.unwrap();

    // Wait well past the backoff: the retry spawn's skip-guard must drop it.
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "retry-backoff cancel must not resurrect the task"
    );
    assert_eq!(
        stored_status(&storage, &track_id).await,
        TaskStatus::Cancelled
    );
    pool.shutdown().await;
}

// ---------------------------------------------------------------------------
// EC-19: duplicate upload while queued / processing / after-failed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn e2e_spec091_queue_duplicate_while_processing() {
    let storage = MemoryTaskStorage::new();
    let ws = workspace();
    let pdf_id = Uuid::new_v4();

    let mk_pdf = || {
        Task::new(
            tenant(),
            ws,
            TaskType::PdfProcessing,
            serde_json::json!({"pdf_id": pdf_id.to_string()}),
        )
    };

    // While QUEUED: the active-task projection finds the single-flight row.
    let first = mk_pdf();
    storage.create_task(&first).await.unwrap();
    let dup = storage
        .find_active_pdf_processing_task(pdf_id, ws)
        .await
        .unwrap();
    assert_eq!(
        dup.map(|t| t.track_id),
        Some(first.track_id.clone()),
        "duplicate while queued joins the in-flight task"
    );

    // While PROCESSING: still the same in-flight row (no second execution).
    let claimed = storage
        .claim_next("worker-a", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("claimable");
    assert_eq!(claimed.status, TaskStatus::Processing);
    let dup = storage
        .find_active_pdf_processing_task(pdf_id, ws)
        .await
        .unwrap();
    assert!(
        dup.is_some(),
        "duplicate while processing still single-flighted"
    );

    // After FAILED (terminal): the projection no longer blocks reprocess —
    // a fresh upload is admitted as a NEW single-flight row.
    let mut failed = claimed;
    failed.mark_failed("permanent: corrupt pdf".into());
    storage.update_task(&failed).await.unwrap();
    let dup = storage
        .find_active_pdf_processing_task(pdf_id, ws)
        .await
        .unwrap();
    assert!(
        dup.is_none(),
        "after terminal failure a fresh upload may be admitted"
    );

    let second = mk_pdf();
    storage.create_task(&second).await.unwrap();
    assert_ne!(second.track_id, first.track_id);
    assert!(storage
        .find_active_pdf_processing_task(pdf_id, ws)
        .await
        .unwrap()
        .is_some());
}

// ---------------------------------------------------------------------------
// EC-20: provider budget never exceeded (N tasks × M tenants, fair-share)
// ---------------------------------------------------------------------------

/// Samples the fair-share lane's total in-flight at processing boundaries
/// and records the maximum (the LAW-Q3/Q5 invariant probe).
struct SamplingProcessor {
    limiter_slot: Arc<Mutex<Option<TenantConcurrencyLimiter>>>,
    max_seen: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TaskProcessor for SamplingProcessor {
    async fn process(
        &self,
        _task: &mut Task,
        _cancel_token: CancellationToken,
    ) -> TaskResult<serde_json::Value> {
        let limiter = self
            .limiter_slot
            .lock()
            .expect("probe slot")
            .clone()
            .expect("limiter installed before pool start");
        self.max_seen
            .fetch_max(limiter.total_active().await, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(30)).await;
        self.max_seen
            .fetch_max(limiter.total_active().await, Ordering::SeqCst);
        self.completed.fetch_add(1, Ordering::SeqCst);
        Ok(serde_json::json!({"ok": true}))
    }
}

#[tokio::test]
async fn e2e_spec091_queue_provider_budget_never_exceeded() {
    const BUDGET: usize = 2;
    const TENANTS: usize = 3;
    const TASKS_PER_TENANT: usize = 4;

    let queue = Arc::new(ChannelTaskQueue::new(64));
    let storage = Arc::new(MemoryTaskStorage::new());
    let max_seen = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));
    let limiter_slot = Arc::new(Mutex::new(None));

    let processor: SharedTaskProcessor = Arc::new(SamplingProcessor {
        limiter_slot: Arc::clone(&limiter_slot),
        max_seen: Arc::clone(&max_seen),
        completed: Arc::clone(&completed),
    });
    let mut pool = WorkerPool::new(
        chaos_config(4, BUDGET),
        queue.clone(),
        storage.clone(),
        processor,
    );
    *limiter_slot.lock().expect("probe slot") = pool.tenant_limiter();
    pool.start();

    let mut ids = Vec::new();
    for t in 0..TENANTS {
        for i in 0..TASKS_PER_TENANT {
            let task = Task::new(
                Uuid::from_u128(0x1000 + t as u128),
                // Distinct workspaces per tenant-pair so the nested workspace
                // lane (cap 1) does not mask the provider-budget invariant.
                Uuid::from_u128(0x2000 + (t * TASKS_PER_TENANT + i) as u128),
                TaskType::Insert,
                serde_json::json!({"tenant": t, "i": i}),
            );
            ids.push(task.track_id.clone());
            storage.create_task(&task).await.unwrap();
            queue.send(task).await.unwrap();
        }
    }

    let done = completed.clone();
    wait_for("all tasks complete", Duration::from_secs(30), move || {
        done.load(Ordering::SeqCst) >= TENANTS * TASKS_PER_TENANT
    })
    .await;

    assert!(
        max_seen.load(Ordering::SeqCst) <= BUDGET,
        "provider budget exceeded: max in-flight {} > {}",
        max_seen.load(Ordering::SeqCst),
        BUDGET
    );
    for id in &ids {
        assert_eq!(
            stored_status(&storage, id).await,
            TaskStatus::Indexed,
            "no tenant may starve under fair-share (LAW-Q5)"
        );
    }
    pool.shutdown().await;
}

// ---------------------------------------------------------------------------
// EC-22: worker crash — lease expiry reclaims the task (chaos)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn chaos_spec091_queue_worker_crash_lease_reclaim() {
    let storage = MemoryTaskStorage::new();
    let task = insert_task(workspace(), 1);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.unwrap();

    // worker-a claims with a short lease, then "crashes": no heartbeat, no
    // completion, no release (kill -9 semantics).
    let claimed = storage
        .claim_next("worker-a", Duration::from_millis(120))
        .await
        .unwrap()
        .expect("claimable");
    assert_eq!(claimed.status, TaskStatus::Processing);
    let lease_token = claimed.lease_token.expect("lease token issued");

    // While the lease is valid, nobody else can claim.
    assert!(storage
        .claim_next("worker-b", Duration::from_secs(60))
        .await
        .unwrap()
        .is_none());

    // After expiry, worker-b reclaims the abandoned task (EC-22).
    tokio::time::sleep(Duration::from_millis(180)).await;
    let reclaimed = storage
        .claim_next("worker-b", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("expired lease must be reclaimable");
    assert_eq!(reclaimed.track_id, track_id);
    assert_eq!(reclaimed.lease_owner.as_deref(), Some("worker-b"));

    // The crashed owner's heartbeat is rejected (fencing): refresh with the
    // stale token fails, proving the split-brain guard.
    let refreshed = storage
        .refresh_lease(&track_id, "worker-a", lease_token, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(!refreshed, "stale lease owner must lose the CAS");

    // worker-b completes through legal transitions.
    let mut done = reclaimed;
    done.mark_success(serde_json::json!({"ok": true}));
    storage.update_task(&done).await.unwrap();
    assert_eq!(
        stored_status(&storage, &track_id).await,
        TaskStatus::Indexed
    );
}

// ---------------------------------------------------------------------------
// EC-23: graceful shutdown drain — nothing lost, workers stop (chaos)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn chaos_spec091_queue_shutdown_drain() {
    let queue = Arc::new(ChannelTaskQueue::new(10));
    let storage = Arc::new(MemoryTaskStorage::new());
    let started = Arc::new(AtomicUsize::new(0));
    let processor: SharedTaskProcessor = Arc::new(TokenHonoringProcessor {
        started: Arc::clone(&started),
    });

    let mut pool = WorkerPool::new(
        chaos_config(1, 0),
        queue.clone(),
        storage.clone(),
        processor,
    );
    pool.start();

    let inflight = insert_task(workspace(), 1);
    let inflight_id = inflight.track_id.clone();
    let queued1 = insert_task(workspace(), 2);
    let queued1_id = queued1.track_id.clone();
    let queued2 = insert_task(workspace(), 3);
    let queued2_id = queued2.track_id.clone();
    for t in [inflight, queued1, queued2] {
        storage.create_task(&t).await.unwrap();
        queue.send(t).await.unwrap();
    }

    let s = started.clone();
    wait_for("in-flight started", Duration::from_secs(5), move || {
        s.load(Ordering::SeqCst) >= 1
    })
    .await;

    // Graceful shutdown: broadcast + cancel in-flight tokens, then join.
    tokio::time::timeout(Duration::from_secs(10), pool.shutdown())
        .await
        .expect("pool must stop inside the drain budget");

    // In-flight work was cooperatively cancelled and persisted terminal.
    let s = stored_status(&storage, &inflight_id).await;
    assert!(
        matches!(s, TaskStatus::Cancelled | TaskStatus::Failed),
        "in-flight task must be terminal after drain, got {s:?}"
    );

    // EC-23 invariant: queued rows were NOT lost or cancelled — a fresh
    // "next boot" claim picks them up in FIFO order and completes them.
    let first = storage
        .claim_next("worker-next-boot", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("queued work survives drain");
    assert_eq!(first.track_id, queued1_id);
    let mut done = first;
    done.mark_success(serde_json::json!({"ok": true}));
    storage.update_task(&done).await.unwrap();

    let second = storage
        .claim_next("worker-next-boot", Duration::from_secs(60))
        .await
        .unwrap()
        .expect("second queued task survives drain");
    assert_eq!(second.track_id, queued2_id);
}

// ---------------------------------------------------------------------------
// EC-24: provider stall — timeout fires, pool recovers (chaos)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn chaos_spec091_queue_provider_stall() {
    let queue = Arc::new(ChannelTaskQueue::new(10));
    let storage = Arc::new(MemoryTaskStorage::new());

    struct StallThenOk;
    #[async_trait::async_trait]
    impl TaskProcessor for StallThenOk {
        async fn process(
            &self,
            task: &mut Task,
            _cancel_token: CancellationToken,
        ) -> TaskResult<serde_json::Value> {
            if task.task_data.get("stall").is_some() {
                // Provider hangs: never returns, ignores the token (worst
                // case — the worker-level processing timeout is the backstop).
                std::future::pending::<()>().await;
            }
            Ok(serde_json::json!({"ok": true}))
        }
    }

    let mut config = chaos_config(1, 0);
    config.processing_timeout_secs = 1; // stall backstop
    let mut pool = WorkerPool::new(
        config,
        queue.clone(),
        storage.clone(),
        Arc::new(StallThenOk),
    );
    pool.start();

    let stall = Task::new(
        tenant(),
        workspace(),
        TaskType::Insert,
        serde_json::json!({"stall": true}),
    );
    let stall_id = stall.track_id.clone();
    let next = insert_task(workspace(), 2);
    let next_id = next.track_id.clone();
    storage.create_task(&stall).await.unwrap();
    storage.create_task(&next).await.unwrap();
    queue.send(stall).await.unwrap();
    queue.send(next).await.unwrap();

    // Stall backstop: the processing timeout marks it permanently Failed.
    wait_for_status(&storage, &stall_id, TaskStatus::Failed).await;

    // The pool recovers: the queued task is processed and Indexed.
    wait_for_status(&storage, &next_id, TaskStatus::Indexed).await;
    pool.shutdown().await;
}
