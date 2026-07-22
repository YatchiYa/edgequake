//! Process-wide admission control for local LLM / embedding calls.
//!
//! WHY: Per-pipeline extract clamps cannot stop cross-worker storms when
//! `WORKER_THREADS` × embed/merge fan-out all hit Ollama. A single semaphore
//! caps in-flight chat+embed against local providers process-wide.
//!
//! Env: `EDGEQUAKE_LOCAL_MAX_INFLIGHT` (default 2). Set to `0` to disable.

use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Env key for the process-wide local inference permit count.
pub const LOCAL_MAX_INFLIGHT_ENV: &str = "EDGEQUAKE_LOCAL_MAX_INFLIGHT";

/// Default in-flight local LLM+embed calls when env is unset.
pub const DEFAULT_LOCAL_MAX_INFLIGHT: usize = 2;

static GATE: LazyLock<LocalInferenceGate> = LazyLock::new(LocalInferenceGate::from_env);

/// Shared admission gate for Ollama / LM Studio traffic.
pub struct LocalInferenceGate {
    /// `None` when disabled (`EDGEQUAKE_LOCAL_MAX_INFLIGHT=0`).
    semaphore: Option<Arc<Semaphore>>,
    max_inflight: usize,
    /// Last time we logged saturation (rate-limit warn spam).
    last_saturated_log: Mutex<Option<Instant>>,
}

impl LocalInferenceGate {
    /// Build from env (used once for the process singleton).
    pub fn from_env() -> Self {
        let max =
            parse_local_max_inflight(&std::env::var(LOCAL_MAX_INFLIGHT_ENV).unwrap_or_default());
        Self::new(max)
    }

    /// Construct with an explicit permit count (`0` disables).
    pub fn new(max_inflight: usize) -> Self {
        let semaphore = if max_inflight == 0 {
            None
        } else {
            Some(Arc::new(Semaphore::new(max_inflight)))
        };
        Self {
            semaphore,
            max_inflight,
            last_saturated_log: Mutex::new(None),
        }
    }

    pub fn max_inflight(&self) -> usize {
        self.max_inflight
    }

    pub fn enabled(&self) -> bool {
        self.semaphore.is_some()
    }

    fn maybe_log_saturated(&self) {
        let mut guard = self
            .last_saturated_log
            .lock()
            .expect("local inference gate log mutex");
        let now = Instant::now();
        let should_log = match *guard {
            None => true,
            Some(prev) => now.duration_since(prev).as_secs() >= 5,
        };
        if should_log {
            *guard = Some(now);
            tracing::warn!(
                max_inflight = self.max_inflight,
                "Local inference gate saturated — waiting for an Ollama/LM Studio slot"
            );
        }
    }

    /// Acquire a permit when the provider is local; no-op otherwise.
    pub async fn acquire_for_provider(&self, provider_name: &str) -> Option<OwnedSemaphorePermit> {
        if !edgequake_pipeline::is_local_extraction_provider(provider_name) {
            return None;
        }
        let sem = self.semaphore.clone()?;

        if sem.available_permits() == 0 {
            self.maybe_log_saturated();
        }

        match sem.acquire_owned().await {
            Ok(permit) => Some(permit),
            Err(_) => {
                tracing::error!("Local inference gate closed unexpectedly");
                None
            }
        }
    }
}

/// Process-wide gate singleton.
pub fn global_local_inference_gate() -> &'static LocalInferenceGate {
    &GATE
}

/// Parse `EDGEQUAKE_LOCAL_MAX_INFLIGHT` (`0` disables; default 2; max 32).
pub fn parse_local_max_inflight(raw: &str) -> usize {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return DEFAULT_LOCAL_MAX_INFLIGHT;
    }
    match trimmed.parse::<usize>() {
        Ok(0) => 0,
        Ok(n) => n.min(32),
        Err(_) => DEFAULT_LOCAL_MAX_INFLIGHT,
    }
}

/// Acquire a process-wide permit for a local provider call (drop to release).
pub async fn acquire_local_inference_permit(provider_name: &str) -> Option<OwnedSemaphorePermit> {
    global_local_inference_gate()
        .acquire_for_provider(provider_name)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_and_disable() {
        assert_eq!(parse_local_max_inflight(""), DEFAULT_LOCAL_MAX_INFLIGHT);
        assert_eq!(parse_local_max_inflight("0"), 0);
        assert_eq!(parse_local_max_inflight("2"), 2);
        assert_eq!(parse_local_max_inflight("99"), 32);
        assert_eq!(parse_local_max_inflight("nope"), DEFAULT_LOCAL_MAX_INFLIGHT);
    }

    #[tokio::test]
    async fn gate_limits_concurrent_local_acquires() {
        let gate = LocalInferenceGate::new(1);
        let p1 = gate.acquire_for_provider("ollama").await;
        assert!(p1.is_some());
        assert_eq!(gate.semaphore.as_ref().unwrap().available_permits(), 0);

        // Cloud providers bypass the gate.
        assert!(gate.acquire_for_provider("openai").await.is_none());

        drop(p1);
        let p2 = gate.acquire_for_provider("ollama").await;
        assert!(p2.is_some());
    }

    #[tokio::test]
    async fn gate_disabled_when_zero() {
        let gate = LocalInferenceGate::new(0);
        assert!(!gate.enabled());
        assert!(gate.acquire_for_provider("ollama").await.is_none());
    }
}
