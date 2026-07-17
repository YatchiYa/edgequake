//! Thin ingestion status helpers (SPEC-057 P0).
//!
//! DRY cancel/status projection for HTTP cancel and PDF cancel writers.
//! Full stage_bridge collapse is deferred (P0.3b).

use chrono::Utc;
use edgequake_storage::PdfProcessingStatus;
use serde_json::{json, Map, Value};

use crate::services::classify_ingestion_failure;

/// PDF terminal status for user/system cancel (never Failed).
pub fn pdf_status_for_cancel() -> PdfProcessingStatus {
    PdfProcessingStatus::Cancelled
}

/// Apply cancelled document KV fields including `failure_class` (SPEC-045 / SPEC-057).
pub fn apply_doc_cancelled_fields(metadata: &mut Map<String, Value>, message: &str) {
    let failure = classify_ingestion_failure(message);
    let workspace = metadata
        .get("workspace_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    edgequake_observability::record_ingestion_failure(failure.as_str(), workspace);

    metadata.insert("status".to_string(), json!("cancelled"));
    metadata.insert("current_stage".to_string(), json!("cancelled"));
    metadata.insert("stage_message".to_string(), json!(message));
    metadata.insert("error_message".to_string(), json!(message));
    metadata.insert("failure_class".to_string(), json!(failure.as_str()));
    metadata.insert(
        "recommended_action".to_string(),
        json!(failure.recommended_action()),
    );
    metadata.insert("updated_at".to_string(), json!(Utc::now().to_rfc3339()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    #[test]
    fn apply_doc_cancelled_sets_failure_class() {
        let mut meta = Map::new();
        meta.insert("workspace_id".to_string(), json!("ws-1"));
        apply_doc_cancelled_fields(&mut meta, "Task cancelled by user");
        assert_eq!(meta.get("status").and_then(|v| v.as_str()), Some("cancelled"));
        assert_eq!(
            meta.get("failure_class").and_then(|v| v.as_str()),
            Some("cancelled")
        );
        assert_eq!(
            meta.get("recommended_action").and_then(|v| v.as_str()),
            Some("none")
        );
    }

    #[test]
    fn pdf_cancel_status_is_cancelled() {
        assert_eq!(pdf_status_for_cancel(), PdfProcessingStatus::Cancelled);
    }
}
