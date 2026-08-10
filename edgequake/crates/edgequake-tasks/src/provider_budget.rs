//! SPEC-091 QW1 — Cluster-global provider in-flight budget (LAW-Q3, LD-11).
//!
//! The scarce resource in ingestion is provider inference capacity (a local
//! Ollama server executes near-serially). This module makes that capacity a
//! **leased, cluster-visible budget** instead of a process-local semaphore:
//!
//! - [`ProviderBudget`] is the port (SOLID/DIP): acquire / refresh / release /
//!   reap, mirroring the task-claim lease discipline.
//! - [`PostgresProviderBudget`] enforces it cluster-wide via
//!   `edgequake.provider_slot` (migration 110) with `FOR UPDATE SKIP LOCKED`
//!   acquisition, TTL expiry, and a fencing token — N replicas cannot multiply
//!   provider load by N.
//! - [`MemoryProviderBudget`] is the in-memory adapter for tests and the
//!   conformance suite (LSP: same contract, both adapters).
//! - [`ProviderSlotGuard`] is the RAII wrapper: heartbeat while held,
//!   best-effort release on drop, TTL as the crash backstop (EC-22).
//!
//! Spec: `specs/091-simplify-data-layer/13-queue-admission-target-spec.md`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::{TaskError, TaskResult};

/// Env key for the cluster-wide provider in-flight budget (B).
pub const PROVIDER_BUDGET_ENV: &str = "EDGEQUAKE_PROVIDER_BUDGET";
/// Legacy env fallback (pre-QW1 process semaphore size).
pub const LOCAL_MAX_INFLIGHT_ENV: &str = "EDGEQUAKE_LOCAL_MAX_INFLIGHT";
/// Default budget for a local single-GPU provider when nothing is configured.
pub const DEFAULT_PROVIDER_BUDGET: u16 = 1;
/// Hard ceiling for any configured budget (migration CHECK agrees).
pub const MAX_PROVIDER_BUDGET: u16 = 64;

/// Resolve the provider budget (LAW-Q1: one resolver, one number).
///
/// Order: `EDGEQUAKE_PROVIDER_BUDGET` → `EDGEQUAKE_LOCAL_MAX_INFLIGHT`
/// (backward compat) → [`DEFAULT_PROVIDER_BUDGET`]. `0` disables the ledger
/// (cloud-only deployments pay no round trip).
pub fn provider_budget_from_env() -> u16 {
    for key in [PROVIDER_BUDGET_ENV, LOCAL_MAX_INFLIGHT_ENV] {
        let raw = std::env::var(key).unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(n) = trimmed.parse::<u16>() {
            return n.min(MAX_PROVIDER_BUDGET);
        }
    }
    DEFAULT_PROVIDER_BUDGET
}

/// A held provider slot (fencing token proves ownership).
#[derive(Debug, Clone)]
pub struct ProviderSlotLease {
    pub provider_key: String,
    pub slot_id: i32,
    pub lease_owner: String,
    pub lease_token: Uuid,
    pub lease_expires_at: DateTime<Utc>,
}

/// Port for the cluster-global provider budget (LAW-Q3).
#[async_trait]
pub trait ProviderBudget: Send + Sync {
    /// Try to lease one free (or stale) slot for `provider_key`.
    /// `Ok(None)` = saturated; callers park/retry, never churn.
    async fn try_acquire(
        &self,
        provider_key: &str,
        owner: &str,
        ttl: Duration,
    ) -> TaskResult<Option<ProviderSlotLease>>;

    /// Heartbeat: extend the lease. `Ok(false)` = CAS lost (slot re-leased or
    /// reaped) — the holder must stop heartbeating and finish promptly.
    async fn refresh(&self, lease: &ProviderSlotLease, ttl: Duration) -> TaskResult<bool>;

    /// Release the slot (CAS on owner+token; wrong token is a no-op error).
    async fn release(&self, lease: &ProviderSlotLease) -> TaskResult<()>;

    /// Reap all expired leases; returns the number freed (EC-22 backstop).
    async fn reap_expired(&self) -> TaskResult<u64>;
}

/// Shared alias for gate wiring.
pub type SharedProviderBudget = Arc<dyn ProviderBudget>;

// ---------------------------------------------------------------------------
// In-memory adapter (tests + conformance)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct MemorySlotRow {
    owner: String,
    token: Uuid,
    expires_at: DateTime<Utc>,
}

/// In-memory [`ProviderBudget`] — same contract as the Postgres adapter.
#[derive(Debug, Default)]
pub struct MemoryProviderBudget {
    /// provider_key → slot_id → row (absent = free).
    slots: Mutex<HashMap<(String, i32), MemorySlotRow>>,
    /// provider_key → budget.
    budgets: Mutex<HashMap<String, u16>>,
}

impl MemoryProviderBudget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a provider budget (test/conformance equivalent of migration seeding).
    pub fn with_budget(self, provider_key: &str, budget: u16) -> Self {
        self.budgets
            .lock()
            .expect("memory budget map")
            .insert(provider_key.to_string(), budget);
        self
    }

    fn budget_for(&self, provider_key: &str) -> u16 {
        self.budgets
            .lock()
            .expect("memory budget map")
            .get(provider_key)
            .copied()
            .unwrap_or(0)
    }
}

#[async_trait]
impl ProviderBudget for MemoryProviderBudget {
    async fn try_acquire(
        &self,
        provider_key: &str,
        owner: &str,
        ttl: Duration,
    ) -> TaskResult<Option<ProviderSlotLease>> {
        let budget = self.budget_for(provider_key);
        if budget == 0 {
            return Ok(None);
        }
        let now = Utc::now();
        let expires_at = crate::lease_expires_at(now, ttl);
        let mut slots = self.slots.lock().expect("memory provider slots");
        for slot_id in 0..i32::from(budget) {
            let key = (provider_key.to_string(), slot_id);
            let free = match slots.get(&key) {
                None => true,
                Some(row) => row.expires_at <= now,
            };
            if free {
                let row = MemorySlotRow {
                    owner: owner.to_string(),
                    token: Uuid::new_v4(),
                    expires_at,
                };
                slots.insert(key, row.clone());
                return Ok(Some(ProviderSlotLease {
                    provider_key: provider_key.to_string(),
                    slot_id,
                    lease_owner: row.owner,
                    lease_token: row.token,
                    lease_expires_at: row.expires_at,
                }));
            }
        }
        Ok(None)
    }

    async fn refresh(&self, lease: &ProviderSlotLease, ttl: Duration) -> TaskResult<bool> {
        let mut slots = self.slots.lock().expect("memory provider slots");
        let key = (lease.provider_key.clone(), lease.slot_id);
        let matches = matches!(
            slots.get(&key),
            Some(row) if row.owner == lease.lease_owner && row.token == lease.lease_token
        );
        if !matches {
            return Ok(false);
        }
        slots.insert(
            key,
            MemorySlotRow {
                owner: lease.lease_owner.clone(),
                token: lease.lease_token,
                expires_at: crate::lease_expires_at(Utc::now(), ttl),
            },
        );
        Ok(true)
    }

    async fn release(&self, lease: &ProviderSlotLease) -> TaskResult<()> {
        let mut slots = self.slots.lock().expect("memory provider slots");
        let key = (lease.provider_key.clone(), lease.slot_id);
        match slots.get(&key) {
            Some(row) if row.owner == lease.lease_owner && row.token == lease.lease_token => {
                slots.remove(&key);
                Ok(())
            }
            _ => Err(TaskError::StorageError(
                "provider slot release CAS failed: wrong owner or token".to_string(),
            )),
        }
    }

    async fn reap_expired(&self) -> TaskResult<u64> {
        let now = Utc::now();
        let mut slots = self.slots.lock().expect("memory provider slots");
        let before = slots.len();
        slots.retain(|_, row| row.expires_at > now);
        Ok((before - slots.len()) as u64)
    }
}

// ---------------------------------------------------------------------------
// RAII guard: heartbeat while held, release on drop
// ---------------------------------------------------------------------------

/// Process-local inflight count per provider (feeds Prometheus gauge on start/drop).
static SLOT_INFLIGHT: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, u64>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn bump_slot_inflight(provider: &str, delta: i64) -> u64 {
    let mut map = SLOT_INFLIGHT.lock().unwrap_or_else(|e| e.into_inner());
    let entry = map.entry(provider.to_string()).or_insert(0);
    if delta >= 0 {
        *entry = entry.saturating_add(delta as u64);
    } else {
        *entry = entry.saturating_sub((-delta) as u64);
    }
    *entry
}

/// RAII wrapper for a held [`ProviderSlotLease`].
///
/// - Heartbeats every `ttl/2` (min 5 s) so long calls never lose the slot.
/// - On CAS loss: warns and stops heartbeats (the bounded call finishes; the
///   slot is re-leased only after TTL — never released out from under a call).
/// - On drop: best-effort async release; TTL is the crash backstop (EC-22).
/// - SPEC-091 WP0: records inflight gauge + hold-duration histogram on start/drop.
pub struct ProviderSlotGuard {
    budget: SharedProviderBudget,
    lease: Option<ProviderSlotLease>,
    heartbeat: Option<tokio::task::JoinHandle<()>>,
    held_since: std::time::Instant,
}

impl ProviderSlotGuard {
    /// Wrap a lease and start its heartbeat.
    pub fn start(budget: SharedProviderBudget, lease: ProviderSlotLease, ttl: Duration) -> Self {
        let provider_key = lease.provider_key.clone();
        let inflight_now = bump_slot_inflight(&provider_key, 1);
        edgequake_observability::metrics::record_provider_slots_inflight(
            &provider_key,
            inflight_now,
        );

        let hb_budget = Arc::clone(&budget);
        let hb_lease = lease.clone();
        let interval = (ttl / 2).max(Duration::from_millis(50));
        let heartbeat = tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match hb_budget.refresh(&hb_lease, ttl).await {
                    Ok(true) => {}
                    Ok(false) => {
                        tracing::warn!(
                            provider = %hb_lease.provider_key,
                            slot = hb_lease.slot_id,
                            "Provider slot CAS lost — stopping heartbeat (call continues, TTL backstops)"
                        );
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(
                            provider = %hb_lease.provider_key,
                            slot = hb_lease.slot_id,
                            error = %e,
                            "Provider slot heartbeat failed — retrying"
                        );
                    }
                }
            }
        });
        Self {
            budget,
            lease: Some(lease),
            heartbeat: Some(heartbeat),
            held_since: std::time::Instant::now(),
        }
    }
}

impl std::fmt::Debug for ProviderSlotGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderSlotGuard")
            .field("lease", &self.lease)
            .finish()
    }
}

impl Drop for ProviderSlotGuard {
    fn drop(&mut self) {
        if let Some(h) = self.heartbeat.take() {
            h.abort();
        }
        if let Some(ref lease) = self.lease {
            let hold_secs = self.held_since.elapsed().as_secs_f64();
            edgequake_observability::metrics::record_provider_slot_hold_duration(
                &lease.provider_key,
                hold_secs,
            );
            let inflight_now = bump_slot_inflight(&lease.provider_key, -1);
            edgequake_observability::metrics::record_provider_slots_inflight(
                &lease.provider_key,
                inflight_now,
            );
            edgequake_observability::metrics::record_provider_slot_acquire(
                &lease.provider_key,
                "released",
            );
        }
        if let (Some(lease), Ok(handle)) =
            (self.lease.take(), tokio::runtime::Handle::try_current())
        {
            let budget = Arc::clone(&self.budget);
            handle.spawn(async move {
                if let Err(e) = budget.release(&lease).await {
                    tracing::debug!(
                        provider = %lease.provider_key,
                        slot = lease.slot_id,
                        error = %e,
                        "Provider slot release failed (TTL will reclaim)"
                    );
                }
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Postgres adapter (cluster-wide enforcement)
// ---------------------------------------------------------------------------

#[cfg(feature = "postgres")]
mod pg {
    use super::*;
    use sqlx::PgPool;

    /// Postgres-backed [`ProviderBudget`] over `edgequake.provider_slot`
    /// (migration 110). Acquisition is `FOR UPDATE SKIP LOCKED`; release and
    /// refresh are CAS on `(lease_owner, lease_token)`.
    pub struct PostgresProviderBudget {
        pool: PgPool,
    }

    impl PostgresProviderBudget {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }

        /// Seed/reconcile a provider budget (boot wiring): upsert the budget
        /// row, then align slot rows (`provider_budget_reconcile_slots`).
        pub async fn seed_budget(
            &self,
            provider_key: &str,
            budget: u16,
            source: &str,
        ) -> TaskResult<()> {
            let budget_i = i32::from(budget);
            sqlx::query(
                r#"
                INSERT INTO edgequake.provider_budget (provider_key, budget, source)
                VALUES ($1, $2, $3)
                ON CONFLICT (provider_key) DO UPDATE
                SET budget = EXCLUDED.budget, source = EXCLUDED.source, updated_at = now()
                "#,
            )
            .bind(provider_key)
            .bind(budget_i)
            .bind(source)
            .execute(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider_budget upsert failed: {e}")))?;

            sqlx::query("SELECT edgequake.provider_budget_reconcile_slots($1, $2)")
                .bind(provider_key)
                .bind(budget_i)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    TaskError::StorageError(format!("provider slot reconcile failed: {e}"))
                })?;
            Ok(())
        }

        /// Current (provider, inflight, budget) projection — observability.
        pub async fn inflight(&self, provider_key: &str) -> TaskResult<(i64, Option<i32>)> {
            let row: Option<(i64, Option<i32>)> = sqlx::query_as(
                "SELECT inflight, budget FROM edgequake.provider_inflight WHERE provider_key = $1",
            )
            .bind(provider_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider_inflight read failed: {e}")))?;
            Ok(row.unwrap_or((0, None)))
        }
    }

    #[async_trait]
    impl ProviderBudget for PostgresProviderBudget {
        async fn try_acquire(
            &self,
            provider_key: &str,
            owner: &str,
            ttl: Duration,
        ) -> TaskResult<Option<ProviderSlotLease>> {
            let token = Uuid::new_v4();
            let expires_at = crate::lease_expires_at(Utc::now(), ttl);
            let row: Option<(i32,)> = sqlx::query_as(
                r#"
                UPDATE edgequake.provider_slot s
                SET lease_owner = $2,
                    lease_token = $3,
                    lease_expires_at = $4,
                    acquired_at = NOW()
                FROM (
                    SELECT provider_key, slot_id
                    FROM edgequake.provider_slot
                    WHERE provider_key = $1
                      AND (lease_owner IS NULL OR lease_expires_at < NOW())
                    ORDER BY slot_id
                    FOR UPDATE SKIP LOCKED
                    LIMIT 1
                ) candidate
                WHERE s.provider_key = candidate.provider_key
                  AND s.slot_id = candidate.slot_id
                RETURNING s.slot_id
                "#,
            )
            .bind(provider_key)
            .bind(owner)
            .bind(token)
            .bind(expires_at)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider slot acquire failed: {e}")))?;

            Ok(row.map(|(slot_id,)| ProviderSlotLease {
                provider_key: provider_key.to_string(),
                slot_id,
                lease_owner: owner.to_string(),
                lease_token: token,
                lease_expires_at: expires_at,
            }))
        }

        async fn refresh(&self, lease: &ProviderSlotLease, ttl: Duration) -> TaskResult<bool> {
            let expires_at = crate::lease_expires_at(Utc::now(), ttl);
            let result = sqlx::query(
                r#"
                UPDATE edgequake.provider_slot
                SET lease_expires_at = $4
                WHERE provider_key = $1
                  AND slot_id = $2
                  AND lease_owner = $3
                  AND lease_token = $5
                "#,
            )
            .bind(&lease.provider_key)
            .bind(lease.slot_id)
            .bind(&lease.lease_owner)
            .bind(expires_at)
            .bind(lease.lease_token)
            .execute(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider slot refresh failed: {e}")))?;
            Ok(result.rows_affected() > 0)
        }

        async fn release(&self, lease: &ProviderSlotLease) -> TaskResult<()> {
            let result = sqlx::query(
                r#"
                UPDATE edgequake.provider_slot
                SET lease_owner = NULL,
                    lease_token = NULL,
                    lease_expires_at = NULL,
                    task_track_id = NULL,
                    workspace_id = NULL,
                    acquired_at = NULL
                WHERE provider_key = $1
                  AND slot_id = $2
                  AND lease_owner = $3
                  AND lease_token = $4
                "#,
            )
            .bind(&lease.provider_key)
            .bind(lease.slot_id)
            .bind(&lease.lease_owner)
            .bind(lease.lease_token)
            .execute(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider slot release failed: {e}")))?;
            if result.rows_affected() == 0 {
                return Err(TaskError::StorageError(
                    "provider slot release CAS failed: wrong owner or token".to_string(),
                ));
            }
            Ok(())
        }

        async fn reap_expired(&self) -> TaskResult<u64> {
            let result = sqlx::query(
                r#"
                UPDATE edgequake.provider_slot
                SET lease_owner = NULL,
                    lease_token = NULL,
                    lease_expires_at = NULL,
                    task_track_id = NULL,
                    workspace_id = NULL,
                    acquired_at = NULL
                WHERE lease_owner IS NOT NULL
                  AND lease_expires_at < NOW()
                "#,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| TaskError::StorageError(format!("provider slot reap failed: {e}")))?;
            Ok(result.rows_affected())
        }
    }
}

#[cfg(feature = "postgres")]
pub use pg::PostgresProviderBudget;

#[cfg(test)]
mod tests {
    use super::*;

    fn budget2() -> Arc<MemoryProviderBudget> {
        Arc::new(MemoryProviderBudget::new().with_budget("ollama", 2))
    }

    /// SPEC-091 QW1 / F-091-18, LAW-Q3: acquire up to budget, then saturated;
    /// release frees exactly one slot.
    #[tokio::test]
    async fn contract_spec091_provider_budget_acquire_release() {
        let budget = budget2();
        let a = budget
            .try_acquire("ollama", "w1", Duration::from_secs(60))
            .await
            .unwrap()
            .expect("slot 1");
        let b = budget
            .try_acquire("ollama", "w2", Duration::from_secs(60))
            .await
            .unwrap()
            .expect("slot 2");
        assert!(budget
            .try_acquire("ollama", "w3", Duration::from_secs(60))
            .await
            .unwrap()
            .is_none());

        // Wrong-token release is rejected (CAS / fencing).
        let mut forged = a.clone();
        forged.lease_token = Uuid::new_v4();
        assert!(budget.release(&forged).await.is_err());

        budget.release(&a).await.unwrap();
        let c = budget
            .try_acquire("ollama", "w3", Duration::from_secs(60))
            .await
            .unwrap()
            .expect("slot freed after release");
        assert_eq!(c.slot_id, a.slot_id);
        budget.release(&b).await.unwrap();
        budget.release(&c).await.unwrap();
    }

    /// Refresh extends a live lease; a stale slot is reclaimable (EC-22 seed).
    #[tokio::test]
    async fn contract_spec091_provider_budget_refresh_and_stale_reclaim() {
        let budget = Arc::new(MemoryProviderBudget::new().with_budget("ollama", 1));
        let ttl = Duration::from_millis(80);
        let lease = budget
            .try_acquire("ollama", "w1", ttl)
            .await
            .unwrap()
            .expect("slot");

        // Heartbeat keeps it alive past the original expiry.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(budget.refresh(&lease, ttl).await.unwrap());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(budget
            .try_acquire("ollama", "w2", Duration::from_secs(60))
            .await
            .unwrap()
            .is_none());

        // Without refresh it expires and is reclaimed by another owner.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let lease2 = budget
            .try_acquire("ollama", "w2", Duration::from_secs(60))
            .await
            .unwrap()
            .expect("stale slot reclaimed");
        // Old fencing token can no longer refresh or release.
        assert!(!budget.refresh(&lease, ttl).await.unwrap());
        assert!(budget.release(&lease).await.is_err());
        budget.release(&lease2).await.unwrap();
    }

    /// Reaper frees every expired lease.
    #[tokio::test]
    async fn contract_spec091_provider_budget_reap_expired() {
        let budget = budget2();
        let ttl = Duration::from_millis(30);
        budget.try_acquire("ollama", "w1", ttl).await.unwrap();
        budget.try_acquire("ollama", "w2", ttl).await.unwrap();
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(budget.reap_expired().await.unwrap(), 2);
        assert!(budget
            .try_acquire("ollama", "w3", Duration::from_secs(60))
            .await
            .unwrap()
            .is_some());
    }

    /// Concurrent claimants never exceed the budget (cluster-safety seed).
    #[tokio::test]
    async fn contract_spec091_provider_budget_concurrent_never_exceeds() {
        let budget = budget2();
        let mut handles = Vec::new();
        for i in 0..8 {
            let b = Arc::clone(&budget);
            handles.push(tokio::spawn(async move {
                b.try_acquire("ollama", &format!("w{i}"), Duration::from_secs(60))
                    .await
                    .unwrap()
            }));
        }
        let mut acquired = Vec::new();
        for h in handles {
            acquired.push(h.await.unwrap());
        }
        assert_eq!(acquired.iter().filter(|l| l.is_some()).count(), 2);
        assert_eq!(acquired.iter().filter(|l| l.is_none()).count(), 6);
    }

    /// RAII guard: heartbeat extends while held; drop releases.
    #[tokio::test]
    async fn contract_spec091_provider_slot_guard_raii() {
        let budget = Arc::new(MemoryProviderBudget::new().with_budget("ollama", 1));
        let ttl = Duration::from_millis(120);
        let lease = budget
            .try_acquire("ollama", "w1", ttl)
            .await
            .unwrap()
            .expect("slot");
        {
            let _guard =
                ProviderSlotGuard::start(Arc::clone(&budget) as SharedProviderBudget, lease, ttl);
            // Heartbeat (interval ttl/2 = 60ms) keeps the slot across > 1 TTL.
            tokio::time::sleep(Duration::from_millis(200)).await;
            assert!(budget
                .try_acquire("ollama", "w2", Duration::from_secs(60))
                .await
                .unwrap()
                .is_none());
        }
        // Drop releases — allow the spawned release to land.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(budget
            .try_acquire("ollama", "w2", Duration::from_secs(60))
            .await
            .unwrap()
            .is_some());
    }

    /// Env resolution: explicit budget wins, legacy fallback, default, clamps.
    #[test]
    fn provider_budget_env_resolution() {
        // Serialize env mutation across tests.
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var(PROVIDER_BUDGET_ENV);
        std::env::remove_var(LOCAL_MAX_INFLIGHT_ENV);
        assert_eq!(provider_budget_from_env(), DEFAULT_PROVIDER_BUDGET);
        std::env::set_var(LOCAL_MAX_INFLIGHT_ENV, "5");
        assert_eq!(provider_budget_from_env(), 5);
        std::env::set_var(PROVIDER_BUDGET_ENV, "7");
        assert_eq!(provider_budget_from_env(), 7);
        std::env::set_var(PROVIDER_BUDGET_ENV, "999");
        assert_eq!(provider_budget_from_env(), MAX_PROVIDER_BUDGET);
        std::env::set_var(PROVIDER_BUDGET_ENV, "0");
        assert_eq!(provider_budget_from_env(), 0);
        std::env::remove_var(PROVIDER_BUDGET_ENV);
        std::env::remove_var(LOCAL_MAX_INFLIGHT_ENV);
    }
}
