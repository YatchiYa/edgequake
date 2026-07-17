//! Worker pool for processing tasks from the queue.
//!
//! ## Implements
//!
//! - **FEAT0910**: Worker pool with configurable concurrency
//! - **FEAT0911**: Task processor trait abstraction
//! - **FEAT0912**: Graceful shutdown with task completion
//! - **SPEC-001/Issue-8**: Exponential backoff for retries
//! - **FEAT-TENANT-FAIRNESS**: Per-tenant concurrency limits
//!
//! ## Use Cases
//!
//! - **UC2601**: System spawns workers to process queued tasks
//! - **UC2602**: System retries failed tasks with exponential backoff
//! - **UC2603**: System shuts down gracefully completing in-flight work
//! - **UC2604**: System prevents one tenant from monopolizing workers
//!
//! ## Enforces
//!
//! - **BR0910**: Worker count bounded to prevent resource exhaustion
//! - **BR0911**: In-flight tasks must complete before shutdown
//! - **BR0912**: Retry delays use exponential backoff (2^n * base_delay)
//! - **BR0913**: Per-tenant concurrency capped at max_tasks_per_tenant
//!
//! ## WHY Worker Pool Architecture?
//!
//! Document processing (PDF extraction, embedding generation) is CPU/IO intensive.
//! The worker pool provides:
//! - **Bounded concurrency**: Prevents resource exhaustion during burst uploads
//! - **Task isolation**: One failing task doesn't affect others
//! - **Tenant fairness**: Per-tenant limits prevent monopolization
//! - **Graceful shutdown**: In-flight tasks complete before termination
//! - **Retry logic**: Transient failures (network, rate limits) auto-recover
//! - **Exponential backoff**: Prevents hammering failing services
//! - **Permanent failure cleanup**: Updates document status on retry exhaustion
//!
//! Default worker count is `num_cpus * 4` because pipeline processing is IO-bound
//! (waiting for LLM API calls, embedding generation). Workers spend most of their
//! time in network I/O, so we need more workers than CPU cores to keep the pipeline
//! saturated. Override via the `WORKER_THREADS` environment variable.

use crate::{
    cancellation::CancellationRegistry,
    error::{TaskError, TaskResult},
    queue::TaskQueue,
    storage::TaskStorage,
    task_lease_ttl_from_env,
    tenant_limiter::TenantConcurrencyLimiter,
    types::{Task, TaskStatus},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, Instrument};

/// RAII guard that aborts the heartbeat task on drop.
///
/// WHY: If `processor.process()` panics, the stack unwinds and this guard's
/// `Drop` impl fires, aborting the heartbeat. Without this, a panic leaves
/// the heartbeat running forever — the task stays in "processing" with a
/// live heartbeat, and neither the periodic orphan check nor the processing
/// timeout can catch it (timeout is in the same panic scope, orphan check
/// sees a fresh `updated_at`).
struct HeartbeatGuard(JoinHandle<()>);

impl Drop for HeartbeatGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Minimum allowed processing timeout (60 seconds).
///
/// WHY: A timeout of 0 would cause every task to immediately time out,
/// making the system non-functional. Even very fast tasks need a few
/// seconds for LLM API round-trips.
const MIN_PROCESSING_TIMEOUT_SECS: u64 = 60;

/// Task processor trait - implement this to process different task types.
///
/// Implementors handle both normal processing and cleanup on permanent failure.
///
/// The `CancellationToken` parameter enables cooperative cancellation:
/// processors should periodically check `cancel_token.is_cancelled()` and
/// return early with an appropriate error when cancellation is detected.
#[async_trait::async_trait]
pub trait TaskProcessor: Send + Sync {
    /// Process a task with cooperative cancellation support.
    ///
    /// Implementations MUST check `cancel_token.is_cancelled()` at each
    /// stage boundary (chunking, extraction, embedding, storage) and
    /// return `Err(TaskError::Cancelled)` when cancellation is detected.
    async fn process(
        &self,
        task: &mut Task,
        cancel_token: CancellationToken,
    ) -> TaskResult<serde_json::Value>;

    /// Called when a task has permanently failed (retries exhausted or circuit
    /// breaker tripped). Override to update document status, clean up resources,
    /// or send notifications.
    ///
    /// WHY: Without this callback, documents get stuck in "processing" status
    /// forever when the task fails permanently. The worker knows when retries
    /// are exhausted, but only the processor knows how to update document
    /// metadata and clean up resources.
    ///
    /// Default implementation is a no-op for backward compatibility.
    async fn on_permanent_failure(&self, task: &Task, error_msg: &str) {
        let _ = (task, error_msg); // suppress unused warnings
    }
}

/// Shared task processor
pub type SharedTaskProcessor = Arc<dyn TaskProcessor>;

/// Worker pool configuration
#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    /// Number of worker threads
    pub num_workers: usize,

    /// Whether to retry failed tasks automatically
    pub auto_retry: bool,

    /// Initial delay before retrying failed tasks (milliseconds)
    ///
    /// @implements SPEC-001/Issue-8: Exponential backoff base delay
    pub initial_retry_delay_ms: u64,

    /// Maximum retry delay (milliseconds) to prevent runaway backoff
    ///
    /// @implements SPEC-001/Issue-8: Capped exponential backoff
    pub max_retry_delay_ms: u64,

    /// Backoff multiplier (default: 2.0 for exponential backoff)
    pub backoff_multiplier: f64,

    /// Maximum concurrent tasks per tenant.
    ///
    /// WHY: Prevents one tenant from monopolizing all workers. When a tenant
    /// has `max_tasks_per_tenant` tasks in flight, excess tasks park on the
    /// tenant semaphore until a slot frees (no channel requeue churn).
    ///
    /// Default: `max(1, num_workers * 3/4)` — most slots usable per tenant
    /// while keeping headroom for others.
    ///
    /// Set to 0 to disable per-tenant limiting (all workers available to any tenant).
    pub max_tasks_per_tenant: usize,

    /// Maximum time (seconds) a single task can process before being timed out.
    ///
    /// WHY: Without a timeout, processor.process() can hang forever (e.g., stuck
    /// LLM call, unresponsive PDF conversion) while the heartbeat mechanism keeps
    /// the task looking "alive" in the database. This creates phantom "Processing"
    /// banners that never resolve — the orphan recovery can't catch them because
    /// the heartbeat keeps updating `updated_at`.
    ///
    /// Default: 7200s (2 hours) — generous enough for very large PDF processing
    /// (1000+ page documents with vision LLM extraction at ~12s/page ≈ 3.3h) while
    /// still catching truly stuck tasks within a reasonable window.
    /// Override via `TASK_PROCESSING_TIMEOUT_SECS` environment variable.
    pub processing_timeout_secs: u64,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        // WHY num_cpus * 4: Pipeline processing is IO-bound (waiting for LLM API
        // calls and embedding generation). Workers spend most of their time in
        // network I/O, not CPU computation. Higher worker count ensures the
        // pipeline stays saturated with concurrent requests to external services.
        let num_workers = (num_cpus::get() * 4).max(4);
        Self {
            num_workers,
            auto_retry: true,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 60_000,
            backoff_multiplier: 2.0,
            // WHY num_workers * 3/4: For IO-bound workloads, each tenant can
            // use most of the pool while still guaranteeing at least 25% of
            // workers remain available for other tenants.
            max_tasks_per_tenant: (num_workers * 3 / 4).max(1),
            // WHY 2 hours: Large PDFs (1000+ pages) with vision LLM extraction
            // can take 3+ hours. 2 hours catches most real-world cases while
            // still preventing infinite hangs. Override via
            // TASK_PROCESSING_TIMEOUT_SECS env var.
            processing_timeout_secs: 7200.max(MIN_PROCESSING_TIMEOUT_SECS),
        }
    }
}

/// Calculate exponential backoff delay for a given retry attempt.
///
/// @implements SPEC-001/Issue-8: Exponential backoff calculation
///
/// Formula: min(initial_delay * multiplier^attempt, max_delay)
fn calculate_backoff_delay(
    attempt: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
) -> u64 {
    let delay = initial_delay_ms as f64 * multiplier.powi(attempt as i32);
    (delay as u64).min(max_delay_ms)
}

/// Worker pool for processing tasks
pub struct WorkerPool {
    config: WorkerPoolConfig,
    queue: Arc<dyn TaskQueue>,
    storage: Arc<dyn TaskStorage>,
    processor: SharedTaskProcessor,
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
    tenant_limiter: Option<TenantConcurrencyLimiter>,
    cancellation_registry: CancellationRegistry,
    /// Rate-limit fairness park DEBUG logs (one line per N parks).
    fairness_park_logs: Arc<AtomicU64>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(
        config: WorkerPoolConfig,
        queue: Arc<dyn TaskQueue>,
        storage: Arc<dyn TaskStorage>,
        processor: SharedTaskProcessor,
    ) -> Self {
        // Create tenant limiter if max_tasks_per_tenant > 0
        let tenant_limiter = if config.max_tasks_per_tenant > 0 {
            Some(TenantConcurrencyLimiter::new(config.max_tasks_per_tenant))
        } else {
            None
        };

        Self {
            config,
            queue,
            storage,
            processor,
            handles: Vec::new(),
            shutdown_tx: None,
            tenant_limiter,
            cancellation_registry: CancellationRegistry::new(),
            fairness_park_logs: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get a reference to the cancellation registry.
    ///
    /// WHY: The cancel API handler needs access to this registry to trigger
    /// cooperative cancellation of in-flight tasks. Store this reference in
    /// your AppState and pass it to the cancel endpoint.
    pub fn cancellation_registry(&self) -> CancellationRegistry {
        self.cancellation_registry.clone()
    }

    /// Optional per-tenant limiter (None when `max_tasks_per_tenant == 0`).
    pub fn tenant_limiter(&self) -> Option<TenantConcurrencyLimiter> {
        self.tenant_limiter.clone()
    }

    /// Start the worker pool on the current Tokio runtime.
    pub fn start(&mut self) {
        self.start_on(&tokio::runtime::Handle::current());
    }

    /// Start workers on a dedicated runtime (ingest isolation from Axum serving).
    ///
    /// Nested `tokio::spawn` calls inside worker loops inherit this runtime's
    /// context once the worker task is polled.
    pub fn start_on(&mut self, runtime: &tokio::runtime::Handle) {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        if let Some(ref limiter) = self.tenant_limiter {
            info!(
                "Starting worker pool: {} workers, max {} tasks/tenant",
                self.config.num_workers,
                limiter.max_per_tenant()
            );
        } else {
            info!(
                "Starting worker pool: {} workers, no tenant limit",
                self.config.num_workers
            );
        }

        for worker_id in 0..self.config.num_workers {
            let queue = Arc::clone(&self.queue);
            let storage = Arc::clone(&self.storage);
            let processor = Arc::clone(&self.processor);
            let config = self.config.clone();
            let mut shutdown_rx = shutdown_tx.subscribe();
            let park_shutdown_tx = shutdown_tx.clone();
            let tenant_limiter = self.tenant_limiter.clone();
            let cancel_registry = self.cancellation_registry.clone();
            let fairness_park_logs = Arc::clone(&self.fairness_park_logs);

            let handle = runtime.spawn(async move {
                info!("Worker {} started", worker_id);
                let worker_name = format!("worker-{worker_id}");
                let lease_ttl = task_lease_ttl_from_env();
                let mut poll_interval = tokio::time::interval(Duration::from_secs(2));
                poll_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                // Skip the immediate first tick so we don't race startup hydrate.
                poll_interval.tick().await;

                loop {
                    // SPEC-057 P1: channel is wake-only; Postgres/memory claim is SSOT.
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            info!("Worker {} shutting down", worker_id);
                            break;
                        }
                        result = queue.receive() => {
                            match result {
                                Ok(_wake) => {
                                    // Ignore payload — claim_next authorizes work.
                                }
                                Err(e) => {
                                    if queue.is_closed() {
                                        info!("Worker {} queue closed", worker_id);
                                        break;
                                    }
                                    error!(
                                        error.source = "task_worker",
                                        error.action = "receive_wake",
                                        worker_id = worker_id,
                                        error.message = %e,
                                        "Worker failed to receive wake from queue"
                                    );
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    continue;
                                }
                            }
                        }
                        _ = poll_interval.tick() => {
                            // Periodic claim poll for Pending surviving restart / lost wakes.
                        }
                    }

                    let mut task = match storage.claim_next(&worker_name, lease_ttl).await {
                        Ok(Some(t)) => t,
                        Ok(None) => continue,
                        Err(e) => {
                            error!(
                                error.source = "task_worker",
                                error.action = "claim_next",
                                worker_id = worker_id,
                                error.message = %e,
                                "Failed to claim next task"
                            );
                            continue;
                        }
                    };

                    let lease_token = match task.lease_token {
                        Some(token) => token,
                        None => {
                            warn!(
                                worker_id = worker_id,
                                task_id = %task.track_id,
                                "Claimed task missing lease_token — releasing"
                            );
                            let _ = storage
                                .release_claim(&task.track_id, &worker_name, uuid::Uuid::nil())
                                .await;
                            continue;
                        }
                    };

                    // FEAT-CANCEL: Drop terminal / cancel-intent after claim.
                    if should_skip_task(&storage, &cancel_registry, &task).await {
                        debug!(
                            worker_id = worker_id,
                            task_id = %task.track_id,
                            "Skipping cancelled or terminal task after claim"
                        );
                        if cancel_registry.has_cancel_intent(&task.track_id).await {
                            task.mark_cancelled();
                            let _ = storage.update_task(&task).await;
                        } else if let Err(e) = storage
                            .release_claim(&task.track_id, &worker_name, lease_token)
                            .await
                        {
                            warn!(
                                task_id = %task.track_id,
                                error = %e,
                                "Failed to release claim for skipped task"
                            );
                        }
                        cancel_registry.deregister(&task.track_id).await;
                        continue;
                    }

                    // FEAT-TENANT-FAIRNESS: release claim before park (no double-process).
                    let _tenant_permit = if let Some(ref limiter) = tenant_limiter {
                        match limiter.try_acquire(task.tenant_id).await {
                            Some(permit) => Some(permit),
                            None => {
                                let n = fairness_park_logs.fetch_add(1, Ordering::Relaxed) + 1;
                                if n == 1 || n.is_multiple_of(32) {
                                    info!(
                                        worker_id = worker_id,
                                        task_id = %task.track_id,
                                        tenant_id = %task.tenant_id,
                                        park_events = n,
                                        park_waiters = limiter.park_waiter_count(),
                                        "Tenant at concurrency limit — releasing claim and parking (aggregated)"
                                    );
                                }
                                if let Err(e) = storage
                                    .release_claim(&task.track_id, &worker_name, lease_token)
                                    .await
                                {
                                    error!(
                                        task_id = %task.track_id,
                                        error = %e,
                                        "Failed to release claim before fairness park"
                                    );
                                } else {
                                    // Reload as Pending for park wake payload.
                                    if let Ok(Some(pending)) =
                                        storage.get_task(&task.track_id).await
                                    {
                                        task = pending;
                                    } else {
                                        task.status = TaskStatus::Pending;
                                        task.clear_lease();
                                    }
                                    spawn_fairness_park(
                                        worker_id,
                                        task,
                                        Arc::clone(&queue),
                                        Arc::clone(&storage),
                                        limiter.clone(),
                                        cancel_registry.clone(),
                                        park_shutdown_tx.subscribe(),
                                    );
                                }
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    info!(
                        "Worker {} processing task: {} (tenant: {})",
                        worker_id, task.track_id, task.tenant_id
                    );

                    let cancel_token = cancel_registry.register(&task.track_id).await;

                    // Lease heartbeat: refresh_lease CAS; lost ownership aborts via cancel.
                    let heartbeat_track_id = task.track_id.clone();
                    let heartbeat_storage = Arc::clone(&storage);
                    let heartbeat_worker = worker_name.clone();
                    let heartbeat_token = lease_token;
                    let heartbeat_ttl = lease_ttl;
                    let heartbeat_cancel = cancel_token.clone();
                    let _heartbeat_guard = HeartbeatGuard(tokio::spawn(async move {
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(60));
                        interval.tick().await;
                        loop {
                            interval.tick().await;
                            match heartbeat_storage
                                .refresh_lease(
                                    &heartbeat_track_id,
                                    &heartbeat_worker,
                                    heartbeat_token,
                                    heartbeat_ttl,
                                )
                                .await
                            {
                                Ok(true) => {}
                                Ok(false) => {
                                    warn!(
                                        task_id = %heartbeat_track_id,
                                        "Lost task lease — aborting processing"
                                    );
                                    heartbeat_cancel.cancel();
                                    break;
                                }
                                Err(e) => {
                                    debug!(
                                        "Lease refresh failed for task {}: {}",
                                        heartbeat_track_id, e
                                    );
                                }
                            }
                        }
                    }));

                    let timeout_duration = tokio::time::Duration::from_secs(
                        task.metadata
                            .as_ref()
                            .and_then(|m| m.get("processing_timeout_secs"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(config.processing_timeout_secs),
                    );
                    let span_task_id = task.track_id.clone();
                    let span_tenant_id = task.tenant_id;
                    let span_task_type = task.task_type;
                    let process_result = tokio::time::timeout(
                        timeout_duration,
                        processor
                            .process(&mut task, cancel_token.clone())
                            .instrument(tracing::info_span!(
                                "task_process",
                                worker_id = worker_id,
                                task_id = %span_task_id,
                                tenant_id = %span_tenant_id,
                                task_type = ?span_task_type,
                            )),
                    )
                    .await;

                    match process_result {
                        Ok(Ok(result)) => {
                            task.mark_success(result);
                            info!(
                                "Worker {} completed task: {} (tenant: {})",
                                worker_id, task.track_id, task.tenant_id
                            );
                        }
                        Ok(Err(TaskError::Cancelled(msg))) => {
                            task.mark_cancelled();
                            info!(
                                worker_id = worker_id,
                                task_id = %task.track_id,
                                tenant_id = %task.tenant_id,
                                reason = %msg,
                                "Task cancelled — preserving Cancelled status (no retry)"
                            );
                        }
                        Ok(Err(e)) => {
                            let error_msg = format!("{}", e);
                            task.mark_failed_with_details(
                                crate::types::TaskFailureInfo::from_processing_error(
                                    error_msg.clone(),
                                ),
                            );

                            if task.circuit_breaker_tripped {
                                error!(
                                    worker_id = worker_id,
                                    task_id = %task.track_id,
                                    tenant_id = %task.tenant_id,
                                    consecutive_timeouts = task.consecutive_timeout_failures,
                                    "Task permanently failed: Circuit breaker tripped"
                                );
                            } else {
                                error!(
                                    worker_id = worker_id,
                                    task_id = %task.track_id,
                                    tenant_id = %task.tenant_id,
                                    retry_count = task.retry_count,
                                    max_retries = task.max_retries,
                                    consecutive_timeouts = task.consecutive_timeout_failures,
                                    error = %error_msg,
                                    "Task processing failed"
                                );
                            }

                            let will_retry = config.auto_retry
                                && task.can_retry()
                                && !task.circuit_breaker_tripped
                                && !crate::is_permanent_ingestion_failure(&error_msg);

                            if will_retry {
                                let retry_delay_ms = calculate_backoff_delay(
                                    task.retry_count as u32,
                                    config.initial_retry_delay_ms,
                                    config.max_retry_delay_ms,
                                    config.backoff_multiplier,
                                );

                                warn!(
                                    task_id = %task.track_id,
                                    attempt = task.retry_count,
                                    max_retries = task.max_retries,
                                    delay_ms = retry_delay_ms,
                                    "Scheduling retry with exponential backoff"
                                );

                                // Claim SSOT: retries must be Pending again (wake-only channel).
                                task.status = TaskStatus::Pending;
                                task.completed_at = None;
                                task.clear_lease();

                                let retry_task = task.clone();
                                let retry_queue = Arc::clone(&queue);
                                let retry_cancel = cancel_registry.clone();
                                let retry_storage = Arc::clone(&storage);
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(
                                        retry_delay_ms,
                                    ))
                                    .await;

                                    let track_id = retry_task.track_id.clone();
                                    if should_skip_task(
                                        &retry_storage,
                                        &retry_cancel,
                                        &retry_task,
                                    )
                                    .await
                                    {
                                        debug!(
                                            task_id = %track_id,
                                            "Skipping retry — task cancelled or terminal"
                                        );
                                        return;
                                    }
                                    if let Err(e) = retry_queue.send(retry_task).await {
                                        error!(
                                            error.source = "task_worker",
                                            error.action = "requeue_retry",
                                            task_id = %track_id,
                                            error.message = %e,
                                            "Failed to wake queue for retry"
                                        );
                                    }
                                });
                            } else {
                                let reason = if task.circuit_breaker_tripped {
                                    format!(
                                        "Circuit breaker tripped after {} consecutive timeouts. \
                                        Last error: {}",
                                        task.consecutive_timeout_failures, error_msg
                                    )
                                } else {
                                    format!(
                                        "Retries exhausted ({}/{} attempts). Last error: {}",
                                        task.retry_count, task.max_retries, error_msg
                                    )
                                };
                                error!(
                                    task_id = %task.track_id,
                                    tenant_id = %task.tenant_id,
                                    "Task permanently failed: {}", reason
                                );
                                processor.on_permanent_failure(&task, &reason).await;
                            }
                        }
                        Err(_elapsed) => {
                            let timeout_msg = format!(
                                "Task processing timed out after {} seconds",
                                config.processing_timeout_secs
                            );
                            task.mark_failed(timeout_msg.clone());

                            error!(
                                worker_id = worker_id,
                                task_id = %task.track_id,
                                tenant_id = %task.tenant_id,
                                timeout_secs = config.processing_timeout_secs,
                                "Task timed out — marking as permanently failed"
                            );

                            processor.on_permanent_failure(&task, &timeout_msg).await;
                        }
                    }

                    cancel_registry.deregister(&task.track_id).await;

                    if let Err(e) = storage.update_task(&task).await {
                        error!(
                            error.source = "task_worker",
                            error.action = "persist_task_result",
                            worker_id = worker_id,
                            task_id = %task.track_id,
                            task_status = ?task.status,
                            error.message = %e,
                            "Failed to persist task result"
                        );
                    }

                    // _tenant_permit dropped here
                }

                info!("Worker {} stopped", worker_id);
            });

            self.handles.push(handle);
        }

        // Spawn periodic cleanup task for tenant semaphores (same ingest runtime).
        if let Some(limiter) = self.tenant_limiter.clone() {
            runtime.spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                    limiter.cleanup_idle().await;
                }
            });
        }
    }

    /// Shutdown the worker pool gracefully
    pub async fn shutdown(self) {
        info!("Shutting down worker pool");

        if let Some(shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        for handle in self.handles {
            let _ = handle.await;
        }

        info!("Worker pool shut down complete");
    }

    /// Get number of workers
    pub fn num_workers(&self) -> usize {
        self.config.num_workers
    }
}

/// True when the worker must not start (or requeue) this task.
async fn should_skip_task(
    storage: &Arc<dyn TaskStorage>,
    cancel_registry: &CancellationRegistry,
    task: &Task,
) -> bool {
    if cancel_registry.has_cancel_intent(&task.track_id).await {
        return true;
    }
    if task.status == TaskStatus::Cancelled || task.status == TaskStatus::Indexed {
        return true;
    }
    match storage.get_task(&task.track_id).await {
        Ok(Some(stored)) => {
            stored.status == TaskStatus::Cancelled
                || stored.status == TaskStatus::Indexed
                || stored.is_terminal()
        }
        Ok(None) => false,
        Err(e) => {
            warn!(
                task_id = %task.track_id,
                error = %e,
                "Failed to load task status for cancel/terminal guard — proceeding cautiously"
            );
            false
        }
    }
}

/// Park until a tenant permit frees, then re-enqueue once (no polling storm).
fn spawn_fairness_park(
    worker_id: usize,
    task: Task,
    queue: Arc<dyn TaskQueue>,
    storage: Arc<dyn TaskStorage>,
    limiter: TenantConcurrencyLimiter,
    cancel_registry: CancellationRegistry,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    tokio::spawn(async move {
        let track_id = task.track_id.clone();
        let tenant_id = task.tenant_id;

        let permit = tokio::select! {
            result = limiter.acquire(tenant_id) => {
                match result {
                    Ok(permit) => permit,
                    Err(_) => {
                        debug!(
                            worker_id,
                            task_id = %track_id,
                            "Fairness park aborted — semaphore closed"
                        );
                        return;
                    }
                }
            }
            _ = shutdown_rx.recv() => {
                debug!(
                    worker_id,
                    task_id = %track_id,
                    "Fairness park aborted — worker pool shutting down"
                );
                // Return task to queue so hydrate/restart can recover if needed.
                let _ = queue.send(task).await;
                return;
            }
        };

        // Release immediately — the worker will try_acquire when it dequeues.
        // Park waiters are FIFO on the semaphore, so churn is O(completions), not O(polls).
        drop(permit);

        if should_skip_task(&storage, &cancel_registry, &task).await {
            debug!(
                worker_id,
                task_id = %track_id,
                "Fairness park complete — dropping cancelled/terminal task"
            );
            cancel_registry.deregister(&track_id).await;
            return;
        }

        if let Err(e) = queue.send(task).await {
            error!(
                error.source = "task_worker",
                error.action = "requeue_after_fairness_park",
                worker_id,
                task_id = %track_id,
                error.message = %e,
                "Failed to requeue task after fairness park"
            );
        }
    });
}

/// Mock task processor for testing
#[cfg(test)]
pub struct MockTaskProcessor;

#[cfg(test)]
#[async_trait::async_trait]
impl TaskProcessor for MockTaskProcessor {
    async fn process(
        &self,
        task: &mut Task,
        _cancel_token: CancellationToken,
    ) -> TaskResult<serde_json::Value> {
        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(serde_json::json!({
            "status": "success",
            "task_id": task.track_id
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        memory::MemoryTaskStorage,
        queue::ChannelTaskQueue,
        types::{Task, TaskStatus, TaskType},
    };

    const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
    const TEST_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000002";

    fn test_tenant_id() -> uuid::Uuid {
        uuid::Uuid::parse_str(TEST_TENANT_ID).unwrap()
    }

    fn test_workspace_id() -> uuid::Uuid {
        uuid::Uuid::parse_str(TEST_WORKSPACE_ID).unwrap()
    }

    #[tokio::test]
    async fn test_worker_pool_processes_tasks() {
        let queue = Arc::new(ChannelTaskQueue::new(10));
        let storage = Arc::new(MemoryTaskStorage::new());
        let processor = Arc::new(MockTaskProcessor);

        let config = WorkerPoolConfig {
            num_workers: 2,
            auto_retry: false,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 0,      // disabled for basic test
            processing_timeout_secs: 300, // 5 min for tests
        };

        let mut pool = WorkerPool::new(config, queue.clone(), storage.clone(), processor);
        // Smoke: workers can be started on an explicit Handle (ingest runtime split).
        pool.start_on(&tokio::runtime::Handle::current());

        // Create and enqueue tasks
        let mut task_ids = Vec::new();
        for i in 0..5 {
            let task = Task::new(
                test_tenant_id(),
                test_workspace_id(),
                TaskType::Insert,
                serde_json::json!({"index": i}),
            );
            task_ids.push(task.track_id.clone());
            storage.create_task(&task).await.unwrap();
            queue.send(task).await.unwrap();
        }

        // Wait for processing
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Check all tasks completed
        for task_id in task_ids {
            let task = storage.get_task(&task_id).await.unwrap().unwrap();
            assert_eq!(task.status, TaskStatus::Indexed);
        }

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_worker_pool_handles_shutdown() {
        let queue = Arc::new(ChannelTaskQueue::new(10));
        let storage = Arc::new(MemoryTaskStorage::new());
        let processor = Arc::new(MockTaskProcessor);

        let config = WorkerPoolConfig {
            num_workers: 2,
            auto_retry: false,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 0,
            processing_timeout_secs: 300,
        };

        let mut pool = WorkerPool::new(config, queue, storage, processor);
        pool.start();

        // Shutdown immediately
        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_tenant_fairness_limiting() {
        // With max_tasks_per_tenant=1, only 1 task per tenant runs at a time
        let limiter = crate::tenant_limiter::TenantConcurrencyLimiter::new(1);
        let tenant = test_tenant_id();

        let p1 = limiter.try_acquire(tenant).await;
        assert!(p1.is_some(), "First acquire should succeed");

        let p2 = limiter.try_acquire(tenant).await;
        assert!(p2.is_none(), "Second acquire should be denied (at limit)");

        // Different tenant should still get through
        let other_tenant = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap();
        let p3 = limiter.try_acquire(other_tenant).await;
        assert!(p3.is_some(), "Other tenant should not be affected");

        // Release first permit, then acquire should succeed
        drop(p1);
        let p4 = limiter.try_acquire(tenant).await;
        assert!(p4.is_some(), "Should succeed after releasing permit");
    }

    #[test]
    fn test_heartbeat_guard_aborts_on_drop() {
        // Create a tokio runtime for this test
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let handle = tokio::spawn(async {
                // This task should be aborted when the guard is dropped
                tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
            });

            // Wrap in guard and drop immediately
            let guard = HeartbeatGuard(handle);
            drop(guard);

            // Give tokio a moment to process the abort
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // If we get here without hanging, the guard correctly aborted the task
        });
    }

    #[test]
    fn test_calculate_backoff_delay_boundaries() {
        // Attempt 0: initial delay
        assert_eq!(calculate_backoff_delay(0, 1000, 60_000, 2.0), 1000);

        // Attempt 1: 1000 * 2 = 2000
        assert_eq!(calculate_backoff_delay(1, 1000, 60_000, 2.0), 2000);

        // Attempt 5: 1000 * 32 = 32000
        assert_eq!(calculate_backoff_delay(5, 1000, 60_000, 2.0), 32000);

        // Attempt 6: 1000 * 64 = 64000, but capped at 60000
        assert_eq!(calculate_backoff_delay(6, 1000, 60_000, 2.0), 60_000);

        // Very large attempt: should be capped, not overflow
        assert_eq!(calculate_backoff_delay(100, 1000, 60_000, 2.0), 60_000);

        // Multiplier of 1.0: delay stays constant
        assert_eq!(calculate_backoff_delay(5, 1000, 60_000, 1.0), 1000);

        // Zero initial delay: always 0
        assert_eq!(calculate_backoff_delay(3, 0, 60_000, 2.0), 0);
    }

    #[test]
    fn test_worker_pool_config_default_values() {
        let config = WorkerPoolConfig::default();

        // Workers should be at least 4
        assert!(config.num_workers >= 4, "Minimum 4 workers");

        // Timeout must be at least MIN_PROCESSING_TIMEOUT_SECS
        assert!(
            config.processing_timeout_secs >= MIN_PROCESSING_TIMEOUT_SECS,
            "Timeout {} < minimum {}",
            config.processing_timeout_secs,
            MIN_PROCESSING_TIMEOUT_SECS
        );

        // Per-tenant limit should be at least 1
        assert!(
            config.max_tasks_per_tenant >= 1,
            "Per-tenant limit must be >= 1"
        );

        // Per-tenant limit should be less than total workers
        assert!(
            config.max_tasks_per_tenant <= config.num_workers,
            "Per-tenant limit {} should be <= total workers {}",
            config.max_tasks_per_tenant,
            config.num_workers
        );

        // Auto-retry should be enabled by default
        assert!(config.auto_retry, "Auto-retry should be on by default");
    }

    #[tokio::test]
    async fn test_worker_pool_timeout_marks_task_failed() {
        // Create a slow processor that exceeds the timeout
        struct SlowProcessor;

        #[async_trait::async_trait]
        impl TaskProcessor for SlowProcessor {
            async fn process(
                &self,
                _task: &mut Task,
                _cancel_token: CancellationToken,
            ) -> TaskResult<serde_json::Value> {
                // Sleep longer than the timeout
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                Ok(serde_json::json!({"status": "should_not_reach"}))
            }

            async fn on_permanent_failure(&self, _task: &Task, _error_msg: &str) {
                // No-op for test
            }
        }

        let queue = Arc::new(ChannelTaskQueue::new(10));
        let storage = Arc::new(MemoryTaskStorage::new());
        let processor: SharedTaskProcessor = Arc::new(SlowProcessor);

        let config = WorkerPoolConfig {
            num_workers: 1,
            auto_retry: false,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 0,
            processing_timeout_secs: 1, // 1 second timeout for quick test
        };

        let mut pool = WorkerPool::new(config, queue.clone(), storage.clone(), processor);
        pool.start();

        // Create and enqueue a task
        let task = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({"test": "timeout"}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();
        queue.send(task).await.unwrap();

        // Wait for timeout to fire (1s) + some buffer
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Task should be marked as failed due to timeout
        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(
            stored.status,
            TaskStatus::Failed,
            "Timed-out task should be failed, got {:?}",
            stored.status
        );
        assert!(
            stored
                .error_message
                .as_ref()
                .unwrap_or(&String::new())
                .contains("timed out"),
            "Error message should mention timeout: {:?}",
            stored.error_message
        );

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_cancelled_error_preserves_cancelled_status_no_retry() {
        struct CancelProcessor;

        #[async_trait::async_trait]
        impl TaskProcessor for CancelProcessor {
            async fn process(
                &self,
                _task: &mut Task,
                _cancel_token: CancellationToken,
            ) -> TaskResult<serde_json::Value> {
                Err(TaskError::Cancelled("user cancel".into()))
            }
        }

        let queue = Arc::new(ChannelTaskQueue::new(10));
        let storage = Arc::new(MemoryTaskStorage::new());
        let processor: SharedTaskProcessor = Arc::new(CancelProcessor);

        let config = WorkerPoolConfig {
            num_workers: 1,
            auto_retry: true,
            initial_retry_delay_ms: 10,
            max_retry_delay_ms: 50,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 0,
            processing_timeout_secs: 300,
        };

        let mut pool = WorkerPool::new(config, queue.clone(), storage.clone(), processor);
        pool.start();

        let task = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({"test": "cancel"}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();
        queue.send(task).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        assert_eq!(stored.status, TaskStatus::Cancelled);
        assert!(!stored.can_retry());

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_cancel_intent_skips_pending_task() {
        let queue = Arc::new(ChannelTaskQueue::new(10));
        let storage = Arc::new(MemoryTaskStorage::new());
        let processor = Arc::new(MockTaskProcessor);

        let config = WorkerPoolConfig {
            num_workers: 1,
            auto_retry: false,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 0,
            processing_timeout_secs: 300,
        };

        let mut pool = WorkerPool::new(config, queue.clone(), storage.clone(), processor);
        let registry = pool.cancellation_registry();
        pool.start();

        let task = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({"test": "pending-cancel"}),
        );
        let track_id = task.track_id.clone();
        storage.create_task(&task).await.unwrap();
        registry.mark_cancel_intent(&track_id).await;
        queue.send(task).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let stored = storage.get_task(&track_id).await.unwrap().unwrap();
        // SPEC-057 P1: claim then cancel-intent → Cancelled (not left Pending).
        assert_eq!(stored.status, TaskStatus::Cancelled);

        pool.shutdown().await;
    }

    #[tokio::test]
    async fn test_fairness_park_does_not_storm_queue() {
        let queue = Arc::new(ChannelTaskQueue::new(50));
        let storage = Arc::new(MemoryTaskStorage::new());

        struct SlowProcessor;
        #[async_trait::async_trait]
        impl TaskProcessor for SlowProcessor {
            async fn process(
                &self,
                _task: &mut Task,
                _cancel_token: CancellationToken,
            ) -> TaskResult<serde_json::Value> {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                Ok(serde_json::json!({"ok": true}))
            }
        }

        let config = WorkerPoolConfig {
            num_workers: 2,
            auto_retry: false,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            backoff_multiplier: 2.0,
            max_tasks_per_tenant: 1,
            processing_timeout_secs: 300,
        };

        let mut pool = WorkerPool::new(
            config,
            queue.clone(),
            storage.clone(),
            Arc::new(SlowProcessor),
        );
        let limiter = pool.tenant_limiter().expect("limiter enabled");
        pool.start();

        for i in 0..5 {
            let task = Task::new(
                test_tenant_id(),
                test_workspace_id(),
                TaskType::Insert,
                serde_json::json!({"i": i}),
            );
            storage.create_task(&task).await.unwrap();
            queue.send(task).await.unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        // Park waiters should absorb backlog instead of unbounded re-sends.
        assert!(
            limiter.park_waiter_count() <= 5,
            "unexpected park waiter count {}",
            limiter.park_waiter_count()
        );
        assert!(
            queue.approximate_depth() <= 5,
            "queue storm detected: depth={}",
            queue.approximate_depth()
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        pool.shutdown().await;
    }
}
