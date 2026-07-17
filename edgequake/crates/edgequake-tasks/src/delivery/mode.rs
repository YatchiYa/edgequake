//! Task delivery mode from environment (SPEC-026 P-12 / SPEC-057 P3).

/// Env: intended API replica count for multi-replica delivery gate.
pub const REPLICAS_ENV: &str = "EDGEQUAKE_REPLICAS";

/// How tasks are delivered to workers after Postgres persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskDeliveryMode {
    /// In-process channel only (default monolith).
    #[default]
    Local,
    /// Notify external bridge AND local channel (migration / hybrid).
    Bridged,
    /// Notify only — workers hydrate from Postgres SSOT via claim.
    NotifyOnly,
}

impl TaskDeliveryMode {
    /// True when wake path is intended for multi-process deployments.
    pub fn is_multi_replica_wake(self) -> bool {
        matches!(self, Self::Bridged | Self::NotifyOnly)
    }
}

/// Parse `EDGEQUAKE_TASK_DELIVERY` env var.
pub fn parse_delivery_mode(raw: &str) -> TaskDeliveryMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "bridged" | "bridge" | "dual" => TaskDeliveryMode::Bridged,
        "notify_only" | "notify-only" | "external" => TaskDeliveryMode::NotifyOnly,
        _ => TaskDeliveryMode::Local,
    }
}

/// Read delivery mode from environment.
pub fn delivery_mode_from_env() -> TaskDeliveryMode {
    std::env::var("EDGEQUAKE_TASK_DELIVERY")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(|v| parse_delivery_mode(&v))
        .unwrap_or_default()
}

/// Read intended replica count (`EDGEQUAKE_REPLICAS`, default 1, clamp ≥ 1).
pub fn replicas_from_env() -> usize {
    std::env::var(REPLICAS_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(1)
        .max(1)
}

/// SPEC-057 P3: Local delivery is forbidden when replicas > 1.
///
/// Correctness uses claim/lease; Bridged/NotifyOnly are required wake modes
/// so operators cannot accidentally run multi-replica on Local-only channel.
pub fn validate_delivery_for_replicas(
    mode: TaskDeliveryMode,
    replicas: usize,
) -> Result<(), String> {
    if replicas > 1 && mode == TaskDeliveryMode::Local {
        return Err(format!(
            "EDGEQUAKE_REPLICAS={replicas} requires multi-replica task delivery \
             (set EDGEQUAKE_TASK_DELIVERY=bridged or notify_only). \
             Local channel wake is single-process only; claim_next alone is not \
             enough without an explicit multi-replica wake mode."
        ));
    }
    Ok(())
}

/// True when orphan recovery / boot must treat leases as shared across processes.
pub fn is_multi_replica_deployment(mode: TaskDeliveryMode, replicas: usize) -> bool {
    replicas > 1 || mode.is_multi_replica_wake()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_mode_from_env_parses() {
        assert_eq!(parse_delivery_mode("local"), TaskDeliveryMode::Local);
        assert_eq!(parse_delivery_mode("bridged"), TaskDeliveryMode::Bridged);
        assert_eq!(
            parse_delivery_mode("notify_only"),
            TaskDeliveryMode::NotifyOnly
        );
    }

    #[test]
    fn local_with_two_replicas_fails_validation() {
        let err = validate_delivery_for_replicas(TaskDeliveryMode::Local, 2).unwrap_err();
        assert!(err.contains("EDGEQUAKE_TASK_DELIVERY"));
        assert!(validate_delivery_for_replicas(TaskDeliveryMode::Bridged, 2).is_ok());
        assert!(validate_delivery_for_replicas(TaskDeliveryMode::NotifyOnly, 2).is_ok());
        assert!(validate_delivery_for_replicas(TaskDeliveryMode::Local, 1).is_ok());
    }

    #[test]
    fn multi_replica_flag_from_replicas_or_mode() {
        assert!(!is_multi_replica_deployment(TaskDeliveryMode::Local, 1));
        assert!(is_multi_replica_deployment(TaskDeliveryMode::Local, 2));
        assert!(is_multi_replica_deployment(TaskDeliveryMode::Bridged, 1));
    }
}
