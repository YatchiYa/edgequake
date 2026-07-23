//! In-flight byte / token admission control (SPEC-083 / X-19).
//!
//! Tenant fairness (`tenant_limiter`) bounds **concurrent slots**. This module
//! bounds **in-flight estimated bytes** so a few huge PDFs cannot blow memory
//! or storm provider quotas even when slot capacity remains.
//!
//! ```text
//! claim_next → fairness try_acquire → admission try_admit(bytes)?
//!                                      YES → process (permit RAII)
//!                                      NO  → release claim, retry later
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::types::Task;

/// Default global in-flight budget when env is unset (512 MiB).
pub const DEFAULT_MAX_IN_FLIGHT_BYTES: u64 = 512 * 1024 * 1024;

/// Floor estimate when task payload has no size hint (keeps small jobs moving).
pub const DEFAULT_TASK_BYTE_COST: u64 = 4 * 1024 * 1024;

/// Env override for the global byte budget (`0` disables admission).
pub const ADMISSION_MAX_BYTES_ENV: &str = "EDGEQUAKE_ADMISSION_MAX_IN_FLIGHT_BYTES";

/// Outcome of a non-blocking admission attempt.
#[derive(Debug)]
pub enum AdmissionOutcome {
    /// Proceed; hold until processing completes (or drop to release).
    Admitted(AdmissionPermit),
    /// Budget exhausted — release claim and retry later.
    Rejected {
        requested: u64,
        in_flight: u64,
        max_bytes: u64,
    },
}

/// RAII permit that returns reserved bytes to the budget on drop.
#[derive(Debug)]
pub struct AdmissionPermit {
    budget: Arc<InFlightByteBudget>,
    bytes: u64,
}

impl AdmissionPermit {
    /// Reserved byte cost for this permit.
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        self.budget.release(self.bytes);
    }
}

/// Process-wide (or pool-scoped) in-flight byte budget.
#[derive(Debug)]
pub struct InFlightByteBudget {
    max_bytes: u64,
    in_flight: AtomicU64,
}

impl InFlightByteBudget {
    /// Create a budget. `max_bytes == 0` means admission is disabled (always admit 0).
    pub fn new(max_bytes: u64) -> Arc<Self> {
        Arc::new(Self {
            max_bytes,
            in_flight: AtomicU64::new(0),
        })
    }

    /// From `EDGEQUAKE_ADMISSION_MAX_IN_FLIGHT_BYTES` or [`DEFAULT_MAX_IN_FLIGHT_BYTES`].
    pub fn from_env() -> Arc<Self> {
        let max = std::env::var(ADMISSION_MAX_BYTES_ENV)
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_MAX_IN_FLIGHT_BYTES);
        Self::new(max)
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub fn in_flight_bytes(&self) -> u64 {
        self.in_flight.load(Ordering::Acquire)
    }

    /// Try to reserve `bytes`. Returns [`AdmissionOutcome::Rejected`] when over budget.
    pub fn try_admit(self: &Arc<Self>, bytes: u64) -> AdmissionOutcome {
        if self.max_bytes == 0 {
            return AdmissionOutcome::Admitted(AdmissionPermit {
                budget: Arc::clone(self),
                bytes: 0,
            });
        }
        let cost = bytes.max(1);
        loop {
            let current = self.in_flight.load(Ordering::Acquire);
            if current.saturating_add(cost) > self.max_bytes {
                return AdmissionOutcome::Rejected {
                    requested: cost,
                    in_flight: current,
                    max_bytes: self.max_bytes,
                };
            }
            match self.in_flight.compare_exchange_weak(
                current,
                current + cost,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return AdmissionOutcome::Admitted(AdmissionPermit {
                        budget: Arc::clone(self),
                        bytes: cost,
                    });
                }
                Err(_) => continue,
            }
        }
    }

    fn release(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.in_flight
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |v| {
                Some(v.saturating_sub(bytes))
            })
            .ok();
    }
}

/// Estimate in-flight byte cost from task payload (best-effort).
pub fn estimate_task_bytes(task: &Task) -> u64 {
    for key in [
        "file_size",
        "size_bytes",
        "byte_size",
        "content_length",
        "bytes",
    ] {
        if let Some(n) = task.task_data.get(key).and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_i64().map(|i| i.max(0) as u64))
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }) {
            return n.max(1);
        }
        if let Some(meta) = &task.metadata {
            if let Some(n) = meta.get(key).and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_i64().map(|i| i.max(0) as u64))
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }) {
                return n.max(1);
            }
        }
    }
    DEFAULT_TASK_BYTE_COST
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskType;
    use uuid::Uuid;

    #[test]
    fn contract_admission_rejects_over_budget() {
        let budget = InFlightByteBudget::new(10_000);
        let p1 = match budget.try_admit(8_000) {
            AdmissionOutcome::Admitted(p) => p,
            other => panic!("expected Admitted, got {other:?}"),
        };
        assert_eq!(budget.in_flight_bytes(), 8_000);

        match budget.try_admit(3_000) {
            AdmissionOutcome::Rejected {
                requested,
                in_flight,
                max_bytes,
            } => {
                assert_eq!(requested, 3_000);
                assert_eq!(in_flight, 8_000);
                assert_eq!(max_bytes, 10_000);
            }
            other => panic!("expected Rejected, got {other:?}"),
        }

        drop(p1);
        assert_eq!(budget.in_flight_bytes(), 0);
        let _p2 = match budget.try_admit(3_000) {
            AdmissionOutcome::Admitted(p) => p,
            other => panic!("expected Admitted after release, got {other:?}"),
        };
    }

    #[test]
    fn estimate_reads_file_size_from_task_data() {
        let mut task = Task::new(
            Uuid::nil(),
            Uuid::nil(),
            TaskType::Upload,
            serde_json::json!({ "file_size": 12345 }),
        );
        assert_eq!(estimate_task_bytes(&task), 12345);
        task.task_data = serde_json::json!({});
        task.metadata = Some(serde_json::json!({ "size_bytes": 99 }));
        assert_eq!(estimate_task_bytes(&task), 99);
    }
}
