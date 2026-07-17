//! Per-tenant concurrency limiter for fair task scheduling.
//!
//! ## WHY Per-Tenant Fair Scheduling?
//!
//! Without tenant isolation, one tenant uploading 50 PDFs monopolizes all
//! worker threads, forcing other tenants to wait until the entire batch
//! finishes. This violates multi-tenant fairness guarantees.
//!
//! ## Strategy: Semaphore + Park-Until-Permit
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────┐
//! │                    WORKER POOL (N workers)                │
//! │                                                           │
//! │  Worker picks task ──► try_acquire(tenant)?               │
//! │         │                   │                             │
//! │         │                   YES → run with permit         │
//! │         │                   NO  → park on acquire()       │
//! │         │                         (no channel churn)      │
//! │         │                         worker continues        │
//! └───────────────────────────────────────────────────────────┘
//! ```
//!
//! Workers use `try_acquire()` first so they can immediately serve other
//! tenants. When a tenant is at capacity, the task parks on `acquire()` in
//! a background waiter instead of bouncing through the queue every 500ms.
//!
//! ## Implements
//!
//! - **FEAT-TENANT-FAIRNESS**: At least 1 worker slot per tenant
//! - **BR-TENANT-ISOLATION**: One tenant cannot block other tenants

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tracing::debug;
use uuid::Uuid;

/// Snapshot of limiter observability counters.
#[derive(Debug, Clone, Copy, Default)]
pub struct TenantLimiterStats {
    pub max_per_tenant: usize,
    pub tracked_tenants: usize,
    pub park_waiters: u64,
    pub park_completions: u64,
    pub park_aborts: u64,
}

/// RAII counter for park waiters. Decrements on drop; records abort unless armed.
struct ParkWaitGuard {
    waiters: Arc<AtomicU64>,
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
        if !self.success {
            self.aborts.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Per-tenant concurrency limiter using semaphores.
#[derive(Clone)]
pub struct TenantConcurrencyLimiter {
    max_per_tenant: usize,
    semaphores: Arc<RwLock<HashMap<Uuid, Arc<Semaphore>>>>,
    park_waiters: Arc<AtomicU64>,
    park_completions: Arc<AtomicU64>,
    park_aborts: Arc<AtomicU64>,
}

impl TenantConcurrencyLimiter {
    /// Create a new limiter.
    pub fn new(max_per_tenant: usize) -> Self {
        let max_per_tenant = max_per_tenant.max(1);
        Self {
            max_per_tenant,
            semaphores: Arc::new(RwLock::new(HashMap::new())),
            park_waiters: Arc::new(AtomicU64::new(0)),
            park_completions: Arc::new(AtomicU64::new(0)),
            park_aborts: Arc::new(AtomicU64::new(0)),
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
                max_concurrent = self.max_per_tenant,
                "Created tenant concurrency semaphore"
            );
            Arc::new(Semaphore::new(self.max_per_tenant))
        });
        Arc::clone(sem)
    }

    /// Try to acquire a processing slot for the given tenant (non-blocking).
    pub async fn try_acquire(&self, tenant_id: Uuid) -> Option<OwnedSemaphorePermit> {
        let semaphore = self.semaphore_for(tenant_id).await;
        semaphore.try_acquire_owned().ok()
    }

    /// Park until a processing slot is available for the tenant.
    ///
    /// WHY: Replacing try+500ms-requeue with park eliminates channel churn when
    /// a single tenant has a large backlog under a low concurrency cap.
    ///
    /// Cancel-safe for waiter accounting: dropping this future (e.g. via
    /// `select!`) decrements `park_waiters` via `ParkWaitGuard`.
    pub async fn acquire(
        &self,
        tenant_id: Uuid,
    ) -> Result<OwnedSemaphorePermit, tokio::sync::AcquireError> {
        self.park_waiters.fetch_add(1, Ordering::Relaxed);
        let mut wait_guard = ParkWaitGuard {
            waiters: Arc::clone(&self.park_waiters),
            aborts: Arc::clone(&self.park_aborts),
            success: false,
        };
        let semaphore = self.semaphore_for(tenant_id).await;
        let permit = semaphore.acquire_owned().await?;
        wait_guard.arm_success();
        self.park_completions.fetch_add(1, Ordering::Relaxed);
        Ok(permit)
    }

    /// Get current active task count for a tenant (for metrics/logging).
    pub async fn active_count(&self, tenant_id: &Uuid) -> usize {
        let read_guard = self.semaphores.read().await;
        if let Some(sem) = read_guard.get(tenant_id) {
            self.max_per_tenant - sem.available_permits()
        } else {
            0
        }
    }

    /// Aggregate active permits across all tracked tenants.
    pub async fn total_active(&self) -> usize {
        let read_guard = self.semaphores.read().await;
        read_guard
            .values()
            .map(|sem| self.max_per_tenant.saturating_sub(sem.available_permits()))
            .sum()
    }

    /// Observability snapshot.
    pub async fn stats(&self) -> TenantLimiterStats {
        TenantLimiterStats {
            max_per_tenant: self.max_per_tenant,
            tracked_tenants: self.semaphores.read().await.len(),
            park_waiters: self.park_waiters.load(Ordering::Relaxed),
            park_completions: self.park_completions.load(Ordering::Relaxed),
            park_aborts: self.park_aborts.load(Ordering::Relaxed),
        }
    }

    /// Clean up semaphores for tenants with no active tasks.
    pub async fn cleanup_idle(&self) {
        let mut write_guard = self.semaphores.write().await;
        let before = write_guard.len();
        write_guard.retain(|_tenant_id, sem| sem.available_permits() < self.max_per_tenant);
        let removed = before - write_guard.len();
        if removed > 0 {
            debug!(
                removed = removed,
                remaining = write_guard.len(),
                "Cleaned up idle tenant semaphores"
            );
        }
    }

    pub fn max_per_tenant(&self) -> usize {
        self.max_per_tenant
    }

    pub fn park_waiter_count(&self) -> u64 {
        self.park_waiters.load(Ordering::Relaxed)
    }
}

impl std::fmt::Debug for TenantConcurrencyLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TenantConcurrencyLimiter")
            .field("max_per_tenant", &self.max_per_tenant)
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
        let limiter = TenantConcurrencyLimiter::new(2);

        let permit1 = limiter.try_acquire(tenant_a()).await;
        assert!(permit1.is_some());
        assert_eq!(limiter.active_count(&tenant_a()).await, 1);

        let permit2 = limiter.try_acquire(tenant_a()).await;
        assert!(permit2.is_some());
        assert_eq!(limiter.active_count(&tenant_a()).await, 2);

        let permit3 = limiter.try_acquire(tenant_a()).await;
        assert!(permit3.is_none());

        drop(permit1);
        let permit4 = limiter.try_acquire(tenant_a()).await;
        assert!(permit4.is_some());
    }

    #[tokio::test]
    async fn test_park_acquire_waits_for_release() {
        let limiter = TenantConcurrencyLimiter::new(1);
        let held = limiter.try_acquire(tenant_a()).await.unwrap();

        let limiter2 = limiter.clone();
        let waiter = tokio::spawn(async move { limiter2.acquire(tenant_a()).await.unwrap() });

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
        let limiter = TenantConcurrencyLimiter::new(1);

        let _permit_a = limiter.try_acquire(tenant_a()).await;
        assert!(_permit_a.is_some());

        let permit_b = limiter.try_acquire(tenant_b()).await;
        assert!(permit_b.is_some());

        let permit_a2 = limiter.try_acquire(tenant_a()).await;
        assert!(permit_a2.is_none());
    }

    #[tokio::test]
    async fn test_min_one_permit() {
        let limiter = TenantConcurrencyLimiter::new(0);
        let permit = limiter.try_acquire(tenant_a()).await;
        assert!(permit.is_some());
    }

    #[tokio::test]
    async fn test_cleanup_idle() {
        let limiter = TenantConcurrencyLimiter::new(2);

        let permit = limiter.try_acquire(tenant_a()).await.unwrap();
        drop(permit);

        limiter.cleanup_idle().await;

        let permit = limiter.try_acquire(tenant_a()).await;
        assert!(permit.is_some());
    }

    #[tokio::test]
    async fn test_active_while_in_flight() {
        let limiter = TenantConcurrencyLimiter::new(3);

        let _p1 = limiter.try_acquire(tenant_a()).await.unwrap();
        let _p2 = limiter.try_acquire(tenant_a()).await.unwrap();

        assert_eq!(limiter.active_count(&tenant_a()).await, 2);

        limiter.cleanup_idle().await;
        assert_eq!(limiter.active_count(&tenant_a()).await, 2);
    }
}
