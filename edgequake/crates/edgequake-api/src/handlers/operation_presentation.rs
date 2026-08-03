//! SPEC-120 Lens 8: operation badge / affordance SSOT for task responses.

use edgequake_tasks::CapacityLayer;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// UI presentation hints derived from durable task state (Lens 8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OperationPresentation {
    /// Human-readable badge label (`Queued`, `Stopping…`, …).
    pub badge: String,
    /// Semantic tone for styling (`neutral`, `warning`, `info`, `success`, `danger`).
    pub tone: String,
    /// Stop/cancel affordance (`cancel`, `stop`, `disabled`, `none`).
    pub stop_affordance: String,
    /// Progress rendering mode (`none`, `indeterminate`, `determinate`, `frozen`, `full`).
    pub progress_mode: String,
}

/// Default badge when capacity-waiting without a named layer stamp.
pub const GENERIC_CAPACITY_WAIT_BADGE: &str = "Waiting for capacity";

/// Format capacity-wait badge from optional SSOT layer / reason string.
pub fn capacity_wait_badge(
    layer: Option<&CapacityLayer>,
    reason: Option<&str>,
) -> String {
    if let Some(layer) = layer {
        return layer.wait_message();
    }
    if let Some(reason) = reason.map(str::trim).filter(|s| !s.is_empty()) {
        return reason.to_string();
    }
    GENERIC_CAPACITY_WAIT_BADGE.to_string()
}

/// Map durable `(status, cancel_requested_at)` → presentation (pure, no I/O).
///
/// For `held`, assumes an active fairness hold (capacity wait). Prefer
/// [`operation_presentation_with_hold`] when hold TTL is known.
pub fn operation_presentation(status: &str, cancel_requested_at: bool) -> OperationPresentation {
    operation_presentation_with_hold(status, cancel_requested_at, true)
}

/// Map durable task state → presentation, distinguishing active vs expired holds.
///
/// INV-Q10: capacity badge only when the fairness hold is still active.
pub fn operation_presentation_with_hold(
    status: &str,
    cancel_requested_at: bool,
    fairness_hold_active: bool,
) -> OperationPresentation {
    operation_presentation_with_capacity(status, cancel_requested_at, fairness_hold_active, None)
}

/// Same as [`operation_presentation_with_hold`], with named capacity layer / reason.
pub fn operation_presentation_with_capacity(
    status: &str,
    cancel_requested_at: bool,
    fairness_hold_active: bool,
    capacity_wait_reason: Option<&str>,
) -> OperationPresentation {
    let capacity_badge = capacity_wait_badge(None, capacity_wait_reason);
    match status.trim().to_ascii_lowercase().as_str() {
        "pending" if cancel_requested_at => OperationPresentation {
            badge: "Cancelling…".into(),
            tone: "warning".into(),
            stop_affordance: "disabled".into(),
            progress_mode: "none".into(),
        },
        // INV-Q10: pending + active fairness hold is capacity park (status may
        // briefly lag `held` before release_claim; still show capacity badge).
        "pending" if fairness_hold_active => OperationPresentation {
            badge: capacity_badge,
            tone: "neutral".into(),
            stop_affordance: "cancel".into(),
            progress_mode: "none".into(),
        },
        "pending" => OperationPresentation {
            badge: "Queued".into(),
            tone: "neutral".into(),
            stop_affordance: "cancel".into(),
            progress_mode: "none".into(),
        },
        "held" if fairness_hold_active => OperationPresentation {
            badge: capacity_badge,
            tone: "neutral".into(),
            stop_affordance: "cancel".into(),
            progress_mode: "none".into(),
        },
        "held" => OperationPresentation {
            badge: "Queued".into(),
            tone: "neutral".into(),
            stop_affordance: "cancel".into(),
            progress_mode: "none".into(),
        },
        "processing" if cancel_requested_at => OperationPresentation {
            badge: "Stopping…".into(),
            tone: "warning".into(),
            stop_affordance: "disabled".into(),
            progress_mode: "frozen".into(),
        },
        "processing" => OperationPresentation {
            badge: "Running".into(),
            tone: "info".into(),
            stop_affordance: "stop".into(),
            progress_mode: "determinate".into(),
        },
        "cancelling" => OperationPresentation {
            badge: "Stopping…".into(),
            tone: "warning".into(),
            stop_affordance: "disabled".into(),
            progress_mode: "frozen".into(),
        },
        "indexed" => OperationPresentation {
            badge: "Ready".into(),
            tone: "success".into(),
            stop_affordance: "none".into(),
            progress_mode: "full".into(),
        },
        "failed" => OperationPresentation {
            badge: "Failed, retrying".into(),
            tone: "warning".into(),
            stop_affordance: "cancel".into(),
            progress_mode: "none".into(),
        },
        "cancelled" => OperationPresentation {
            badge: "Cancelled".into(),
            tone: "neutral".into(),
            stop_affordance: "none".into(),
            progress_mode: "none".into(),
        },
        "dead_letter" => OperationPresentation {
            badge: "Needs attention".into(),
            tone: "danger".into(),
            stop_affordance: "none".into(),
            progress_mode: "none".into(),
        },
        other => OperationPresentation {
            badge: other.to_string(),
            tone: "neutral".into(),
            stop_affordance: "none".into(),
            progress_mode: "none".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_with_cancel_intent_is_stopping() {
        let p = operation_presentation("processing", true);
        assert_eq!(p.badge, "Stopping…");
        assert_eq!(p.tone, "warning");
        assert_eq!(p.stop_affordance, "disabled");
        assert_eq!(p.progress_mode, "frozen");
    }

    #[test]
    fn held_shows_capacity_wait() {
        let p = operation_presentation("held", false);
        assert_eq!(p.badge, GENERIC_CAPACITY_WAIT_BADGE);
        assert_eq!(p.stop_affordance, "cancel");
    }

    #[test]
    fn pending_with_active_hold_shows_capacity_wait() {
        let p = operation_presentation_with_hold("pending", false, true);
        assert_eq!(p.badge, GENERIC_CAPACITY_WAIT_BADGE);
    }

    #[test]
    fn held_without_active_hold_shows_queued() {
        let p = operation_presentation_with_hold("held", false, false);
        assert_eq!(p.badge, "Queued");
        assert_eq!(p.stop_affordance, "cancel");
    }

    #[test]
    fn held_with_named_reason_shows_tenant_fair_share() {
        let reason = CapacityLayer::TenantFairShare {
            in_use: 1,
            max: 1,
        }
        .wait_message();
        let p = operation_presentation_with_capacity("held", false, true, Some(&reason));
        assert!(p.badge.contains("tenant fair-share"));
        assert!(p.badge.contains("1 of 1"));
    }

    #[test]
    fn dead_letter_needs_attention() {
        let p = operation_presentation("dead_letter", false);
        assert_eq!(p.badge, "Needs attention");
        assert_eq!(p.tone, "danger");
    }
}
