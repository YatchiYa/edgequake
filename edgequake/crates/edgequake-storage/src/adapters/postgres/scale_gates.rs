//! SPEC-091 Wave-5 — measured scaling gates (partition / quantization).

use serde::{Deserialize, Serialize};

/// Evidence required before physical layout changes (LD-10).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScaleGateEvidence {
    pub threshold_breach_reproduced: bool,
    pub baseline_metric_ref: String,
    pub current_metric_ref: String,
    pub recall_at_10_baseline: Option<f64>,
    pub recall_at_10_current: Option<f64>,
    pub notes: Option<String>,
}

impl ScaleGateEvidence {
    pub fn is_complete(&self) -> bool {
        self.threshold_breach_reproduced
            && !self.baseline_metric_ref.is_empty()
            && !self.current_metric_ref.is_empty()
    }
}

/// Returns true only when evidence documents a reproduced threshold breach.
pub fn partition_allowed(evidence: &ScaleGateEvidence) -> bool {
    evidence.is_complete()
}

/// Returns true only when evidence documents a reproduced threshold breach.
pub fn quantization_allowed(evidence: &ScaleGateEvidence) -> bool {
    evidence.is_complete()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_scale_gates_require_complete_evidence() {
        let incomplete = ScaleGateEvidence {
            threshold_breach_reproduced: false,
            baseline_metric_ref: "w0_ann_p95".into(),
            current_metric_ref: "w4_ann_p95".into(),
            recall_at_10_baseline: Some(0.95),
            recall_at_10_current: Some(0.94),
            notes: None,
        };
        assert!(!partition_allowed(&incomplete));
        assert!(!quantization_allowed(&incomplete));

        let complete = ScaleGateEvidence {
            threshold_breach_reproduced: true,
            ..incomplete
        };
        assert!(partition_allowed(&complete));
        assert!(quantization_allowed(&complete));
    }
}
