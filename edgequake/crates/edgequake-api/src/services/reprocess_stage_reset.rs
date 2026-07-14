//! SPEC-048: reset document stage fields when reprocess is accepted (DEF-03).

use chrono::Utc;
use serde_json::{Map, Value};

use edgequake_tasks::ReprocessMode;

/// Apply stage reset fields onto a metadata object (mutates in place).
///
/// Sets `status=processing`, clears progress, and picks a start stage by mode.
pub fn apply_reprocess_stage_reset(obj: &mut Map<String, Value>, mode: ReprocessMode) {
    let (stage, message) = match mode {
        ReprocessMode::Full => ("queued", "Reprocess queued (full)"),
        ReprocessMode::EntitiesOnly => ("queued", "Reprocess queued (entities)"),
        ReprocessMode::MergeOnly => ("merging", "Reprocess queued (merge-only)"),
    };

    obj.insert(
        "status".to_string(),
        Value::String("processing".to_string()),
    );
    obj.insert(
        "current_stage".to_string(),
        Value::String(stage.to_string()),
    );
    obj.insert(
        "stage_message".to_string(),
        Value::String(message.to_string()),
    );
    obj.insert(
        "stage_progress".to_string(),
        Value::Number(serde_json::Number::from_f64(0.0).unwrap_or_else(|| 0.into())),
    );
    obj.insert(
        "reprocess_mode".to_string(),
        Value::String(mode.to_string()),
    );
    obj.insert(
        "updated_at".to_string(),
        Value::String(Utc::now().to_rfc3339()),
    );
    // Clear stale terminal notices
    obj.remove("error_message");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reset_clears_stale_extracting_message() {
        let mut v = json!({
            "status": "completed",
            "current_stage": "completed",
            "stage_message": "Done with 9000 entities",
            "stage_progress": 1.0,
            "error_message": "old"
        });
        let obj = v.as_object_mut().unwrap();
        apply_reprocess_stage_reset(obj, ReprocessMode::EntitiesOnly);
        assert_eq!(
            obj.get("status").and_then(|x| x.as_str()),
            Some("processing")
        );
        assert_eq!(
            obj.get("current_stage").and_then(|x| x.as_str()),
            Some("queued")
        );
        assert_eq!(
            obj.get("stage_progress").and_then(|x| x.as_f64()),
            Some(0.0)
        );
        assert!(obj.get("error_message").is_none());
        assert_eq!(
            obj.get("reprocess_mode").and_then(|x| x.as_str()),
            Some("entities")
        );
    }

    #[test]
    fn merge_mode_starts_at_merging() {
        let mut v = json!({});
        apply_reprocess_stage_reset(v.as_object_mut().unwrap(), ReprocessMode::MergeOnly);
        assert_eq!(
            v.get("current_stage").and_then(|x| x.as_str()),
            Some("merging")
        );
    }
}
