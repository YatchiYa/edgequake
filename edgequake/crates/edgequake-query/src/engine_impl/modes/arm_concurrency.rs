//! SPEC-058: process-wide Mix/Hybrid arm concurrency gate.
//!
//! Intent arm masks skip arms; this semaphore limits *in-flight* arms so
//! connection cost stays bounded under concurrent Mix queries.

use std::sync::{Arc, OnceLock};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

static ARM_SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// Default permits when env unset (enough for typical Mix 3-arm × a few queries).
const DEFAULT_ARM_CONCURRENCY: usize = 4;

fn arm_concurrency_from_env() -> usize {
    std::env::var("EDGEQUAKE_QUERY_ARM_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_ARM_CONCURRENCY)
        .clamp(1, 256)
}

fn arm_semaphore() -> Arc<Semaphore> {
    ARM_SEMAPHORE
        .get_or_init(|| Arc::new(Semaphore::new(arm_concurrency_from_env())))
        .clone()
}

/// Acquire one Mix/Hybrid arm permit (awaits when the pool of arms is saturated).
pub(super) async fn acquire_arm_permit() -> OwnedSemaphorePermit {
    arm_semaphore()
        .acquire_owned()
        .await
        .expect("query arm semaphore must not be closed")
}

/// SPEC-059: test harness for arm concurrency load (process-local semaphore).
#[doc(hidden)]
pub async fn acquire_arm_permit_for_tests() -> OwnedSemaphorePermit {
    acquire_arm_permit().await
}

/// SPEC-059: current available permits (observability for load tests).
#[doc(hidden)]
pub fn available_arm_permits_for_tests() -> usize {
    arm_semaphore().available_permits()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn default_arm_concurrency_is_positive() {
        assert!(arm_concurrency_from_env() >= 1);
    }

    #[tokio::test]
    async fn arm_semaphore_saturates_under_concurrent_acquire() {
        // Process-wide OnceLock — use whatever limit was initialized.
        let limit = available_arm_permits_for_tests().max(1);
        let mut held = Vec::new();
        for _ in 0..limit {
            held.push(acquire_arm_permit_for_tests().await);
        }
        assert_eq!(available_arm_permits_for_tests(), 0);

        let start = Instant::now();
        let waiter = tokio::spawn(async { acquire_arm_permit_for_tests().await });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !waiter.is_finished(),
            "extra acquire must block while saturated"
        );
        drop(held);
        let _permit = tokio::time::timeout(Duration::from_secs(2), waiter)
            .await
            .expect("timeout")
            .expect("join");
        assert!(
            start.elapsed() >= Duration::from_millis(40),
            "waiter should have been blocked until release"
        );
    }
}
