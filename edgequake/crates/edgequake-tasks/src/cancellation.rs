//! Per-task cooperative cancellation registry.
//!
//! ## WHY
//!
//! The previous cancellation mechanism was a single global boolean in
//! `PipelineState`. This had two critical flaws:
//! 1. **Global scope**: Cancelling one task's processing would set the flag
//!    for ALL tasks.
//! 2. **Never checked**: No pipeline stage actually read the flag during
//!    processing, so cancellation had zero effect on in-flight work.
//!
//! This module provides per-task `CancellationToken`s that:
//! - Are scoped to a single task (identified by `track_id`)
//! - Are cooperatively checked at every stage boundary in the pipeline
//! - Allow the cancel API to immediately signal a running task to stop
//! - Persist a cancel *intent* so pending / parked tasks are dropped even
//!   when no in-flight token is registered yet
//! - Are automatically cleaned up when a task completes
//!
//! ## Architecture
//!
//! ```text
//!  cancel_task API ──► CancellationRegistry::cancel("track-123")
//!                              │
//!                              ├─► cancel_intents.insert(track_id)
//!                              └─► CancellationToken::cancel() (if registered)
//!                              │
//!                     ┌────────┴────────────────────────┐
//!                     ▼                                  ▼
//!           worker dequeue guard              extraction loop checks
//!           drops pending/parked              token.is_cancelled()
//! ```
//!
//! ## Implements
//!
//! - **FEAT-CANCEL**: Per-task cooperative cancellation
//!
//! ## Enforces
//!
//! - **BR-CANCEL-01**: Cancellation must be per-task, not global
//! - **BR-CANCEL-02**: All pipeline stages must check for cancellation
//! - **BR-CANCEL-03**: Tokens must be cleaned up after task completion
//! - **BR-CANCEL-04**: Cancel is terminal — pending/requeued work must not restart

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Registry that maps task track_ids to their cancellation tokens.
///
/// Shared between the worker pool (which registers tokens when tasks start)
/// and the cancel API handler (which triggers cancellation by track_id).
#[derive(Clone)]
pub struct CancellationRegistry {
    tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
    /// Track IDs with a durable cancel intent (survives until cleared).
    ///
    /// WHY: Tokens are only registered while a worker is processing. Pending
    /// channel tasks and fairness-park waiters have no token; without an intent
    /// set, cancel would only update DB status while workers still start work.
    cancel_intents: Arc<RwLock<HashSet<String>>>,
    /// Monotonic count of cancel intents recorded (observability).
    cancel_intent_total: Arc<AtomicU64>,
}

impl Default for CancellationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            cancel_intents: Arc::new(RwLock::new(HashSet::new())),
            cancel_intent_total: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Register a new cancellation token for a task.
    ///
    /// If a cancel intent was already recorded for this track_id, the returned
    /// token is pre-cancelled so the worker observes it immediately.
    pub async fn register(&self, track_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        if self.has_cancel_intent(track_id).await {
            token.cancel();
        }
        let mut tokens = self.tokens.write().await;
        tokens.insert(track_id.to_string(), token.clone());
        token
    }

    /// Record cancel intent and signal any in-flight token.
    ///
    /// Returns `true` if an in-flight token was found and cancelled.
    pub async fn cancel(&self, track_id: &str) -> bool {
        self.mark_cancel_intent(track_id).await;
        let tokens = self.tokens.read().await;
        if let Some(token) = tokens.get(track_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    /// Record that this track_id must not be processed (pending or running).
    pub async fn mark_cancel_intent(&self, track_id: &str) {
        let mut intents = self.cancel_intents.write().await;
        if intents.insert(track_id.to_string()) {
            self.cancel_intent_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// True when cancel API (or equivalent) has requested stop for this id.
    pub async fn has_cancel_intent(&self, track_id: &str) -> bool {
        self.cancel_intents.read().await.contains(track_id)
    }

    /// Remove a task's token and cancel intent (cleanup after completion).
    pub async fn deregister(&self, track_id: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(track_id);
        drop(tokens);
        let mut intents = self.cancel_intents.write().await;
        intents.remove(track_id);
    }

    /// Check if a specific task has been cancelled (in-flight token).
    pub async fn is_cancelled(&self, track_id: &str) -> bool {
        if self.has_cancel_intent(track_id).await {
            return true;
        }
        let tokens = self.tokens.read().await;
        tokens
            .get(track_id)
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// Get the number of active tokens (for monitoring).
    pub async fn active_count(&self) -> usize {
        self.tokens.read().await.len()
    }

    /// Number of outstanding cancel intents (pending drain + in-flight).
    pub async fn cancel_intent_count(&self) -> usize {
        self.cancel_intents.read().await.len()
    }

    /// Lifetime total of cancel intents recorded.
    pub fn cancel_intent_total(&self) -> u64 {
        self.cancel_intent_total.load(Ordering::Relaxed)
    }

    /// Cancel every currently registered in-flight task (pipeline-wide stop).
    ///
    /// Also records cancel intents for those track_ids. Pending tasks that were
    /// never registered are not covered — callers should mark those via storage
    /// scan when needed.
    pub async fn cancel_all_active(&self) -> Vec<String> {
        let ids: Vec<String> = {
            let tokens = self.tokens.read().await;
            tokens.keys().cloned().collect()
        };
        for id in &ids {
            self.cancel(id).await;
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_cancel() {
        let registry = CancellationRegistry::new();

        let token = registry.register("task-1").await;
        assert!(!token.is_cancelled());

        let cancelled = registry.cancel("task-1").await;
        assert!(cancelled);
        assert!(token.is_cancelled());
        assert!(registry.has_cancel_intent("task-1").await);
    }

    #[tokio::test]
    async fn test_cancel_nonexistent_records_intent() {
        let registry = CancellationRegistry::new();

        let cancelled = registry.cancel("does-not-exist").await;
        assert!(!cancelled);
        assert!(registry.has_cancel_intent("does-not-exist").await);
        assert_eq!(registry.cancel_intent_total(), 1);
    }

    #[tokio::test]
    async fn test_register_after_intent_is_pre_cancelled() {
        let registry = CancellationRegistry::new();
        registry.mark_cancel_intent("task-1").await;
        let token = registry.register("task-1").await;
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_deregister_removes_token_and_intent() {
        let registry = CancellationRegistry::new();

        let _token = registry.register("task-1").await;
        registry.cancel("task-1").await;
        assert_eq!(registry.active_count().await, 1);
        assert_eq!(registry.cancel_intent_count().await, 1);

        registry.deregister("task-1").await;
        assert_eq!(registry.active_count().await, 0);
        assert_eq!(registry.cancel_intent_count().await, 0);
        assert!(!registry.has_cancel_intent("task-1").await);
    }

    #[tokio::test]
    async fn test_is_cancelled() {
        let registry = CancellationRegistry::new();

        let _token = registry.register("task-1").await;
        assert!(!registry.is_cancelled("task-1").await);

        registry.cancel("task-1").await;
        assert!(registry.is_cancelled("task-1").await);
    }

    #[tokio::test]
    async fn test_multiple_tasks_independent() {
        let registry = CancellationRegistry::new();

        let token1 = registry.register("task-1").await;
        let token2 = registry.register("task-2").await;

        registry.cancel("task-1").await;

        assert!(token1.is_cancelled());
        assert!(!token2.is_cancelled());
        assert!(!registry.has_cancel_intent("task-2").await);
    }

    #[tokio::test]
    async fn test_default_impl() {
        let registry = CancellationRegistry::default();
        assert_eq!(registry.active_count().await, 0);
    }
}
