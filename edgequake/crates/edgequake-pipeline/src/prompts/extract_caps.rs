//! LightRAG-parity per-response extraction quantity caps (054).
//!
//! Defaults match LightRAG `DEFAULT_MAX_EXTRACTION_ENTITIES=40` and
//! `DEFAULT_MAX_EXTRACTION_RECORDS=100`. Override via
//! `EDGEQUAKE_MAX_EXTRACTION_ENTITIES` / `EDGEQUAKE_MAX_EXTRACTION_RECORDS`.

use std::collections::HashSet;

use crate::extractor::ExtractionResult;

/// LightRAG default: max entity rows per LLM extraction response.
pub const DEFAULT_MAX_EXTRACTION_ENTITIES: usize = 40;

/// LightRAG default: max total entity+relationship rows per response.
pub const DEFAULT_MAX_EXTRACTION_RECORDS: usize = 100;

/// Resolved caps (env override or defaults).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractionCaps {
    pub max_entities: usize,
    pub max_total_records: usize,
}

impl Default for ExtractionCaps {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ExtractionCaps {
    /// Read env overrides; fall back to LightRAG defaults.
    pub fn from_env() -> Self {
        Self {
            max_entities: parse_usize_env(
                "EDGEQUAKE_MAX_EXTRACTION_ENTITIES",
                DEFAULT_MAX_EXTRACTION_ENTITIES,
            ),
            max_total_records: parse_usize_env(
                "EDGEQUAKE_MAX_EXTRACTION_RECORDS",
                DEFAULT_MAX_EXTRACTION_RECORDS,
            ),
        }
    }

    /// Prompt fragment shared by JSON and SOTA extractors.
    pub fn prompt_quantity_limits_section(&self) -> String {
        format!(
            "## Quantity Limits (STRICT)\n\
             - Output at most {max_ents} entity records in this response.\n\
             - Output at most {max_total} total records across entities and relationships.\n\
             - Output fewer records if fewer high-value items are present. Do not try to fill the limit.\n\
             - Only output relationships whose source and target are both included in the selected entities for this response.\n\
             - If the limit is reached, stop adding new records immediately.",
            max_ents = self.max_entities,
            max_total = self.max_total_records,
        )
    }
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(default)
}

/// Deterministic post-parse truncate matching LightRAG prompt contract.
///
/// 1. Keep first `max_entities` entities (response order).
/// 2. Drop relationships whose endpoints are not in the kept entity set.
/// 3. Cap total rows (`entities + relationships`) at `max_total_records`
///    by trimming relationships after entities are fixed.
pub fn apply_extraction_caps(result: &mut ExtractionResult, caps: ExtractionCaps) {
    let before_ents = result.entities.len();
    let before_rels = result.relationships.len();

    if result.entities.len() > caps.max_entities {
        result.entities.truncate(caps.max_entities);
    }

    let kept: HashSet<&str> = result.entities.iter().map(|e| e.name.as_str()).collect();
    result
        .relationships
        .retain(|r| kept.contains(r.source.as_str()) && kept.contains(r.target.as_str()));

    let max_rels = caps.max_total_records.saturating_sub(result.entities.len());
    if result.relationships.len() > max_rels {
        result.relationships.truncate(max_rels);
    }

    let truncated =
        before_ents != result.entities.len() || before_rels != result.relationships.len();
    if truncated {
        result.metadata.insert(
            "extract_caps_applied".to_string(),
            serde_json::json!({
                "max_entities": caps.max_entities,
                "max_total_records": caps.max_total_records,
                "entities_before": before_ents,
                "entities_after": result.entities.len(),
                "relationships_before": before_rels,
                "relationships_after": result.relationships.len(),
            }),
        );
        tracing::debug!(
            entities_before = before_ents,
            entities_after = result.entities.len(),
            relationships_before = before_rels,
            relationships_after = result.relationships.len(),
            max_entities = caps.max_entities,
            max_total = caps.max_total_records,
            "Applied LightRAG extract quantity caps"
        );
    }
}

/// Apply default (env-resolved) caps.
pub fn apply_default_extraction_caps(result: &mut ExtractionResult) {
    apply_extraction_caps(result, ExtractionCaps::from_env());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::{ExtractedEntity, ExtractedRelationship};

    fn ent(name: &str) -> ExtractedEntity {
        ExtractedEntity::new(name, "CONCEPT", "d")
    }

    fn rel(src: &str, tgt: &str) -> ExtractedRelationship {
        ExtractedRelationship::new(src, tgt, "RELATED").with_description("d")
    }

    #[test]
    fn truncate_keeps_first_n_entities_and_drops_orphan_rels() {
        let mut result = ExtractionResult::new("c1");
        for i in 0..45 {
            result.add_entity(ent(&format!("E{i}")));
        }
        result.add_relationship(rel("E0", "E1"));
        result.add_relationship(rel("E0", "E44")); // orphan after truncate
        result.add_relationship(rel("E40", "E41")); // both outside kept set

        apply_extraction_caps(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 100,
            },
        );

        assert_eq!(result.entities.len(), 40);
        assert_eq!(result.entities[39].name, "E39");
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "E0");
        assert_eq!(result.relationships[0].target, "E1");
        assert!(result.metadata.contains_key("extract_caps_applied"));
    }

    #[test]
    fn total_row_cap_trims_relationships() {
        let mut result = ExtractionResult::new("c1");
        for i in 0..10 {
            result.add_entity(ent(&format!("E{i}")));
        }
        for i in 0..20 {
            result.add_relationship(rel("E0", &format!("E{}", (i % 9) + 1)));
        }

        apply_extraction_caps(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 15, // 10 ents → at most 5 rels
            },
        );

        assert_eq!(result.entities.len(), 10);
        assert_eq!(result.relationships.len(), 5);
    }

    #[test]
    fn prompt_section_mentions_defaults() {
        let caps = ExtractionCaps {
            max_entities: 40,
            max_total_records: 100,
        };
        let s = caps.prompt_quantity_limits_section();
        assert!(s.contains("40"));
        assert!(s.contains("100"));
        assert!(s.contains("Quantity Limits"));
    }
}
