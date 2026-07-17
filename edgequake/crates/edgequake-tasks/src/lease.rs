//! Task lease TTL helpers (SPEC-057 P1).

use chrono::{DateTime, Utc};
use std::time::Duration;

/// Default lease TTL (seconds). Heartbeat refreshes every 60s.
pub const DEFAULT_TASK_LEASE_TTL_SECS: u64 = 120;

/// Minimum allowed lease TTL.
pub const MIN_TASK_LEASE_TTL_SECS: u64 = 30;

/// Resolve lease TTL from `EDGEQUAKE_TASK_LEASE_TTL_SECS` (default 120, min 30).
pub fn task_lease_ttl_from_env() -> Duration {
    let secs = std::env::var("EDGEQUAKE_TASK_LEASE_TTL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TASK_LEASE_TTL_SECS)
        .max(MIN_TASK_LEASE_TTL_SECS);
    Duration::from_secs(secs)
}

/// Compute lease expiry timestamp from `now` + `ttl`.
pub fn lease_expires_at(now: DateTime<Utc>, ttl: Duration) -> DateTime<Utc> {
    now + chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::seconds(
        DEFAULT_TASK_LEASE_TTL_SECS as i64,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ttl_is_at_least_min() {
        // Do not mutate env in parallel tests — just validate constants.
        assert!(DEFAULT_TASK_LEASE_TTL_SECS >= MIN_TASK_LEASE_TTL_SECS);
        assert_eq!(DEFAULT_TASK_LEASE_TTL_SECS, 120);
    }
}
