//! Process-wide lock for tests that mutate process environment.
//!
//! Multiple modules previously used separate `Mutex`es, which raced on
//! `std::env` and flaked SPEC-105 cutover / vector_backend tests.

use std::sync::{Mutex, MutexGuard};

pub fn test_env_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}
