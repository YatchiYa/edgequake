//! SPEC-091 RM0 — shared drain mode / backoff helpers (DRY for compensation + outbox).

use std::time::Duration;

/// Three-way drain mode used by compensation and outbox workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainMode {
    Off,
    DryRun,
    On,
}

/// Parse `off` | `dry-run` | `on` from an env var value (empty → `default_mode`).
pub fn parse_drain_mode(raw: &str, default_mode: DrainMode) -> DrainMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" => default_mode,
        "dry-run" | "dryrun" | "dry" => DrainMode::DryRun,
        "on" | "1" | "true" | "yes" => DrainMode::On,
        "off" | "0" | "false" | "no" => DrainMode::Off,
        _ => default_mode,
    }
}

/// Interval seconds from env, floored at `min_secs`.
pub fn parse_interval_secs(env_key: &str, default_secs: u64, min_secs: u64) -> Duration {
    let secs = std::env::var(env_key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default_secs);
    Duration::from_secs(secs.max(min_secs))
}

/// Backoff seconds: `attempt_count * step_secs` (compensation/outbox shared).
pub fn backoff_secs(attempt_count: i32, step_secs: i64) -> i64 {
    i64::from(attempt_count.max(1)) * step_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_drain_mode_defaults() {
        assert_eq!(parse_drain_mode("", DrainMode::On), DrainMode::On);
        assert_eq!(parse_drain_mode("off", DrainMode::On), DrainMode::Off);
        assert_eq!(parse_drain_mode("dry-run", DrainMode::Off), DrainMode::DryRun);
        assert_eq!(parse_drain_mode("ON", DrainMode::Off), DrainMode::On);
    }

    #[test]
    fn backoff_grows_with_attempts() {
        assert_eq!(backoff_secs(1, 300), 300);
        assert_eq!(backoff_secs(3, 300), 900);
    }
}
