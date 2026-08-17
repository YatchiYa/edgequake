//! Narrow task-storage capability ports (SPEC-120 P2 / WP3).
//!
//! P2 keeps [`TaskStorage`](crate::storage::TaskStorage) as the compatibility
//! contract. These capability traits let new APIs state a narrower dependency
//! without forcing existing backends or call sites to migrate.
//!
//! Worker pools should depend conceptually on [`TaskClaimer`] + [`LeaseKeeper`];
//! cancel facades on [`CancelStore`] + [`TaskRepository`]; fairness on
//! [`FairnessLedger`].

use crate::{
    error::TaskResult,
    fairness_hold::ClaimFairnessPolicy,
    lease::LeaseVerdict,
    storage::{SharedTaskStorage, TaskStatusSnapshot, TaskStorage},
    types::{FairnessClass, Task},
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Lightweight, scoped batch reads for list/status presentation.
#[async_trait]
pub trait TaskStatusReader: Send + Sync {
    async fn get_task_statuses(
        &self,
        tenant_id: Uuid,
        workspace_id: Uuid,
        track_ids: &[String],
    ) -> TaskResult<HashMap<String, TaskStatusSnapshot>>;
}

#[async_trait]
impl<T> TaskStatusReader for T
where
    T: TaskStorage + ?Sized,
{
    async fn get_task_statuses(
        &self,
        tenant_id: Uuid,
        workspace_id: Uuid,
        track_ids: &[String],
    ) -> TaskResult<HashMap<String, TaskStatusSnapshot>> {
        TaskStorage::get_task_statuses(self, tenant_id, workspace_id, track_ids).await
    }
}

/// Adapts a compatibility [`TaskStorage`] object to the narrow read capability.
pub struct TaskStatusReaderAdapter {
    storage: SharedTaskStorage,
}

impl TaskStatusReaderAdapter {
    pub fn new(storage: SharedTaskStorage) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl TaskStatusReader for TaskStatusReaderAdapter {
    async fn get_task_statuses(
        &self,
        tenant_id: Uuid,
        workspace_id: Uuid,
        track_ids: &[String],
    ) -> TaskResult<HashMap<String, TaskStatusSnapshot>> {
        self.storage
            .get_task_statuses(tenant_id, workspace_id, track_ids)
            .await
    }
}

pub type SharedTaskStatusReader = Arc<dyn TaskStatusReader>;

/// Durable task CRUD, lookup, and reporting capability.
#[async_trait]
pub trait TaskRepository: TaskStorage {}

#[async_trait]
impl<T> TaskRepository for T where T: TaskStorage + ?Sized {}

/// Atomic task claim, fairness hold, and release capability.
///
/// Workers conceptually depend on this port plus [`LeaseKeeper`] rather than
/// the full [`TaskStorage`] surface when refactored.
#[async_trait]
pub trait TaskClaimer: Send + Sync {
    async fn claim_next(&self, worker_id: &str, lease_ttl: Duration) -> TaskResult<Option<Task>>;

    async fn claim_next_with_policy(
        &self,
        worker_id: &str,
        lease_ttl: Duration,
        policy: ClaimFairnessPolicy,
    ) -> TaskResult<Option<Task>>;
}

#[async_trait]
impl<T> TaskClaimer for T
where
    T: TaskStorage + ?Sized,
{
    async fn claim_next(&self, worker_id: &str, lease_ttl: Duration) -> TaskResult<Option<Task>> {
        TaskStorage::claim_next(self, worker_id, lease_ttl).await
    }

    async fn claim_next_with_policy(
        &self,
        worker_id: &str,
        lease_ttl: Duration,
        policy: ClaimFairnessPolicy,
    ) -> TaskResult<Option<Task>> {
        TaskStorage::claim_next_with_policy(self, worker_id, lease_ttl, policy).await
    }
}

/// Lease refresh and cancellation-observation capability.
#[async_trait]
pub trait LeaseKeeper: Send + Sync {
    async fn refresh_lease(
        &self,
        track_id: &str,
        worker_id: &str,
        lease_token: Uuid,
        lease_ttl: Duration,
    ) -> TaskResult<LeaseVerdict>;

    async fn release_claim(
        &self,
        track_id: &str,
        worker_id: &str,
        lease_token: Uuid,
    ) -> TaskResult<bool>;
}

#[async_trait]
impl<T> LeaseKeeper for T
where
    T: TaskStorage + ?Sized,
{
    async fn refresh_lease(
        &self,
        track_id: &str,
        worker_id: &str,
        lease_token: Uuid,
        lease_ttl: Duration,
    ) -> TaskResult<LeaseVerdict> {
        TaskStorage::refresh_lease(self, track_id, worker_id, lease_token, lease_ttl).await
    }

    async fn release_claim(
        &self,
        track_id: &str,
        worker_id: &str,
        lease_token: Uuid,
    ) -> TaskResult<bool> {
        TaskStorage::release_claim(self, track_id, worker_id, lease_token).await
    }
}

/// Durable cancel intent (SPEC-120 P0).
#[async_trait]
pub trait CancelStore: Send + Sync {
    async fn request_cancel(&self, track_id: &str) -> TaskResult<Option<Task>>;
}

#[async_trait]
impl<T> CancelStore for T
where
    T: TaskStorage + ?Sized,
{
    async fn request_cancel(&self, track_id: &str) -> TaskResult<Option<Task>> {
        TaskStorage::request_cancel(self, track_id).await
    }
}

/// Fairness hold and weighted virtual-runtime ledger (SPEC-120 P2).
#[async_trait]
pub trait FairnessLedger: Send + Sync {
    async fn mark_fairness_hold(&self, track_id: &str, hold_ttl: Duration) -> TaskResult<()>;

    async fn clear_fairness_hold(&self, track_id: &str) -> TaskResult<()>;

    async fn charge_vruntime(
        &self,
        tenant_id: Uuid,
        fairness_class: FairnessClass,
        service_units: f64,
        weight: f64,
    ) -> TaskResult<()>;
}

#[async_trait]
impl<T> FairnessLedger for T
where
    T: TaskStorage + ?Sized,
{
    async fn mark_fairness_hold(&self, track_id: &str, hold_ttl: Duration) -> TaskResult<()> {
        TaskStorage::mark_fairness_hold(self, track_id, hold_ttl).await
    }

    async fn clear_fairness_hold(&self, track_id: &str) -> TaskResult<()> {
        TaskStorage::clear_fairness_hold(self, track_id).await
    }

    async fn charge_vruntime(
        &self,
        _tenant_id: Uuid,
        _fairness_class: FairnessClass,
        _service_units: f64,
        _weight: f64,
    ) -> TaskResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryTaskStorage;

    fn accepts_repository(_: &dyn TaskRepository) {}
    fn accepts_claimer(_: &dyn TaskClaimer) {}
    fn accepts_lease_keeper(_: &dyn LeaseKeeper) {}
    fn accepts_cancel_store(_: &dyn CancelStore) {}
    fn accepts_fairness_ledger(_: &dyn FairnessLedger) {}
    fn accepts_status_reader(_: &dyn TaskStatusReader) {}

    #[test]
    fn existing_storage_implements_all_narrow_ports() {
        let storage = MemoryTaskStorage::new();
        accepts_repository(&storage);
        accepts_claimer(&storage);
        accepts_lease_keeper(&storage);
        accepts_cancel_store(&storage);
        accepts_fairness_ledger(&storage);
        accepts_status_reader(&storage);
        let shared: SharedTaskStorage = Arc::new(storage);
        let reader = TaskStatusReaderAdapter::new(shared);
        accepts_status_reader(&reader);
    }
}
