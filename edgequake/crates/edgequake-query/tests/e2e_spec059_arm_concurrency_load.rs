//! SPEC-059 Wave 5 — Mix arm semaphore saturates under concurrent acquire.

use edgequake_query::engine_impl::modes::arm_concurrency::{
    acquire_arm_permit_for_tests, available_arm_permits_for_tests,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn e2e_spec059_arm_concurrency_bounds_in_flight() {
    let limit = available_arm_permits_for_tests().max(1);
    let in_flight = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    let start = Instant::now();
    for _ in 0..(limit * 4) {
        let in_flight = Arc::clone(&in_flight);
        let peak = Arc::clone(&peak);
        handles.push(tokio::spawn(async move {
            let _permit = acquire_arm_permit_for_tests().await;
            let cur = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(cur, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(20)).await;
            in_flight.fetch_sub(1, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.await.expect("join");
    }
    let observed_peak = peak.load(Ordering::SeqCst);
    assert!(
        observed_peak <= limit,
        "arm concurrency peak {observed_peak} exceeded limit {limit}"
    );
    eprintln!(
        "OK SPEC-059 arm load: limit={limit} peak={observed_peak} elapsed={:?}",
        start.elapsed()
    );
}
