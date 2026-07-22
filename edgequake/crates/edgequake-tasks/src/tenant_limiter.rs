//! Per-tenant concurrency limiter for fair task scheduling.
//!
//! ## WHY Per-Tenant Fair Scheduling?
//!
//! Without tenant isolation, one tenant uploading 50 PDFs monopolizes all
//! worker threads, forcing other tenants to wait until the entire batch
//! finishes. This violates multi-tenant fairness guarantees.
//!
//! ## WHY Operation-Class Lanes?
//!
//! Local LLM clamps protect **Ollama/vision**, not Postgres. Deletion/Wipe are
//! DB-bound; sharing the ingest semaphore lets one delete serialize the whole
//! tenant and starve PdfProcessing (stuck Queued).
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │  Worker picks task ──► FairnessClass (Ingest|Lifecycle)   │
//! │         │                   │                             │
//! │         │                   try_acquire(tenant, class)?   │
//! │         │                   YES → run with permit         │
//! │         │                   NO  → park on that lane       │
//! └───────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::debug;
use uuid::Uuid;

use crate::types::FairnessClass;

/// Outcome of a non-blocking fairness-lane acquire.
#[derive(Debug)]
pub enum TryAcquireOutcome {
    /// Lane is unlimited for this class — proceed without a permit.
    Unlimited,
    /// Acquired a slot; hold until processing completes.
    Acquired(OwnedSemaphorePermit),
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

    async fn semaphore_for(&self, tenant_id: Uuid) -> Arc<Semaphore> {
        let read_guard = self.semaphores.read().await;
        if let Some(sem) = read_guard.get(&tenant_id) {
            return Arc::clone(sem);
        }
        drop(read_guard);
        let mut write_guard = self.semaphores.write().await;
        let sem = write_guard.entry(tenant_id).or_insert_with(|| {
            debug!(
                tenant_id = %tenant_id,
                max_concurrent = self.max,
                "Created tenant fairness-lane semaphore"
            );
            Arc::new(Semaphore::new(self.max))
        });
        Arc::clone(sem)
    }

    async fn try_acquire(&self, tenant_id: Uuid) -> Option<OwnedSemaphorePermit> {
        let semaphore = self.semaphore_for(tenant_id).await;
        semaphore.try_acquire_owned().ok()
    }

    async fn acquire(
        &self,
        tenant_id: Uuid,
    ) -> Result<OwnedSemaphorePermit, tokio::sync::AcquireError> {
        let semaphore = self.semaphore_for(tenant_id).await;
        semaphore.acquire_owned().await
    }

    async fn active_count(&self, tenant_id: &Uuid) -> usize {
        let read_guard = self.semaphores.read().await;
        if let Some(sem) = read_guard.get(tenant_id) {
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

    async fn tracked_tenants(&self) -> usize {
        self.semaphores.read().await.len()
    }

    async fn cleanup_idle(&self) {
        let mut write_guard = self.semaphores.write().await;
        write_guard.retain(|_tenant_id, sem| sem.available_permits() < self.max);
    }
}

/// Per-tenant concurrency limiter with ingest vs lifecycle lanes.
#[derive(Clone)]
pub struct TenantConcurrencyLimiter {
    /// `None` = unlimited for ingest class.
    ingest: Option<LaneSemaphores>,
    /// `None` = unlimited for lifecycle class.
    lifecycle: Option<LaneSemaphores>,
    park_waiters: Arc<AtomicU64>,
    park_waiters_ingest: Arc<AtomicU64>,
    park_waiters_lifecycle: Arc<AtomicU64>,
    park_completions: Arc<AtomicU64>,
    park_aborts: Arc<AtomicU64>,
}

impl TenantConcurrencyLimiter {
    /// Create a dual-lane limiter.
    ///
    /// `max_ingest` / `max_lifecycle`: `0` means that lane is unlimited (no park).
    /// At least one lane must be `> 0` or construction is a no-op wrapper with
    /// both unlimited (caller should use `None` on the worker pool instead).
    pub fn new(max_ingest: usize, max_lifecycle: usize) -> Self {
        Self {
            ingest: (max_ingest > 0).then(|| LaneSemaphores::new(max_ingest)),
            lifecycle: (max_lifecycle > 0).then(|| LaneSemaphores::new(max_lifecycle)),
            park_waiters: Arc::new(AtomicU64::new(0)),
            park_waiters_ingest: Arc::new(AtomicU64::new(0)),
            park_waiters_lifecycle: Arc::new(AtomicU64::new(0)),
            park_completions: Arc::new(AtomicU64::new(0)),
            park_aborts: Arc::new(AtomicU64::new(0)),
        }
    }

    fn class_waiters(&self, class: FairnessClass) -> &Arc<AtomicU64> {
        match class {
            FairnessClass::Ingest => &self.park_waiters_ingest,
            FairnessClass::Lifecycle => &self.park_waiters_lifecycle,
        }
    }

    /// Backward-compatible constructor: both lanes share the same max.
    pub fn new_unified(max_per_tenant: usize) -> Self {
        Self::new(max_per_tenant, max_per_tenant)
    }

    fn lane(&self, class: FairnessClass) -> Option<&LaneSemaphores> {
        match class {
            FairnessClass::Ingest => self.ingest.as_ref(),
            FairnessClass::Lifecycle => self.lifecycle.as_ref(),
        }
    }

    /// Whether this class is capacity-limited (false → never park).
    pub fn limits_class(&self, class: FairnessClass) -> bool {
        self.lane(class).is_some()
    }

    /// Try to acquire a processing slot for the given tenant + fairness class.
    pub async fn try_acquire(&self, tenant_id: Uuid, class: FairnessClass) -> TryAcquireOutcome {
        match self.lane(class) {
            None => TryAcquireOutcome::Unlimited,
            Some(lane) => match lane.try_acquire(tenant_id).await {
                Some(permit) => TryAcquireOutcome::Acquired(permit),
                None => TryAcquireOutcome::AtCapacity,
            },
        }
    }

    /// Park until a processing slot is available for the tenant + class.
    pub async fn acquire(
        &self,
        tenant_id: Uuid,
        class: FairnessClass,
    ) -> Result<OwnedSemaphorePermit, tokio::sync::AcquireError> {
        let Some(lane) = self.lane(class) else {
            // Unlimited — should not be called; return a dummy 1-permit semaphore.
            let sem = Arc::new(Semaphore::new(1));
            return sem.acquire_owned().await;
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
        let permit = lane.acquire(tenant_id).await?;
        wait_guard.arm_success();
        self.park_completions.fetch_add(1, Ordering::Relaxed);
        Ok(permit)
    }

    pub async fn active_count(&self, tenant_id: &Uuid, class: FairnessClass) -> usize {
        match self.lane(class) {
            Some(lane) => lane.active_count(tenant_id).await,
            None => 0,
        }
    }

    pub async fn total_active(&self) -> usize {
        let mut total = 0usize;
        if let Some(lane) = &self.ingest {
            total += lane.total_active().await;
        }
        if let Some(lane) = &self.lifecycle {
            total += lane.total_active().await;
        }
        total
    }

    pub async fn stats(&self) -> TenantLimiterStats {
        let tracked = {
            let mut n = 0usize;
            if let Some(lane) = &self.ingest {
                n = n.max(lane.tracked_tenants().await);
            }
            if let Some(lane) = &self.lifecycle {
                n = n.max(lane.tracked_tenants().await);
            }
            n
        };
        let park_waiters_ingest = self.park_waiters_ingest.load(Ordering::Relaxed);
        let park_waiters_lifecycle = self.park_waiters_lifecycle.load(Ordering::Relaxed);
        TenantLimiterStats {
            max_per_tenant: self.ingest.as_ref().map(|l| l.max).unwrap_or(0),
            max_lifecycle_per_tenant: self.lifecycle.as_ref().map(|l| l.max).unwrap_or(0),
            tracked_tenants: tracked,
            park_waiters: park_waiters_ingest + park_waiters_lifecycle,
            park_waiters_ingest,
            park_waiters_lifecycle,
            park_completions: self.park_completions.load(Ordering::Relaxed),
            park_aborts: self.park_aborts.load(Ordering::Relaxed),
        }
    }

    pub async fn cleanup_idle(&self) {
        if let Some(lane) = &self.ingest {
            lane.cleanup_idle().await;
        }
        if let Some(lane) = &self.lifecycle {
            lane.cleanup_idle().await;
        }
    }

    pub fn max_per_tenant(&self) -> usize {
        self.ingest.as_ref().map(|l| l.max).unwrap_or(0)
    }

    pub fn max_lifecycle_per_tenant(&self) -> usize {
        self.lifecycle.as_ref().map(|l| l.max).unwrap_or(0)
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
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant_a() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn tenant_b() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    #[tokio::test]
    async fn test_basic_acquire_release() {
        let limiter = TenantConcurrencyLimiter::new(2, 2);

        let permit1 = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert_eq!(
            limiter
                .active_count(&tenant_a(), FairnessClass::Ingest)
                .await,
            1
        );

        let permit2 = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await,
            TryAcquireOutcome::AtCapacity
        ));

        drop(permit1);
        let _permit3 = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired after release, got {other:?}"),
        };
        drop(permit2);
    }

    #[tokio::test]
    async fn lifecycle_lane_independent_of_ingest() {
        let limiter = TenantConcurrencyLimiter::new(1, 2);
        let _ingest = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter
                .try_acquire(tenant_a(), FairnessClass::Lifecycle)
                .await,
            TryAcquireOutcome::Acquired(_)
        ));
        assert!(matches!(
            limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await,
            TryAcquireOutcome::AtCapacity
        ));
    }

    #[tokio::test]
    async fn test_park_acquire_waits_for_release() {
        let limiter = TenantConcurrencyLimiter::new(1, 1);
        let held = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };

        let limiter2 = limiter.clone();
        let waiter = tokio::spawn(async move {
            limiter2
                .acquire(tenant_a(), FairnessClass::Ingest)
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

        let _permit_a = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        let _permit_b = match limiter.try_acquire(tenant_b(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        assert!(matches!(
            limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await,
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
        let limiter = TenantConcurrencyLimiter::new(2, 2);
        let permit = match limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await {
            TryAcquireOutcome::Acquired(p) => p,
            other => panic!("expected Acquired, got {other:?}"),
        };
        drop(permit);
        limiter.cleanup_idle().await;
        assert!(matches!(
            limiter.try_acquire(tenant_a(), FairnessClass::Ingest).await,
            TryAcquireOutcome::Acquired(_)
        ));
    }
}
