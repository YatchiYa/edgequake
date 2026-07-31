//! Per-tenant (+ workspace) concurrency limiter for fair task scheduling.
//!
//! ## WHY Per-Tenant Fair Scheduling?
//!
//! Without tenant isolation, one tenant uploading 50 PDFs monopolizes all
//! worker threads, forcing other tenants to wait until the entire batch
//! finishes. This violates multi-tenant fairness guarantees.
//!
//! ## WHY Workspace Lanes (SPEC-084 / GH-316 / LAW-13)?
//!
//! Tenant fairness alone still lets Workspace A’s backlog hold every tenant
//! ingest slot. Ingest acquires nest a per-`(tenant, workspace)` lane under
//! the tenant cap so two workspaces can make forward progress concurrently.
//!
//! ## WHY Operation-Class Lanes?
//!
//! Local LLM clamps protect **Ollama/vision**, not Postgres. Deletion/Wipe are
//! DB-bound; sharing the ingest semaphore lets one delete serialize the whole
//! tenant and starve PdfProcessing (stuck Queued).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::debug;
use uuid::Uuid;

use crate::types::FairnessClass;

/// RAII permit holding tenant (+ optional workspace) ingest/lifecycle slots.
#[derive(Debug)]
pub struct FairnessPermit {
    _tenant: TenantPermit,
    _workspace: Option<OwnedSemaphorePermit>,
}

/// Tenant slot: hard-cap semaphore permit or LAW-Q5 fair-share slot (QW3).
/// Held purely for RAII release-on-drop; never read.
#[derive(Debug)]
#[allow(dead_code)]
enum TenantPermit {
    Semaphore(OwnedSemaphorePermit),
    FairShare(FairShareSlot),
}

/// Outcome of a non-blocking fairness-lane acquire.
#[derive(Debug)]
pub enum TryAcquireOutcome {
    /// Lane is unlimited for this class — proceed without a permit.
    Unlimited,
    /// Acquired a slot; hold until processing completes.
    Acquired(FairnessPermit),
    /// Lane is at capacity — park and wait.
    AtCapacity,
}

/// Snapshot of limiter observability counters.
#[derive(Debug, Clone, Copy, Default)]
pub struct TenantLimiterStats {
    /// Ingest lane max (0 = unlimited / lane disabled).
    pub max_per_tenant: usize,
    /// Lifecycle lane max (0 = unlimited / lane disabled).
    pub max_lifecycle_per_tenant: usize,
    /// Per-workspace ingest max nested under tenant (0 = no workspace lane).
    pub max_per_workspace_ingest: usize,
    pub tracked_tenants: usize,
    /// Aggregated park waiters (ingest + lifecycle).
    pub park_waiters: u64,
    pub park_waiters_ingest: u64,
    pub park_waiters_lifecycle: u64,
    pub park_completions: u64,
    pub park_aborts: u64,
}

/// RAII counter for park waiters. Decrements on drop; records abort unless armed.
struct ParkWaitGuard {
    waiters: Arc<AtomicU64>,
    class_waiters: Arc<AtomicU64>,
    aborts: Arc<AtomicU64>,
    success: bool,
}

impl ParkWaitGuard {
    fn arm_success(&mut self) {
        self.success = true;
    }
}

impl Drop for ParkWaitGuard {
    fn drop(&mut self) {
        self.waiters.fetch_sub(1, Ordering::Relaxed);
        self.class_waiters.fetch_sub(1, Ordering::Relaxed);
        if !self.success {
            self.aborts.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[derive(Clone)]
struct LaneSemaphores {
    max: usize,
    semaphores: Arc<RwLock<HashMap<Uuid, Arc<Semaphore>>>>,
}

impl LaneSemaphores {
    fn new(max: usize) -> Self {
        Self {
            max: max.max(1),
            semaphores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn semaphore_for(&self, key: Uuid) -> Arc<Semaphore> {
        let read_guard = self.semaphores.read().await;
        if let Some(sem) = read_guard.get(&key) {
            return Arc::clone(sem);
        }
        drop(read_guard);
        let mut write_guard = self.semaphores.write().await;
        let sem = write_guard.entry(key).or_insert_with(|| {
            debug!(
                key = %key,
                max_concurrent = self.max,
                "Created fairness-lane semaphore"
            );
            Arc::new(Semaphore::new(self.max))
        });
        Arc::clone(sem)
    }

    async fn try_acquire(&self, key: Uuid) -> Option<OwnedSemaphorePermit> {
        let semaphore = self.semaphore_for(key).await;
        semaphore.try_acquire_owned().ok()
    }

    async fn acquire(&self, key: Uuid) -> Result<OwnedSemaphorePermit, tokio::sync::AcquireError> {
        let semaphore = self.semaphore_for(key).await;
        semaphore.acquire_owned().await
    }

    async fn active_count(&self, key: &Uuid) -> usize {
        let read_guard = self.semaphores.read().await;
        if let Some(sem) = read_guard.get(key) {
            self.max - sem.available_permits()
        } else {
            0
        }
    }

    async fn total_active(&self) -> usize {
        let read_guard = self.semaphores.read().await;
        read_guard
            .values()
            .map(|sem| self.max.saturating_sub(sem.available_permits()))
            .sum()
    }

    async fn tracked_keys(&self) -> usize {
        self.semaphores.read().await.len()
    }

    async fn cleanup_idle(&self) {
        let mut write_guard = self.semaphores.write().await;
        write_guard.retain(|_key, sem| sem.available_permits() < self.max);
    }
}

/// Composite key for nested workspace ingest lanes (tenant-scoped).
fn workspace_lane_key(tenant_id: Uuid, workspace_id: Uuid) -> Uuid {
    // Deterministic UUIDv5-style mix without pulling uuid::Uuid::new_v5 deps:
    // XOR of the two UUIDs is unique enough for semaphore map keys in-process.
    let t = tenant_id.as_u128();
    let w = workspace_id.as_u128();
    Uuid::from_u128(t ^ w.rotate_left(17) ^ 0x0840_0316_u128)
}

// ============================================================================
// SPEC-091 QW3 — Weighted fair-share lane (LAW-Q5, LD-13)
// ============================================================================
//
// The provider budget B is the scarce resource; tenants *divide* it by weight
// instead of holding fixed hard caps. Deficit round-robin (DRR):
//
// - A tenant alone can use the WHOLE budget (no idle-starvation — Axiom Q1).
// - N contending tenants each get ≈ B·wᵢ/Σw (no starvation — LAW-Q5).
// - Deficit counters compensate short-term unfairness (classic DRR).
//
// The cluster-wide invariant stays with the QW1 Postgres ledger (LAW-Q3);
// this lane is the process-local scheduler that divides the budget fairly.

/// DRR deficit cap: a tenant can bank at most 2 quanta (prevents hoarding).
const FAIR_SHARE_DEFICIT_CAP_QUANTA: i64 = 2;

#[derive(Debug, Default)]
struct FairShareInner {
    /// In-flight slots per tenant.
    active: std::collections::HashMap<Uuid, usize>,
    /// DRR deficit counters (in slot units; quantum = weight).
    deficit: std::collections::HashMap<Uuid, i64>,
    /// Σ active across all tenants (invariant: ≤ budget).
    total_active: usize,
    /// Parked waiters per tenant (contention signal + observability).
    waiters: std::collections::HashMap<Uuid, usize>,
}

/// Shared state of one fair-share lane.
#[derive(Debug)]
struct FairShareState {
    /// Provider budget B — total slots divided among tenants.
    budget: usize,
    /// Per-tenant DRR quantum (equal-share deployments use 1 — LAW-Q5).
    weight: u32,
    inner: std::sync::Mutex<FairShareInner>,
    /// Wakes parked tenants when a slot is released.
    notify: tokio::sync::Notify,
}

impl FairShareState {
    /// Try to grant one slot to `key` under DRR (LAW-Q5).
    ///
    /// Round semantics (classic DRR): only tenants with *pending demand*
    /// (parked waiters, plus the caller) participate in round boundaries.
    /// A tenant merely holding slots does not gate a new round — this is
    /// what prevents a saturated-then-releasing tenant's banked deficit
    /// from starving a parked peer.
    fn try_grant(&self, key: Uuid) -> bool {
        let mut inner = self.inner.lock().expect("fair share lane lock");
        if inner.total_active >= self.budget {
            return false;
        }
        let quantum = i64::from(self.weight.max(1));
        if inner.deficit.get(&key).copied().unwrap_or(0) < 1 {
            let demanding: Vec<Uuid> = inner
                .waiters
                .keys()
                .copied()
                .chain(std::iter::once(key))
                .collect();
            let all_depleted = demanding
                .iter()
                .all(|k| inner.deficit.get(k).copied().unwrap_or(0) < 1);
            if all_depleted {
                let cap = quantum * FAIR_SHARE_DEFICIT_CAP_QUANTA;
                for k in demanding {
                    let d = inner.deficit.entry(k).or_insert(0);
                    *d = (*d + quantum).min(cap);
                }
            }
        }
        let deficit = inner.deficit.entry(key).or_insert(0);
        if *deficit < 1 {
            return false;
        }
        *deficit -= 1;
        *inner.active.entry(key).or_insert(0) += 1;
        inner.total_active += 1;
        true
    }

    fn release(&self, key: Uuid) {
        let mut inner = self.inner.lock().expect("fair share lane lock");
        if let Some(a) = inner.active.get_mut(&key) {
            *a = a.saturating_sub(1);
            if *a == 0 {
                inner.active.remove(&key);
            }
        }
        inner.total_active = inner.total_active.saturating_sub(1);
        // Classic DRR: an idle tenant (no in-flight, no pending demand) has
        // its deficit reset so credit cannot be hoarded across idle periods.
        let idle =
            !inner.active.contains_key(&key) && inner.waiters.get(&key).copied().unwrap_or(0) == 0;
        if idle {
            inner.deficit.remove(&key);
        }
        drop(inner);
        // Wake ALL parked waiters: a deficit-blocked woken tenant cannot spend
        // the freed slot, but a co-parked tenant with spendable deficit can
        // (notify_one would lose the wake in exactly that case — EC-20 stall).
        self.notify.notify_waiters();
    }

    fn register_waiter(&self, key: Uuid) {
        let mut inner = self.inner.lock().expect("fair share lane lock");
        *inner.waiters.entry(key).or_insert(0) += 1;
    }

    fn unregister_waiter(&self, key: Uuid) {
        let mut inner = self.inner.lock().expect("fair share lane lock");
        if let Some(w) = inner.waiters.get_mut(&key) {
            *w = w.saturating_sub(1);
            if *w == 0 {
                inner.waiters.remove(&key);
            }
        }
    }
}

/// RAII slot in a fair-share lane; releases on drop (LAW-Q3 discipline).
#[derive(Debug)]
pub struct FairShareSlot {
    state: Arc<FairShareState>,
    key: Uuid,
}

impl Drop for FairShareSlot {
    fn drop(&mut self) {
        self.state.release(self.key);
    }
}

/// Weighted fair-share lane over the provider budget (LAW-Q5).
#[derive(Debug, Clone)]
pub struct FairShareLane {
    state: Arc<FairShareState>,
}

impl FairShareLane {
    pub fn new(budget: usize, tenant_weight: u32) -> Self {
        Self {
            state: Arc::new(FairShareState {
                budget: budget.max(1),
                weight: tenant_weight.max(1),
                inner: std::sync::Mutex::new(FairShareInner::default()),
                notify: tokio::sync::Notify::new(),
            }),
        }
    }

    pub fn budget(&self) -> usize {
        self.state.budget
    }

    pub fn tenant_weight(&self) -> u32 {
        self.state.weight
    }

    fn try_acquire(&self, key: Uuid) -> Option<FairShareSlot> {
        self.state.try_grant(key).then(|| FairShareSlot {
            state: Arc::clone(&self.state),
            key,
        })
    }

    /// Park until DRR grants a slot (woken by releases via `Notify`).
    ///
    /// Liveness invariants (chaos-proven, EC-20):
    /// 1. `release` uses `notify_waiters` (not `notify_one`): a single woken
    ///    waiter may be deficit-blocked while ANOTHER tenant holds spendable
    ///    deficit — waking only it loses the wake with capacity still idle.
    /// 2. The `enable()` + double-check pattern closes the registration race:
    ///    a release landing between the first failed `try_acquire` and the
    ///    park would otherwise be missed (`notify_waiters` stores no permit
    ///    for late registrants); the second check observes the freed slot.
    async fn acquire(&self, key: Uuid) -> FairShareSlot {
        self.state.register_waiter(key);
        let _wait_guard = FairShareWaitGuard {
            state: Arc::clone(&self.state),
            key,
        };
        loop {
            if let Some(slot) = self.try_acquire(key) {
                return slot;
            }
            let notified = self.state.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if let Some(slot) = self.try_acquire(key) {
                return slot;
            }
            notified.await;
        }
    }

    async fn active_count(&self, key: &Uuid) -> usize {
        let inner = self.state.inner.lock().expect("fair share lane lock");
        inner.active.get(key).copied().unwrap_or(0)
    }

    async fn total_active(&self) -> usize {
        let inner = self.state.inner.lock().expect("fair share lane lock");
        inner.total_active
    }

    async fn tracked_keys(&self) -> usize {
        let inner = self.state.inner.lock().expect("fair share lane lock");
        inner.active.len().max(inner.waiters.len())
    }
}

/// RAII waiter deregistration for the fair-share park loop.
struct FairShareWaitGuard {
    state: Arc<FairShareState>,
    key: Uuid,
}

impl Drop for FairShareWaitGuard {
    fn drop(&mut self) {
        self.state.unregister_waiter(self.key);
    }
}

/// Ingest lane: legacy hard caps, or LAW-Q5 weighted fair-share of the
/// provider budget (QW3). Lifecycle lane stays hard-capped (DB-bound class).
#[derive(Clone)]
enum IngestLane {
    Capped(LaneSemaphores),
    /// SPEC-091 hardening (provider-keyed LAW-Q5): one DRR lane per LOCAL
    /// provider key, each sized from the same cluster provider budget.
    /// Cloud-effective tasks bypass the lane entirely (callers return
    /// `Unlimited`) — they do not consume the scarce local-model capacity.
    FairShareKeyed(FairShareLaneMap),
}

/// Provider-keyed fair-share lanes. Lanes are created lazily on first use of
/// a provider key; a single-key deployment degenerates to exactly one lane
/// (behavior identical to the pre-hardening single lane).
#[derive(Clone)]
struct FairShareLaneMap {
    budget: usize,
    tenant_weight: u32,
    lanes: Arc<std::sync::RwLock<std::collections::HashMap<String, FairShareLane>>>,
}

impl FairShareLaneMap {
    fn new(budget: usize, tenant_weight: u32) -> Self {
        Self {
            budget,
            tenant_weight,
            lanes: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn lane_for(&self, key: &str) -> FairShareLane {
        if let Some(lane) = self
            .lanes
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(key)
        {
            return lane.clone();
        }
        let mut lanes = self.lanes.write().unwrap_or_else(|e| e.into_inner());
        lanes
            .entry(key.to_string())
            .or_insert_with(|| FairShareLane::new(self.budget, self.tenant_weight))
            .clone()
    }

    async fn total_active(&self) -> usize {
        let lanes: Vec<FairShareLane> = self
            .lanes
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect();
        let mut total = 0;
        for lane in lanes {
            total += lane.total_active().await;
        }
        total
    }

    async fn tracked_keys(&self) -> usize {
        let lanes: Vec<FairShareLane> = self
            .lanes
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect();
        let mut n = 0;
        for lane in lanes {
            n = n.max(lane.tracked_keys().await);
        }
        n
    }
}

/// Per-tenant concurrency limiter with ingest vs lifecycle lanes and
/// nested per-workspace ingest lanes (SPEC-084 / GH-316).
#[derive(Clone)]
pub struct TenantConcurrencyLimiter {
    /// `None` = unlimited for ingest class. QW3: may be a LAW-Q5 fair-share
    /// lane over the provider budget instead of hard caps.
    ingest: Option<IngestLane>,
    /// `None` = unlimited for lifecycle class.
    lifecycle: Option<LaneSemaphores>,
    /// Nested under tenant ingest: max concurrent ingest per (tenant, workspace).
    workspace_ingest: Option<LaneSemaphores>,
    park_waiters: Arc<AtomicU64>,
    park_waiters_ingest: Arc<AtomicU64>,
    park_waiters_lifecycle: Arc<AtomicU64>,
    park_completions: Arc<AtomicU64>,
    park_aborts: Arc<AtomicU64>,
}

impl TenantConcurrencyLimiter {
    /// Create a dual-lane limiter with default workspace ingest lane of 1.
    ///
    /// `max_ingest` / `max_lifecycle`: `0` means that lane is unlimited (no park).
    /// When ingest is limited, each workspace may hold at most **1** ingest slot
    /// under the tenant cap (LAW-13 interleave).
    pub fn new(max_ingest: usize, max_lifecycle: usize) -> Self {
        let max_workspace = if max_ingest > 0 { 1 } else { 0 };
        Self::new_with_workspace(max_ingest, max_lifecycle, max_workspace)
    }

    /// Explicit workspace ingest cap (0 = no nested workspace lane).
    pub fn new_with_workspace(
        max_ingest: usize,
        max_lifecycle: usize,
        max_workspace_ingest: usize,
    ) -> Self {
        Self {
            ingest: (max_ingest > 0).then(|| IngestLane::Capped(LaneSemaphores::new(max_ingest))),
            lifecycle: (max_lifecycle > 0).then(|| LaneSemaphores::new(max_lifecycle)),
            workspace_ingest: (max_workspace_ingest > 0)
                .then(|| LaneSemaphores::new(max_workspace_ingest)),
            park_waiters: Arc::new(AtomicU64::new(0)),
            park_waiters_ingest: Arc::new(AtomicU64::new(0)),
            park_waiters_lifecycle: Arc::new(AtomicU64::new(0)),
            park_completions: Arc::new(AtomicU64::new(0)),
            park_aborts: Arc::new(AtomicU64::new(0)),
        }
    }

    /// SPEC-091 QW3 (LAW-Q5, LD-13): ingest lane becomes a weighted fair-share
    /// of the provider budget (DRR) instead of a hard per-tenant cap.
    ///
    /// - A tenant alone uses the whole budget (no idle-starvation, Axiom Q1).
    /// - Contended tenants share ≈ budget·wᵢ/Σw (no starvation).
    /// - Lifecycle lane keeps its hard cap (DB-bound class preserved).
    pub fn new_fair_share(
        provider_budget: usize,
        tenant_weight: u32,
        max_lifecycle: usize,
    ) -> Self {
        Self {
            ingest: Some(IngestLane::FairShareKeyed(FairShareLaneMap::new(
                provider_budget,
                tenant_weight,
            ))),
            lifecycle: (max_lifecycle > 0).then(|| LaneSemaphores::new(max_lifecycle)),
            // Workspace interleave preserved (LAW-13): one ingest slot per
            // (tenant, workspace) under the tenant's fair share.
            workspace_ingest: Some(LaneSemaphores::new(1)),
            park_waiters: Arc::new(AtomicU64::new(0)),
            park_waiters_ingest: Arc::new(AtomicU64::new(0)),
            park_waiters_lifecycle: Arc::new(AtomicU64::new(0)),
            park_completions: Arc::new(AtomicU64::new(0)),
            park_aborts: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Backward-compatible constructor: both lanes share the same max.
    pub fn new_unified(max_per_tenant: usize) -> Self {
        Self::new(max_per_tenant, max_per_tenant)
    }

    fn class_waiters(&self, class: FairnessClass) -> &Arc<AtomicU64> {
        match class {
            FairnessClass::Ingest => &self.park_waiters_ingest,
            FairnessClass::Lifecycle => &self.park_waiters_lifecycle,
        }
    }

    /// Hard-cap semaphore lane (ingest capped variant / lifecycle).
    fn lane(&self, class: FairnessClass) -> Option<&LaneSemaphores> {
        match class {
            FairnessClass::Ingest => match self.ingest.as_ref() {
                Some(IngestLane::Capped(lane)) => Some(lane),
                _ => None,
            },
            FairnessClass::Lifecycle => self.lifecycle.as_ref(),
        }
    }

    /// Whether this class is capacity-limited (false → never park).
    pub fn limits_class(&self, class: FairnessClass) -> bool {
        match class {
            FairnessClass::Ingest => self.ingest.is_some(),
            FairnessClass::Lifecycle => self.lifecycle.is_some(),
        }
    }

    /// Try to acquire a processing slot for tenant + workspace + fairness class.
    /// Try to acquire a processing slot for tenant + workspace + fairness class.
    ///
    /// `provider` is the task's EFFECTIVE provider class (SPEC-091 hardening):
    /// under the fair-share lane, `Cloud` tasks bypass the local-budget lane
    /// (`Unlimited`); `Local(key)` tasks use the lane keyed by that provider.
    /// The legacy hard-cap lane ignores `provider` (cloud-server deployments
    /// keep per-tenant caps by design — see spec 13 §QW3).
    pub async fn try_acquire(
        &self,
        tenant_id: Uuid,
        workspace_id: Uuid,
        class: FairnessClass,
        provider: &crate::provider_class::TaskProviderClass,
    ) -> TryAcquireOutcome {
        // QW3: fair-share ingest lane grants by DRR over the provider budget.
        if class == FairnessClass::Ingest {
            if let Some(IngestLane::FairShareKeyed(map)) = &self.ingest {
                let Some(provider_key) = provider.lane_key() else {
                    // Cloud-effective task: the local provider budget is not
                    // this task's scarce resource — do not throttle on it.
                    return TryAcquireOutcome::Unlimited;
                };
                let lane = map.lane_for(provider_key);
                let Some(slot) = lane.try_acquire(tenant_id) else {
                    return TryAcquireOutcome::AtCapacity;
                };
                let workspace_permit = if let Some(ws_lane) = &self.workspace_ingest {
                    let key = workspace_lane_key(tenant_id, workspace_id);
                    match ws_lane.try_acquire(key).await {
                        Some(p) => Some(p),
                        None => {
                            drop(slot);
                            return TryAcquireOutcome::AtCapacity;
                        }
                    }
                } else {
                    None
                };
                return TryAcquireOutcome::Acquired(FairnessPermit {
                    _tenant: TenantPermit::FairShare(slot),
                    _workspace: workspace_permit,
                });
            }
        }

        let Some(lane) = self.lane(class) else {
            return TryAcquireOutcome::Unlimited;
        };
        let Some(tenant_permit) = lane.try_acquire(tenant_id).await else {
            return TryAcquireOutcome::AtCapacity;
        };

        let workspace_permit = if class == FairnessClass::Ingest {
            if let Some(ws_lane) = &self.workspace_ingest {
                let key = workspace_lane_key(tenant_id, workspace_id);
                match ws_lane.try_acquire(key).await {
                    Some(p) => Some(p),
                    None => {
                        // Drop tenant permit by not wrapping it.
                        drop(tenant_permit);
                        return TryAcquireOutcome::AtCapacity;
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        TryAcquireOutcome::Acquired(FairnessPermit {
            _tenant: TenantPermit::Semaphore(tenant_permit),
            _workspace: workspace_permit,
        })
    }

    /// Park until a processing slot is available for the tenant + workspace + class.
    /// Park until a processing slot is available for the tenant + workspace + class.
    ///
    /// Only called after a `try_acquire` miss, so `provider` is already known
    /// to be lane-gated; the Cloud guard is repeated here defensively (a
    /// workspace provider flip between claim and park must not park a cloud
    /// task on the local-budget lane).
    pub async fn acquire(
        &self,
        tenant_id: Uuid,
        workspace_id: Uuid,
        class: FairnessClass,
        provider: &crate::provider_class::TaskProviderClass,
    ) -> Result<FairnessPermit, tokio::sync::AcquireError> {
        // QW3: fair-share ingest lane parks on DRR (Notify-woken).
        if class == FairnessClass::Ingest {
            if let Some(IngestLane::FairShareKeyed(map)) = &self.ingest {
                if provider.lane_key().is_none() {
                    // Defensive: cloud tasks never park on the local lane.
                    let sem = Arc::new(Semaphore::new(1));
                    let permit = sem.acquire_owned().await?;
                    return Ok(FairnessPermit {
                        _tenant: TenantPermit::Semaphore(permit),
                        _workspace: None,
                    });
                }
                let lane = map.lane_for(provider.lane_key().expect("checked above"));
                let class_waiters = Arc::clone(self.class_waiters(class));
                self.park_waiters.fetch_add(1, Ordering::Relaxed);
                class_waiters.fetch_add(1, Ordering::Relaxed);
                let mut wait_guard = ParkWaitGuard {
                    waiters: Arc::clone(&self.park_waiters),
                    class_waiters,
                    aborts: Arc::clone(&self.park_aborts),
                    success: false,
                };

                let slot = lane.acquire(tenant_id).await;
                let workspace_permit = if let Some(ws_lane) = &self.workspace_ingest {
                    let key = workspace_lane_key(tenant_id, workspace_id);
                    match ws_lane.try_acquire(key).await {
                        Some(p) => Some(p),
                        None => {
                            drop(slot);
                            let ws_permit = ws_lane.acquire(key).await?;
                            let slot = lane.acquire(tenant_id).await;
                            // Rebuild with both held.
                            wait_guard.arm_success();
                            self.park_completions.fetch_add(1, Ordering::Relaxed);
                            return Ok(FairnessPermit {
                                _tenant: TenantPermit::FairShare(slot),
                                _workspace: Some(ws_permit),
                            });
                        }
                    }
                } else {
                    None
                };
                wait_guard.arm_success();
                self.park_completions.fetch_add(1, Ordering::Relaxed);
                return Ok(FairnessPermit {
                    _tenant: TenantPermit::FairShare(slot),
                    _workspace: workspace_permit,
                });
            }
        }

        let Some(lane) = self.lane(class) else {
            let sem = Arc::new(Semaphore::new(1));
            let permit = sem.acquire_owned().await?;
            return Ok(FairnessPermit {
                _tenant: TenantPermit::Semaphore(permit),
                _workspace: None,
            });
        };
        let class_waiters = Arc::clone(self.class_waiters(class));
        self.park_waiters.fetch_add(1, Ordering::Relaxed);
        class_waiters.fetch_add(1, Ordering::Relaxed);
        let mut wait_guard = ParkWaitGuard {
            waiters: Arc::clone(&self.park_waiters),
            class_waiters,
            aborts: Arc::clone(&self.park_aborts),
            success: false,
        };

        // Tenant then workspace (workspace only for ingest).
        // On workspace miss, drop tenant, wait for workspace, then re-acquire tenant.
        let tenant_permit = lane.acquire(tenant_id).await?;
        if class != FairnessClass::Ingest {
            wait_guard.arm_success();
            self.park_completions.fetch_add(1, Ordering::Relaxed);
            return Ok(FairnessPermit {
                _tenant: TenantPermit::Semaphore(tenant_permit),
                _workspace: None,
            });
        }
        let Some(ws_lane) = &self.workspace_ingest else {
            wait_guard.arm_success();
            self.park_completions.fetch_add(1, Ordering::Relaxed);
            return Ok(FairnessPermit {
                _tenant: TenantPermit::Semaphore(tenant_permit),
                _workspace: None,
            });
        };
        let key = workspace_lane_key(tenant_id, workspace_id);
        let (tenant_permit, workspace_permit) = match ws_lane.try_acquire(key).await {
            Some(ws_permit) => (tenant_permit, ws_permit),
            None => {
                drop(tenant_permit);
                let ws_permit = ws_lane.acquire(key).await?;
                let tenant_permit = lane.acquire(tenant_id).await?;
                (tenant_permit, ws_permit)
            }
        };
        wait_guard.arm_success();
        self.park_completions.fetch_add(1, Ordering::Relaxed);
        Ok(FairnessPermit {
            _tenant: TenantPermit::Semaphore(tenant_permit),
            _workspace: Some(workspace_permit),
        })
    }

    pub async fn active_count(&self, tenant_id: &Uuid, class: FairnessClass) -> usize {
        if class == FairnessClass::Ingest {
            if let Some(IngestLane::FairShareKeyed(map)) = &self.ingest {
                let lanes: Vec<FairShareLane> = map
                    .lanes
                    .read()
                    .unwrap_or_else(|e| e.into_inner())
                    .values()
                    .cloned()
                    .collect();
                let mut n = 0;
                for lane in lanes {
                    n += lane.active_count(tenant_id).await;
                }
                return n;
            }
        }
        match self.lane(class) {
            Some(lane) => lane.active_count(tenant_id).await,
            None => 0,
        }
    }

    pub async fn total_active(&self) -> usize {
        let mut total = 0usize;
        match &self.ingest {
            Some(IngestLane::Capped(lane)) => total += lane.total_active().await,
            Some(IngestLane::FairShareKeyed(map)) => total += map.total_active().await,
            None => {}
        }
        if let Some(lane) = &self.lifecycle {
            total += lane.total_active().await;
        }
        total
    }

    pub async fn stats(&self) -> TenantLimiterStats {
        let tracked = {
            let mut n = 0usize;
            match &self.ingest {
                Some(IngestLane::Capped(lane)) => n = n.max(lane.tracked_keys().await),
                Some(IngestLane::FairShareKeyed(map)) => n = n.max(map.tracked_keys().await),
                None => {}
            }
            if let Some(lane) = &self.lifecycle {
                n = n.max(lane.tracked_keys().await);
            }
            n
        };
        let park_waiters_ingest = self.park_waiters_ingest.load(Ordering::Relaxed);
        let park_waiters_lifecycle = self.park_waiters_lifecycle.load(Ordering::Relaxed);
        TenantLimiterStats {
            max_per_tenant: self.max_per_tenant(),
            max_lifecycle_per_tenant: self.lifecycle.as_ref().map(|l| l.max).unwrap_or(0),
            max_per_workspace_ingest: self.workspace_ingest.as_ref().map(|l| l.max).unwrap_or(0),
            tracked_tenants: tracked,
            park_waiters: park_waiters_ingest + park_waiters_lifecycle,
            park_waiters_ingest,
            park_waiters_lifecycle,
            park_completions: self.park_completions.load(Ordering::Relaxed),
            park_aborts: self.park_aborts.load(Ordering::Relaxed),
        }
    }

    pub async fn cleanup_idle(&self) {
        if let Some(IngestLane::Capped(lane)) = &self.ingest {
            lane.cleanup_idle().await;
        }
        if let Some(lane) = &self.lifecycle {
            lane.cleanup_idle().await;
        }
        if let Some(lane) = &self.workspace_ingest {
            lane.cleanup_idle().await;
        }
    }

    /// Ingest lane max: hard cap, or provider budget under fair-share (QW3 —
    /// a lone tenant may use the whole budget; DRR shares it when contended).
    /// Under provider-keyed lanes this is the default-lane budget.
    pub fn max_per_tenant(&self) -> usize {
        match &self.ingest {
            Some(IngestLane::Capped(lane)) => lane.max,
            Some(IngestLane::FairShareKeyed(map)) => map
                .lane_for(crate::provider_class::LOCAL_LANE_DEFAULT_KEY)
                .budget(),
            None => 0,
        }
    }

    pub fn max_lifecycle_per_tenant(&self) -> usize {
        self.lifecycle.as_ref().map(|l| l.max).unwrap_or(0)
    }

    pub fn max_per_workspace_ingest(&self) -> usize {
        self.workspace_ingest.as_ref().map(|l| l.max).unwrap_or(0)
    }

    pub fn park_waiter_count(&self) -> u64 {
        self.park_waiters.load(Ordering::Relaxed)
    }
}

impl std::fmt::Debug for TenantConcurrencyLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TenantConcurrencyLimiter")
            .field("max_ingest", &self.max_per_tenant())
            .field("max_lifecycle", &self.max_lifecycle_per_tenant())
            .field("max_workspace_ingest", &self.max_per_workspace_ingest())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant_a() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    /// Test provider class: local lane on the shared default key.
    fn local_provider() -> crate::provider_class::TaskProviderClass {
        crate::provider_class::TaskProviderClass::Local(
            crate::provider_class::LOCAL_LANE_DEFAULT_KEY.to_string(),
        )
    }

    fn tenant_b() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    fn ws_a() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap()
    }

    fn ws_b() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-0000000000bb").unwrap()
    }

    #[tokio::test]
    async fn test_basic_acquire_release() {
        let limiter = TenantConcurrencyLimiter::new_with_workspace(2, 2, 0);

        let permit1 = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert_eq!(
            limiter
                .active_count(&tenant_a(), FairnessClass::Ingest)
                .await,
            1
        );

        let permit2 = match limiter
            .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::AtCapacity
        ));

        drop(permit1);
        let _permit3 = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired after release, got {other:?}"),
        };
        drop(permit2);
    }

    #[tokio::test]
    async fn lifecycle_lane_independent_of_ingest() {
        let limiter = TenantConcurrencyLimiter::new(1, 2);
        let _ingest = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(
                    tenant_a(),
                    ws_a(),
                    FairnessClass::Lifecycle,
                    &local_provider()
                )
                .await,
            TryAcquireOutcome::Acquired(_)
        ));
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::AtCapacity
        ));
    }

    #[tokio::test]
    async fn workspace_lane_allows_two_workspaces_under_tenant_cap() {
        let limiter = TenantConcurrencyLimiter::new(2, 2);
        assert_eq!(limiter.max_per_workspace_ingest(), 1);

        let _a = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        let _b = match limiter
            .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired for second workspace, got {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::AtCapacity
        ));
    }

    #[tokio::test]
    async fn test_park_acquire_waits_for_release() {
        let limiter = TenantConcurrencyLimiter::new_with_workspace(1, 1, 0);
        let held = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };

        let limiter2 = limiter.clone();
        let waiter = tokio::spawn(async move {
            limiter2
                .acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await
                .unwrap()
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(limiter.park_waiter_count(), 1);

        drop(held);
        let parked = tokio::time::timeout(tokio::time::Duration::from_secs(2), waiter)
            .await
            .expect("join")
            .expect("spawn");
        drop(parked);
        assert_eq!(limiter.stats().await.park_completions, 1);
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let limiter = TenantConcurrencyLimiter::new(1, 1);

        let _permit_a = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        let _permit_b = match limiter
            .try_acquire(tenant_b(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::AtCapacity
        ));
    }

    #[tokio::test]
    async fn unlimited_ingest_lane() {
        let limiter = TenantConcurrencyLimiter::new(0, 2);
        assert!(!limiter.limits_class(FairnessClass::Ingest));
        assert!(limiter.limits_class(FairnessClass::Lifecycle));
    }

    #[tokio::test]
    async fn test_cleanup_idle() {
        let limiter = TenantConcurrencyLimiter::new_with_workspace(2, 2, 0);
        let permit = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        drop(permit);
        limiter.cleanup_idle().await;
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::Acquired(_)
        ));
    }

    // ====================================================================
    // SPEC-091 QW3 — LAW-Q5 weighted fair-share (DRR) contract tests
    // ====================================================================

    /// LAW-Q5 invariant: total in-flight NEVER exceeds the provider budget,
    /// however many tenants contend (EC-20).
    #[tokio::test]
    async fn contract_spec091_fair_share_never_exceeds_budget() {
        let limiter = TenantConcurrencyLimiter::new_fair_share(2, 1, 0);
        let a1 = limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await;
        let a2 = limiter
            .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
            .await;
        let b1 = limiter
            .try_acquire(tenant_b(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await;
        let held: Vec<_> = [a1, a2, b1]
            .into_iter()
            .filter_map(|o| match o {
                TryAcquireOutcome::Acquired(p) => Some(p),
                _ => None,
            })
            .collect();
        assert!(held.len() <= 2, "in-flight ≤ provider budget (LAW-Q3/Q5)");
        assert_eq!(limiter.total_active().await, held.len());
    }

    /// Cloud-effective tasks bypass the local-budget fair-share lane entirely:
    /// they are Unlimited even when the local lane is saturated (SPEC-091
    /// hardening — a saturated local model must not throttle cloud work).
    #[tokio::test]
    async fn contract_spec091_cloud_tasks_bypass_fair_share_lane() {
        let cloud = crate::provider_class::TaskProviderClass::Cloud;
        // Budget 1: saturate the local lane with one local tenant.
        let limiter = TenantConcurrencyLimiter::new_fair_share(1, 1, 0);
        let held = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("first local acquire must succeed: {other:?}"),
        };
        // Second local task is at capacity…
        assert!(matches!(
            limiter
                .try_acquire(tenant_b(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::AtCapacity
        ));
        // …but cloud tasks are Unlimited (they never touch the local budget).
        for _ in 0..4 {
            assert!(matches!(
                limiter
                    .try_acquire(tenant_b(), ws_a(), FairnessClass::Ingest, &cloud)
                    .await,
                TryAcquireOutcome::Unlimited
            ));
        }
        // Cloud tasks contribute nothing to the lane's active count.
        assert_eq!(limiter.total_active().await, 1);
        drop(held);
    }

    /// Distinct local providers get independent DRR lanes: saturating
    /// provider A's lane must not throttle provider B's tasks (per-provider
    /// budget keying, LAW-Q5 refinement).
    #[tokio::test]
    async fn contract_spec091_local_lanes_keyed_by_provider() {
        let ollama = crate::provider_class::TaskProviderClass::Local("ollama".to_string());
        let lmstudio = crate::provider_class::TaskProviderClass::Local("lmstudio".to_string());
        let limiter = TenantConcurrencyLimiter::new_fair_share(1, 1, 0);
        // Saturate the ollama lane.
        let held = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &ollama)
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("first ollama acquire must succeed: {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &ollama)
                .await,
            TryAcquireOutcome::AtCapacity
        ));
        // The lmstudio lane has its own budget → same tenant acquires freely.
        let held2 = match limiter
            .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &lmstudio)
            .await
        {
            TryAcquireOutcome::Acquired(p2) => p2,
            other => panic!("lmstudio lane must be independent of ollama: {other:?}"),
        };
        assert_eq!(limiter.total_active().await, 2);
        drop(held);
        drop(held2);
    }

    /// Axiom Q1: a lone tenant can use the WHOLE budget — no idle-starvation
    /// from hard per-tenant caps.
    #[tokio::test]
    async fn contract_spec091_fair_share_lone_tenant_uses_full_budget() {
        let limiter = TenantConcurrencyLimiter::new_fair_share(4, 1, 0);
        // Budget 4: tenant A may hold up to 4 in-flight (workspace lane caps
        // interleave at 1 per workspace — use 4 distinct workspaces).
        let ws = [ws_a(), ws_b(), Uuid::new_v4(), Uuid::new_v4()];
        let mut held = Vec::new();
        for w in ws {
            match limiter
                .try_acquire(tenant_a(), w, FairnessClass::Ingest, &local_provider())
                .await
            {
                TryAcquireOutcome::Acquired(p) => held.push(p),
                other => panic!("lone tenant must not be capped below budget: {other:?}"),
            }
        }
        assert_eq!(limiter.total_active().await, 4);
        assert!(
            matches!(
                limiter
                    .try_acquire(
                        tenant_a(),
                        Uuid::new_v4(),
                        FairnessClass::Ingest,
                        &local_provider()
                    )
                    .await,
                TryAcquireOutcome::AtCapacity
            ),
            "budget itself is the only bound"
        );
        drop(held);
    }

    /// LAW-Q5: under contention DRR interleaves tenants — a saturated tenant
    /// cannot starve a peer (parked B acquires as A releases).
    #[tokio::test]
    async fn contract_spec091_fair_share_contended_tenants_interleave() {
        let limiter = TenantConcurrencyLimiter::new_fair_share(2, 1, 0);
        // Tenant A saturates the budget (two workspaces for the lane cap).
        let _a1 = match limiter
            .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("{other:?}"),
        };
        let a2 = match limiter
            .try_acquire(tenant_a(), ws_b(), FairnessClass::Ingest, &local_provider())
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("{other:?}"),
        };
        // B parks on DRR (budget full).
        let limiter2 = limiter.clone();
        let parked = tokio::spawn(async move {
            limiter2
                .acquire(tenant_b(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await
                .expect("permit")
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(limiter.stats().await.park_waiters_ingest, 1);
        // A releases one slot → parked B wakes and acquires (no starvation).
        drop(a2);
        let _b = tokio::time::timeout(tokio::time::Duration::from_secs(2), parked)
            .await
            .expect("B must acquire after A releases")
            .expect("join");
        assert!(
            limiter
                .active_count(&tenant_b(), FairnessClass::Ingest)
                .await
                >= 1
        );
    }

    /// Lifecycle class stays independent under fair-share ingest (classes
    /// preserved per plan: ingest shares the provider budget, lifecycle keeps
    /// its DB-bound hard cap).
    #[tokio::test]
    async fn contract_spec091_fair_share_lifecycle_class_preserved() {
        let limiter = TenantConcurrencyLimiter::new_fair_share(2, 1, 2);
        let _l1 = match limiter
            .try_acquire(
                tenant_a(),
                ws_a(),
                FairnessClass::Lifecycle,
                &local_provider(),
            )
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("{other:?}"),
        };
        let _l2 = match limiter
            .try_acquire(
                tenant_a(),
                ws_b(),
                FairnessClass::Lifecycle,
                &local_provider(),
            )
            .await
        {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("{other:?}"),
        };
        assert!(
            matches!(
                limiter
                    .try_acquire(
                        tenant_a(),
                        Uuid::new_v4(),
                        FairnessClass::Lifecycle,
                        &local_provider()
                    )
                    .await,
                TryAcquireOutcome::AtCapacity
            ),
            "lifecycle hard cap preserved"
        );
        // Ingest unaffected by lifecycle saturation.
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), ws_a(), FairnessClass::Ingest, &local_provider())
                .await,
            TryAcquireOutcome::Acquired(_)
        ));
    }
}
