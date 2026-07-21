//! Structured Interrupted-after-restart classification (issue #304).
//!
//! Prefer `failure_code: server_restart_interrupted` over message matching.
//! Legacy message matching remains only as a migration fallback.

use serde_json::Value;

/// Stable failure code written by orphan recovery when AUTO_RESUME is off.
pub const FAILURE_CODE_SERVER_RESTART_INTERRUPTED: &str = "server_restart_interrupted";

/// True when document metadata represents an Interrupted-after-restart failure.
pub fn is_interrupted_restart_metadata(metadata: &Value) -> bool {
    if metadata
        .get("failure_code")
        .and_then(|v| v.as_str())
        .is_some_and(|c| c == FAILURE_CODE_SERVER_RESTART_INTERRUPTED)
    {
        return true;
    }
    // Legacy fallback (pre-structured code).
    let msg = metadata
        .get("error_message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let stage = metadata
        .get("stage_message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let blob = format!("{msg} {stage}").to_ascii_lowercase();
    blob.contains("interrupted by server restart")
        || (blob.contains("interrupted") && blob.contains("reprocess"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefers_structured_failure_code() {
        let meta = json!({
            "status": "failed",
            "failure_code": FAILURE_CODE_SERVER_RESTART_INTERRUPTED,
            "error_message": "something else",
        });
        assert!(is_interrupted_restart_metadata(&meta));
    }

    #[test]
    fn legacy_message_fallback() {
        let meta = json!({
            "status": "failed",
            "error_message": "Interrupted by server restart (was Processing for 3 minutes). Interrupted — use Reprocess",
        });
        assert!(is_interrupted_restart_metadata(&meta));
    }

    #[test]
    fn ordinary_failure_is_not_interrupted() {
        let meta = json!({
            "status": "failed",
            "error_message": "Entity extraction timed out",
        });
        assert!(!is_interrupted_restart_metadata(&meta));
    }
}
