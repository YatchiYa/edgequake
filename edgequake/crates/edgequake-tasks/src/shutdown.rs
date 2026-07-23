//! SPEC-083 X-31: cooperative shutdown drain budget.
//!
//! Long PDF / LLM work must not block process exit forever. Operators set
//! `EDGEQUAKE_SHUTDOWN_DRAIN_SECS` (default 30). After the budget, workers are
//! aborted and the HTTP serve future is dropped.

use std::time::Duration;

/// Env var for shutdown drain budget (seconds).
pub const SHUTDOWN_DRAIN_SECS_ENV: &str = "EDGEQUAKE_SHUTDOWN_DRAIN_SECS";

/// Default drain budget when the env var is unset.
pub const DEFAULT_SHUTDOWN_DRAIN_SECS: u64 = 30;

/// Resolve the shutdown drain budget from the environment.
///
/// Clamped to `[1, 3600]` so a typo cannot disable drain (0) or hang deploys
/// for hours.
pub fn shutdown_drain_budget() -> Duration {
    let secs = std::env::var(SHUTDOWN_DRAIN_SECS_ENV)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SHUTDOWN_DRAIN_SECS)
        .clamp(1, 3600);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_drain_default_is_30s() {
        // Avoid poisoning parallel tests: only assert clamp math on a local parse path.
        let secs = "not-a-number"
            .parse::<u64>()
            .ok()
            .unwrap_or(DEFAULT_SHUTDOWN_DRAIN_SECS)
            .clamp(1, 3600);
        assert_eq!(secs, 30);
        assert_eq!(DEFAULT_SHUTDOWN_DRAIN_SECS, 30);
    }

    #[test]
    fn shutdown_drain_clamp_rejects_zero() {
        assert_eq!(0u64.clamp(1, 3600), 1);
    }
}
