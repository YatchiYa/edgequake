use std::collections::HashMap;

use crate::type_list::{
    apply_kg_schema_preset_metadata as apply_kg_schema_preset_metadata_inner,
    apply_relation_edges_metadata as apply_relation_edges_metadata_inner,
    apply_type_list_metadata, apply_type_list_strict_metadata, normalize_type_list, RelationEdge,
};

// ============ Helper Functions ============

/// Apply `entity_types` list to workspace metadata (create/update).
pub(crate) fn apply_entity_types_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    entity_types: Option<Vec<String>>,
) {
    apply_type_list_metadata(metadata, "entity_types", entity_types);
}

/// Apply strict entity-type enforcement flag (default true when key absent).
pub(crate) fn apply_entity_types_strict_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    strict: Option<bool>,
) {
    apply_type_list_strict_metadata(metadata, "entity_types_strict", strict);
}

/// Apply `relation_types` list (SPEC-114).
pub(crate) fn apply_relation_types_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    relation_types: Option<Vec<String>>,
) {
    apply_type_list_metadata(metadata, "relation_types", relation_types);
}

/// Apply `relation_types_strict` sparse flag (SPEC-114).
pub(crate) fn apply_relation_types_strict_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    strict: Option<bool>,
) {
    apply_type_list_strict_metadata(metadata, "relation_types_strict", strict);
}

/// Apply `kg_schema_preset` (SPEC-114).
pub(crate) fn apply_kg_schema_preset_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    preset: Option<String>,
) -> Result<(), String> {
    apply_kg_schema_preset_metadata_inner(metadata, preset)
}

/// Apply `relation_edges` (SPEC-114b). Call after entity/relation type lists.
pub(crate) fn apply_relation_edges_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    edges: Option<Vec<RelationEdge>>,
) {
    apply_relation_edges_metadata_inner(metadata, edges);
}

// SPEC-102 color helpers live in `crate::entity_type_colors` (shared with in-memory).

/// Apply `extraction_language` to workspace metadata (SPEC-096 / GH-352).
///
/// - `None` → leave unchanged
/// - `""` / `"none"` → remove key (inherit env/default)
/// - allowlisted value → store canonical display name
/// - unsupported → `Err` (API 400)
pub(crate) fn apply_extraction_language_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    language: Option<String>,
) -> Result<(), String> {
    let Some(raw) = language else {
        return Ok(());
    };
    if edgequake_pipeline::is_extraction_language_clear(&raw) {
        metadata.remove("extraction_language");
        return Ok(());
    }
    match edgequake_pipeline::canonicalize_extraction_language(&raw) {
        Some(canonical) => {
            metadata.insert(
                "extraction_language".to_string(),
                serde_json::json!(canonical),
            );
            Ok(())
        }
        None => Err(format!(
            "Unsupported extraction_language '{}'. Allowed values: {}",
            raw.trim(),
            edgequake_pipeline::SUPPORTED_LANGUAGES.join(", ")
        )),
    }
}

/// @implements SPEC-085: Custom entity configuration normalization
/// Apply SPEC-109 default reasoning effort to workspace metadata.
pub(crate) fn apply_default_reasoning_effort_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    effort: Option<String>,
) {
    let Some(raw) = effort else {
        return;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("auto")
    {
        metadata.remove("default_reasoning_effort");
    } else {
        metadata.insert(
            "default_reasoning_effort".to_string(),
            serde_json::json!(trimmed),
        );
    }
}

/// Merge SPEC-109 `llm_roles` object into workspace metadata (shallow role merge).
pub(crate) fn apply_llm_roles_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    roles: Option<serde_json::Value>,
) {
    let Some(incoming) = roles else {
        return;
    };
    if incoming.is_null() {
        metadata.remove("llm_roles");
        return;
    }
    let Some(incoming_obj) = incoming.as_object() else {
        return;
    };
    if incoming_obj.is_empty() {
        metadata.remove("llm_roles");
        return;
    }
    let mut base = metadata
        .get("llm_roles")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    for (role, cfg) in incoming_obj {
        if cfg.is_null() {
            base.remove(role);
            continue;
        }
        let Some(cfg_obj) = cfg.as_object() else {
            continue;
        };
        let entry = base
            .entry(role.clone())
            .or_insert_with(|| serde_json::json!({}));
        if let Some(existing) = entry.as_object_mut() {
            for (k, v) in cfg_obj {
                if v.is_null() {
                    existing.remove(k);
                } else {
                    existing.insert(k.clone(), v.clone());
                }
            }
            if existing.is_empty() {
                base.remove(role);
            }
        }
    }
    if base.is_empty() {
        metadata.remove("llm_roles");
    } else {
        metadata.insert("llm_roles".to_string(), serde_json::Value::Object(base));
    }
}

/// Alias for [`normalize_type_list`] (SPEC-085 / SPEC-114 DRY).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn normalize_entity_types(types: &[String]) -> Vec<String> {
    normalize_type_list(types)
}
