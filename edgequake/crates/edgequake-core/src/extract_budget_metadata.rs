//! SPEC-117 — Workspace extract budget metadata apply (shared by Postgres + in-memory).

use std::collections::HashMap;

use edgequake_pipeline::{ExtractionCaps, META_EXTRACT_MAX_ENTITIES, META_EXTRACT_MAX_RECORDS};

/// Apply SPEC-117 extract budget from API fields.
///
/// - `extract_budget_mode` = `inherit` / `none` / `""` → clear keys  
/// - `custom` (or omitted with both ints) → validate + store  
/// - all omitted → leave unchanged  
/// - one int without the other → error
pub fn apply_extract_budget_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    extract_budget_mode: Option<String>,
    extract_max_entities: Option<u32>,
    extract_max_records: Option<u32>,
) -> Result<(), String> {
    if let Some(raw) = extract_budget_mode {
        let mode = raw.trim().to_ascii_lowercase();
        if mode.is_empty() || mode == "inherit" || mode == "none" {
            clear_extract_budget_metadata(metadata);
            return Ok(());
        }
        if mode != "custom" {
            return Err(format!(
                "Unsupported extract_budget_mode '{raw}'. Allowed: inherit, custom"
            ));
        }
        // custom requires both ints (defaults LightRAG 40/100 if omitted)
        let ents = extract_max_entities.unwrap_or(40);
        let recs = extract_max_records.unwrap_or(100);
        return store_pair(metadata, ents, recs);
    }

    match (extract_max_entities, extract_max_records) {
        (None, None) => Ok(()),
        (Some(0), Some(0)) => {
            clear_extract_budget_metadata(metadata);
            Ok(())
        }
        (Some(_), None) | (None, Some(_)) => Err(
            "extract_max_entities and extract_max_records must both be set (or both omitted)"
                .into(),
        ),
        (Some(ents), Some(recs)) => store_pair(metadata, ents, recs),
    }
}

fn store_pair(
    metadata: &mut HashMap<String, serde_json::Value>,
    ents: u32,
    recs: u32,
) -> Result<(), String> {
    let caps = ExtractionCaps::validate(ents as usize, recs as usize)?;
    metadata.insert(
        META_EXTRACT_MAX_ENTITIES.into(),
        serde_json::json!(caps.max_entities),
    );
    metadata.insert(
        META_EXTRACT_MAX_RECORDS.into(),
        serde_json::json!(caps.max_total_records),
    );
    Ok(())
}

/// Remove extract budget keys (Inherit fleet).
pub fn clear_extract_budget_metadata(metadata: &mut HashMap<String, serde_json::Value>) {
    metadata.remove(META_EXTRACT_MAX_ENTITIES);
    metadata.remove(META_EXTRACT_MAX_RECORDS);
}

/// Resolve caps from workspace metadata (Inherit → None).
pub fn extract_caps_from_metadata(
    metadata: &HashMap<String, serde_json::Value>,
) -> Option<ExtractionCaps> {
    ExtractionCaps::from_metadata(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_clear() {
        let mut meta = HashMap::new();
        apply_extract_budget_metadata(&mut meta, Some("custom".into()), Some(40), Some(100))
            .unwrap();
        assert_eq!(
            meta.get(META_EXTRACT_MAX_ENTITIES).and_then(|v| v.as_u64()),
            Some(40)
        );
        apply_extract_budget_metadata(&mut meta, Some("inherit".into()), None, None).unwrap();
        assert!(!meta.contains_key(META_EXTRACT_MAX_ENTITIES));
    }

    #[test]
    fn rejects_entities_gt_records() {
        let mut meta = HashMap::new();
        assert!(apply_extract_budget_metadata(&mut meta, None, Some(50), Some(40)).is_err());
    }
}
