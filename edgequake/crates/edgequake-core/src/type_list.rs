//! Shared normalization and metadata helpers for workspace type allow-lists.
//!
//! @implements SPEC-114: entity_types + relation_types DRY normalize/cap
//! @implements SPEC-114b: relation_edges normalize/cap
//! @implements SPEC-085: entity type normalization (via alias)

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Maximum entries for entity or relation type allow-lists.
pub const MAX_TYPE_LIST: usize = 50;

/// Maximum typed edges per workspace (SPEC-114b).
pub const MAX_RELATION_EDGES: usize = 100;

/// Typed edge constraint: source entity type — relation → target entity type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationEdge {
    pub source: String,
    pub relation: String,
    pub target: String,
}

impl RelationEdge {
    /// Normalize tokens on a single edge (may yield empty fields).
    pub fn normalize_tokens(&self) -> Self {
        Self {
            source: normalize_type_token_one(&self.source),
            relation: normalize_type_token_one(&self.relation),
            target: normalize_type_token_one(&self.target),
        }
    }
}

fn normalize_type_token_one(raw: &str) -> String {
    raw.trim()
        .to_uppercase()
        .replace([' ', '-'], "_")
        .trim_matches('_')
        .to_string()
}

/// Normalize type tokens for storage (entity or relation).
///
/// Rules:
/// - Trim whitespace
/// - Convert to UPPERCASE
/// - Replace spaces/hyphens with underscores
/// - Skip empty strings
/// - Deduplicate (preserving first occurrence order)
/// - Cap at [`MAX_TYPE_LIST`]
pub fn normalize_type_list(types: &[String]) -> Vec<String> {
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
        .take(MAX_TYPE_LIST)
        .collect()
}

/// Apply a string-list allow-list to workspace metadata under `key`.
///
/// - `None` → leave unchanged
/// - empty after normalize → remove key
/// - otherwise → store JSON array
pub fn apply_type_list_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    key: &str,
    types: Option<Vec<String>>,
) {
    if let Some(types) = types {
        let normalized = normalize_type_list(&types);
        if normalized.is_empty() {
            metadata.remove(key);
        } else {
            metadata.insert(key.to_string(), serde_json::json!(normalized));
        }
    }
}

/// Apply sparse strict flag (default true when key absent).
///
/// - `None` → leave unchanged
/// - `true` → remove key
/// - `false` → store `false`
pub fn apply_type_list_strict_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    key: &str,
    strict: Option<bool>,
) {
    if let Some(strict) = strict {
        if strict {
            metadata.remove(key);
        } else {
            metadata.insert(key.to_string(), serde_json::json!(false));
        }
    }
}

/// Allowed `kg_schema_preset` ids (SPEC-114).
pub const KG_SCHEMA_PRESET_IDS: &[&str] = &[
    "blank",
    "general",
    "manufacturing",
    "healthcare",
    "legal",
    "research",
    "finance",
    "custom",
];

/// Normalize typed edges: UPPER_SNAKE, drop empties/invalids, dedupe, cap.
///
/// When `entity_allow` / `relation_allow` are non-empty, edges referencing
/// unknown tokens are dropped (EC-114-16). Empty allow slices = no filter.
pub fn normalize_relation_edges(
    edges: &[RelationEdge],
    entity_allow: &[String],
    relation_allow: &[String],
) -> Vec<RelationEdge> {
    let entity_set: HashSet<String> = entity_allow
        .iter()
        .map(|t| normalize_type_token_one(t))
        .filter(|t| !t.is_empty())
        .collect();
    let relation_set: HashSet<String> = relation_allow
        .iter()
        .map(|t| normalize_type_token_one(t))
        .filter(|t| !t.is_empty())
        .collect();

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for edge in edges {
        let n = edge.normalize_tokens();
        if n.source.is_empty() || n.relation.is_empty() || n.target.is_empty() {
            continue;
        }
        if !entity_set.is_empty()
            && (!entity_set.contains(&n.source) || !entity_set.contains(&n.target))
        {
            continue;
        }
        if !relation_set.is_empty() && !relation_set.contains(&n.relation) {
            continue;
        }
        if !seen.insert(n.clone()) {
            continue;
        }
        out.push(n);
        if out.len() >= MAX_RELATION_EDGES {
            break;
        }
    }
    out
}

/// Apply `relation_edges` to workspace metadata (SPEC-114b).
///
/// - `None` → leave unchanged
/// - empty after normalize → remove key
/// - otherwise → store JSON array
///
/// Uses current `entity_types` / `relation_types` in metadata as allow-lists
/// when present (apply type lists before calling this).
pub fn apply_relation_edges_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    edges: Option<Vec<RelationEdge>>,
) {
    let Some(edges) = edges else {
        return;
    };
    let entity_allow = metadata
        .get("entity_types")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let relation_allow = metadata
        .get("relation_types")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let normalized = normalize_relation_edges(&edges, &entity_allow, &relation_allow);
    if normalized.is_empty() {
        metadata.remove("relation_edges");
    } else {
        metadata.insert(
            "relation_edges".to_string(),
            serde_json::to_value(normalized).unwrap_or(serde_json::json!([])),
        );
    }
}

/// Apply optional `kg_schema_preset` metadata key.
///
/// - `None` → leave unchanged
/// - `""` / `"none"` → remove key
/// - allowlisted id → store lowercase
/// - unknown → `Err` (API 400)
pub fn apply_kg_schema_preset_metadata(
    metadata: &mut HashMap<String, serde_json::Value>,
    preset: Option<String>,
) -> Result<(), String> {
    let Some(raw) = preset else {
        return Ok(());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        metadata.remove("kg_schema_preset");
        return Ok(());
    }
    let lower = trimmed.to_ascii_lowercase();
    if KG_SCHEMA_PRESET_IDS.contains(&lower.as_str()) {
        metadata.insert("kg_schema_preset".to_string(), serde_json::json!(lower));
        Ok(())
    } else {
        Err(format!(
            "Unsupported kg_schema_preset '{}'. Allowed values: {}",
            trimmed,
            KG_SCHEMA_PRESET_IDS.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_type_list_trims_dedupes_and_caps() {
        let input = vec![
            " person ".to_string(),
            "PERSON".to_string(),
            "org-unit".to_string(),
            "".to_string(),
            "   ".to_string(),
        ];
        let out = normalize_type_list(&input);
        assert_eq!(out, vec!["PERSON".to_string(), "ORG_UNIT".to_string()]);
    }

    #[test]
    fn normalize_type_list_respects_max_fifty() {
        let input: Vec<String> = (0..60).map(|i| format!("type_{i}")).collect();
        assert_eq!(normalize_type_list(&input).len(), 50);
    }

    #[test]
    fn apply_relation_types_empty_clears() {
        let mut meta = HashMap::new();
        meta.insert(
            "relation_types".to_string(),
            serde_json::json!(["WORKS_AT"]),
        );
        apply_type_list_metadata(&mut meta, "relation_types", Some(vec![]));
        assert!(!meta.contains_key("relation_types"));
    }

    #[test]
    fn apply_relation_types_strict_sparse() {
        let mut meta = HashMap::new();
        apply_type_list_strict_metadata(&mut meta, "relation_types_strict", Some(false));
        assert_eq!(
            meta.get("relation_types_strict"),
            Some(&serde_json::json!(false))
        );
        apply_type_list_strict_metadata(&mut meta, "relation_types_strict", Some(true));
        assert!(!meta.contains_key("relation_types_strict"));
    }

    #[test]
    fn apply_kg_schema_preset_round_trip() {
        let mut meta = HashMap::new();
        apply_kg_schema_preset_metadata(&mut meta, Some("Manufacturing".into())).unwrap();
        assert_eq!(
            meta.get("kg_schema_preset").and_then(|v| v.as_str()),
            Some("manufacturing")
        );
        apply_kg_schema_preset_metadata(&mut meta, Some("blank".into())).unwrap();
        assert_eq!(
            meta.get("kg_schema_preset").and_then(|v| v.as_str()),
            Some("blank")
        );
        apply_kg_schema_preset_metadata(&mut meta, Some("none".into())).unwrap();
        assert!(!meta.contains_key("kg_schema_preset"));
    }

    #[test]
    fn apply_kg_schema_preset_rejects_unknown() {
        let mut meta = HashMap::new();
        let err = apply_kg_schema_preset_metadata(&mut meta, Some("aliens".into())).unwrap_err();
        assert!(err.contains("Unsupported"));
    }

    #[test]
    fn normalize_relation_edges_dedupes_filters_and_caps() {
        let entities = vec!["PERSON".into(), "ORGANIZATION".into()];
        let relations = vec!["WORKS_AT".into(), "RELATED_TO".into()];
        let edges = vec![
            RelationEdge {
                source: " person ".into(),
                relation: "works-at".into(),
                target: "ORGANIZATION".into(),
            },
            RelationEdge {
                source: "PERSON".into(),
                relation: "WORKS_AT".into(),
                target: "ORGANIZATION".into(),
            },
            RelationEdge {
                source: "PERSON".into(),
                relation: "FRIENDS_WITH".into(),
                target: "ORGANIZATION".into(),
            },
            RelationEdge {
                source: "ALIEN".into(),
                relation: "WORKS_AT".into(),
                target: "ORGANIZATION".into(),
            },
        ];
        let out = normalize_relation_edges(&edges, &entities, &relations);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].relation, "WORKS_AT");
    }

    #[test]
    fn normalize_relation_edges_respects_max_hundred() {
        let entities = vec!["A".into(), "B".into()];
        let relations = vec!["R".into()];
        let edges: Vec<RelationEdge> = (0..120)
            .map(|i| RelationEdge {
                source: "A".into(),
                relation: "R".into(),
                target: format!("B_{i}"),
            })
            .collect();
        // targets not in allow → all dropped when entity allow non-empty
        let filtered = normalize_relation_edges(&edges, &entities, &relations);
        assert!(filtered.is_empty());
        // no entity filter → cap 100
        let capped = normalize_relation_edges(&edges, &[], &relations);
        assert_eq!(capped.len(), 100);
    }

    #[test]
    fn apply_relation_edges_empty_clears() {
        let mut meta = HashMap::new();
        meta.insert(
            "relation_edges".to_string(),
            serde_json::json!([{
                "source": "PERSON",
                "relation": "WORKS_AT",
                "target": "ORGANIZATION"
            }]),
        );
        apply_relation_edges_metadata(&mut meta, Some(vec![]));
        assert!(!meta.contains_key("relation_edges"));
    }
}
