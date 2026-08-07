use std::collections::HashMap;

// ============ Helper Functions ============

/// Normalize entity types for storage.
///
/// WHY: Consistent normalization ensures that types like "machine" and "MACHINE"
/// map to the same entity type, preventing duplicate type entries in the graph.
///
/// Rules (per SPEC-085):
/// - Trim whitespace
/// - Convert to UPPERCASE
/// - Replace spaces/hyphens with underscores
/// - Skip empty strings
/// - Deduplicate (preserving first occurrence order)
/// - Cap at 50 types to avoid prompt bloat
///
/// Apply `entity_types` list to workspace metadata (create/update).
pub(crate) fn apply_entity_types_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    entity_types: Option<Vec<String>>,
) {
    if let Some(entity_types) = entity_types {
        let normalized = normalize_entity_types(&entity_types);
        if normalized.is_empty() {
            metadata.remove("entity_types");
        } else {
            metadata.insert("entity_types".to_string(), serde_json::json!(normalized));
        }
    }
}

/// Apply strict entity-type enforcement flag (default true when key absent).
pub(crate) fn apply_entity_types_strict_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    strict: Option<bool>,
) {
    if let Some(strict) = strict {
        if strict {
            metadata.remove("entity_types_strict");
        } else {
            metadata.insert("entity_types_strict".to_string(), serde_json::json!(false));
        }
    }
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

pub(crate) fn normalize_entity_types(types: &[String]) -> Vec<String> {
    const MAX_ENTITY_TYPES: usize = 50;

    let mut seen = std::collections::HashSet::new();
    types
        .iter()
        .filter_map(|t| {
            let normalized = t.trim().to_uppercase().replace([' ', '-'], "_");
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        })
        .filter(|t| seen.insert(t.clone()))
        .take(MAX_ENTITY_TYPES)
        .collect()
}
