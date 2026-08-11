//! LightRAG-parity per-response extraction quantity caps (054 / SPEC-117).
//!
//! Defaults match LightRAG `DEFAULT_MAX_EXTRACTION_ENTITIES=40` and
//! `DEFAULT_MAX_EXTRACTION_RECORDS=100`. Override via
//! `EDGEQUAKE_MAX_EXTRACTION_ENTITIES` / `EDGEQUAKE_MAX_EXTRACTION_RECORDS`.
//!
//! Precedence (LAW-117-2): document API > workspace metadata > fleet env > 40/100.
//!
//! Hard truncate selection (LAW-117-8): default **relation-aware** (prefer
//! relation-bearing entities by degree). Acc / LightRAG FIFO parity via
//! `EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo`.

use std::collections::{HashMap, HashSet};

use crate::extractor::{ExtractedEntity, ExtractedRelationship, ExtractionResult};

/// LightRAG default: max entity rows per LLM extraction response.
pub const DEFAULT_MAX_EXTRACTION_ENTITIES: usize = 40;

/// LightRAG default: max total entity+relationship rows per response.
pub const DEFAULT_MAX_EXTRACTION_RECORDS: usize = 100;

/// Workspace / document metadata keys (SPEC-117).
pub const META_EXTRACT_MAX_ENTITIES: &str = "extract_max_entities";
pub const META_EXTRACT_MAX_RECORDS: &str = "extract_max_records";

/// Env pin for Acc / LightRAG FIFO hard-truncate parity.
pub const EXTRACT_CAPS_SELECTION_ENV: &str = "EDGEQUAKE_EXTRACT_CAPS_SELECTION";

/// Who survives when the model emits more rows than \(K\) (LAW-117-8 / LAW-B3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsSelectionStrategy {
    /// Keep first \(K\) entities in response order (LightRAG Acc parity).
    Fifo,
    /// Prefer relation-bearing entities (higher degree first), then fill with
    /// orphans in response order. Restores original relative order among the
    /// selected set. Default product behavior.
    #[default]
    RelationAware,
}

impl CapsSelectionStrategy {
    /// `fifo` → Acc/LR parity; anything else (including unset) → relation-aware.
    pub fn from_env() -> Self {
        match std::env::var(EXTRACT_CAPS_SELECTION_ENV)
            .ok()
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("fifo") => Self::Fifo,
            _ => Self::RelationAware,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fifo => "fifo",
            Self::RelationAware => "relation_aware",
        }
    }
}

/// Resolved caps (env override, workspace, or document).
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

    /// Validate and construct caps (entities ≥ 1, records ≥ entities).
    pub fn validate(max_entities: usize, max_total_records: usize) -> Result<Self, String> {
        if max_entities < 1 {
            return Err("extract_max_entities must be >= 1".into());
        }
        if max_total_records < max_entities {
            return Err("extract_max_records must be >= extract_max_entities".into());
        }
        Ok(Self {
            max_entities,
            max_total_records,
        })
    }

    /// Resolve LAW-117-2 precedence: document > workspace > fleet env.
    pub fn resolve(workspace: Option<&ExtractionCaps>, document: Option<&ExtractionCaps>) -> Self {
        if let Some(c) = document {
            return *c;
        }
        if let Some(c) = workspace {
            return *c;
        }
        Self::from_env()
    }

    /// Resolve from workspace metadata + optional document override (ingestion SSOT).
    pub fn resolve_for_ingestion(
        workspace_metadata: &HashMap<String, serde_json::Value>,
        document: Option<ExtractionCaps>,
    ) -> Self {
        let workspace = Self::from_metadata(workspace_metadata);
        Self::resolve(workspace.as_ref(), document.as_ref())
    }

    /// Parse explicit workspace/document metadata pair (both keys required).
    ///
    /// Returns `None` when keys are absent or JSON null (Inherit). Returns `Err`
    /// when one key is present without the other or values fail validation.
    pub fn try_from_metadata(
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<Self>, String> {
        Self::try_from_pair(
            metadata.get(META_EXTRACT_MAX_ENTITIES),
            metadata.get(META_EXTRACT_MAX_RECORDS),
        )
    }

    /// Parse caps from a JSON object (task / staging metadata).
    pub fn try_from_value(metadata: &serde_json::Value) -> Result<Option<Self>, String> {
        let Some(obj) = metadata.as_object() else {
            return Ok(None);
        };
        Self::try_from_pair(
            obj.get(META_EXTRACT_MAX_ENTITIES),
            obj.get(META_EXTRACT_MAX_RECORDS),
        )
    }

    fn try_from_pair(
        ents: Option<&serde_json::Value>,
        recs: Option<&serde_json::Value>,
    ) -> Result<Option<Self>, String> {
        let ents = match ents {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(v),
        };
        let recs = match recs {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(v),
        };
        match (ents, recs) {
            (None, None) => Ok(None),
            (Some(_), None) | (None, Some(_)) => Err(
                "extract_max_entities and extract_max_records must both be set (or both omitted)"
                    .into(),
            ),
            (Some(e), Some(r)) => {
                let max_entities = e
                    .as_u64()
                    .ok_or_else(|| "extract_max_entities must be a positive integer".to_string())?
                    as usize;
                let max_total_records = r
                    .as_u64()
                    .ok_or_else(|| "extract_max_records must be a positive integer".to_string())?
                    as usize;
                Ok(Some(Self::validate(max_entities, max_total_records)?))
            }
        }
    }

    /// Convenience: `Ok(None)` / `Ok(Some)` / ignore invalid as None for soft reads.
    pub fn from_metadata(metadata: &HashMap<String, serde_json::Value>) -> Option<Self> {
        Self::try_from_metadata(metadata).ok().flatten()
    }

    /// Soft read from a JSON object (null / missing / invalid → None).
    pub fn from_value(metadata: &serde_json::Value) -> Option<Self> {
        Self::try_from_value(metadata).ok().flatten()
    }

    /// Prompt fragment shared by JSON and SOTA extractors (soft + ranking).
    pub fn prompt_quantity_limits_section(&self) -> String {
        format!(
            "## Quantity Limits (STRICT)\n\
             - Output at most {max_ents} entity records in this response.\n\
             - Output at most {max_total} total records across entities and relationships.\n\
             - Prioritize highest-value, relation-bearing entities first (core actors, \
               organizations, methods, and concepts that participate in relationships). \
               Emit them before lower-value or isolated mentions.\n\
             - Output fewer records if fewer high-value items are present. Do not try to fill the limit.\n\
             - Only output relationships whose source and target are both included in the selected entities for this response.\n\
             - If the limit is reached, stop adding new records immediately.",
            max_ents = self.max_entities,
            max_total = self.max_total_records,
        )
    }

    /// Extra gleaning instructions when the prior pass hard-truncated under caps.
    pub fn prompt_gleaning_continue_after_truncate_section(&self) -> String {
        format!(
            "## Continue After Budget Truncation\n\
             - The previous response hit the per-response budget \
               ({max_ents} entities / {max_total} total records) and was truncated.\n\
             - Identify ADDITIONAL high-value entities and relationships that were \
               NOT already listed above.\n\
             - Do not re-emit entities already identified. Prefer relation-bearing \
               entities that complete important graph edges.\n\
             - This continue pass has its own quantity budget (same limits).",
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

/// Deterministic post-parse truncate (SSOT hard safety net).
///
/// Uses [`CapsSelectionStrategy::from_env`] (default relation-aware).
pub fn apply_extraction_caps(result: &mut ExtractionResult, caps: ExtractionCaps) {
    apply_extraction_caps_with_strategy(result, caps, CapsSelectionStrategy::from_env());
}

/// Apply caps with an explicit selection strategy (unit tests / Acc pins).
///
/// 1. Select ≤ `max_entities` entities (FIFO or relation-aware).
/// 2. Drop relationships whose endpoints are not in the kept entity set.
/// 3. Cap total rows (`entities + relationships`) at `max_total_records`
///    by trimming relationships (higher weight first, then response order).
pub fn apply_extraction_caps_with_strategy(
    result: &mut ExtractionResult,
    caps: ExtractionCaps,
    strategy: CapsSelectionStrategy,
) {
    let before_ents = result.entities.len();
    let before_rels = result.relationships.len();

    if result.entities.len() > caps.max_entities {
        result.entities = select_entities_under_cap(
            &result.entities,
            &result.relationships,
            caps.max_entities,
            strategy,
        );
    }

    let kept: HashSet<&str> = result.entities.iter().map(|e| e.name.as_str()).collect();
    result
        .relationships
        .retain(|r| kept.contains(r.source.as_str()) && kept.contains(r.target.as_str()));

    let max_rels = caps.max_total_records.saturating_sub(result.entities.len());
    if result.relationships.len() > max_rels {
        result.relationships = select_relationships_under_cap(&result.relationships, max_rels);
    }

    let truncated =
        before_ents != result.entities.len() || before_rels != result.relationships.len();
    if truncated {
        result.metadata.insert(
            "extract_caps_applied".to_string(),
            serde_json::json!({
                "max_entities": caps.max_entities,
                "max_total_records": caps.max_total_records,
                "selection": strategy.as_str(),
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
            selection = strategy.as_str(),
            "Applied extract quantity caps"
        );
    }
}

/// Select entities under \(K\): FIFO head or relation-aware (degree then order).
fn select_entities_under_cap(
    entities: &[ExtractedEntity],
    relationships: &[ExtractedRelationship],
    max_entities: usize,
    strategy: CapsSelectionStrategy,
) -> Vec<ExtractedEntity> {
    if entities.len() <= max_entities {
        return entities.to_vec();
    }
    match strategy {
        CapsSelectionStrategy::Fifo => entities[..max_entities].to_vec(),
        CapsSelectionStrategy::RelationAware => {
            select_entities_relation_aware(entities, relationships, max_entities)
        }
    }
}

/// Prefer relation-bearing entities (higher degree), then orphans in order.
/// Selected set is re-sorted by original response index for stable output.
fn select_entities_relation_aware(
    entities: &[ExtractedEntity],
    relationships: &[ExtractedRelationship],
    max_entities: usize,
) -> Vec<ExtractedEntity> {
    let mut degree: HashMap<&str, usize> = HashMap::new();
    for r in relationships {
        *degree.entry(r.source.as_str()).or_default() += 1;
        *degree.entry(r.target.as_str()).or_default() += 1;
    }

    let mut connected: Vec<(usize, usize)> = Vec::new();
    let mut orphans: Vec<usize> = Vec::new();
    for (idx, e) in entities.iter().enumerate() {
        let d = degree.get(e.name.as_str()).copied().unwrap_or(0);
        if d > 0 {
            connected.push((idx, d));
        } else {
            orphans.push(idx);
        }
    }
    // Higher degree first; stable by original index on ties (prompt rank).
    connected.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut selected_idx: Vec<usize> = Vec::with_capacity(max_entities);
    for (idx, _) in connected {
        if selected_idx.len() >= max_entities {
            break;
        }
        selected_idx.push(idx);
    }
    for idx in orphans {
        if selected_idx.len() >= max_entities {
            break;
        }
        selected_idx.push(idx);
    }
    selected_idx.sort_unstable();
    selected_idx
        .into_iter()
        .map(|i| entities[i].clone())
        .collect()
}

/// Prefer higher-weight relationships; ties keep response order.
fn select_relationships_under_cap(
    relationships: &[ExtractedRelationship],
    max_rels: usize,
) -> Vec<ExtractedRelationship> {
    if relationships.len() <= max_rels {
        return relationships.to_vec();
    }
    let mut indexed: Vec<(usize, &ExtractedRelationship)> =
        relationships.iter().enumerate().collect();
    indexed.sort_by(|a, b| {
        b.1.weight
            .partial_cmp(&a.1.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    let mut keep: Vec<usize> = indexed.into_iter().take(max_rels).map(|(i, _)| i).collect();
    keep.sort_unstable();
    keep.into_iter().map(|i| relationships[i].clone()).collect()
}

/// Apply default (env-resolved) caps + selection strategy.
pub fn apply_default_extraction_caps(result: &mut ExtractionResult) {
    apply_extraction_caps(result, ExtractionCaps::from_env());
}

/// True when hard truncate metadata is present on an extraction result.
pub fn extract_caps_were_applied(result: &ExtractionResult) -> bool {
    result.metadata.contains_key("extract_caps_applied")
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
    fn fifo_keeps_first_n_entities_and_drops_orphan_rels() {
        let mut result = ExtractionResult::new("c1");
        for i in 0..45 {
            result.add_entity(ent(&format!("E{i}")));
        }
        result.add_relationship(rel("E0", "E1"));
        result.add_relationship(rel("E0", "E44")); // orphan after fifo truncate
        result.add_relationship(rel("E40", "E41")); // both outside kept set

        apply_extraction_caps_with_strategy(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 100,
            },
            CapsSelectionStrategy::Fifo,
        );

        assert_eq!(result.entities.len(), 40);
        assert_eq!(result.entities[39].name, "E39");
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].source, "E0");
        assert_eq!(result.relationships[0].target, "E1");
        let meta = result.metadata.get("extract_caps_applied").unwrap();
        assert_eq!(meta["selection"], "fifo");
    }

    #[test]
    fn relation_aware_prefers_late_connected_entities() {
        // Orphans first (would monopolize FIFO), bridges only at the tail.
        let mut result = ExtractionResult::new("c1");
        for i in 0..45 {
            result.add_entity(ent(&format!("E{i}")));
        }
        result.add_relationship(rel("E42", "E43"));
        result.add_relationship(rel("E43", "E44").with_weight(0.9));

        apply_extraction_caps_with_strategy(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 100,
            },
            CapsSelectionStrategy::RelationAware,
        );

        assert_eq!(result.entities.len(), 40);
        let names: HashSet<_> = result.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains("E42"));
        assert!(names.contains("E43"));
        assert!(names.contains("E44"));
        assert!(!names.contains("E39")); // displaced orphan under relation-aware
        assert_eq!(result.relationships.len(), 2);
        let meta = result.metadata.get("extract_caps_applied").unwrap();
        assert_eq!(meta["selection"], "relation_aware");
    }

    #[test]
    fn relation_aware_trims_rels_by_weight_under_total_cap() {
        let mut result = ExtractionResult::new("c1");
        for i in 0..5 {
            result.add_entity(ent(&format!("E{i}")));
        }
        result.add_relationship(rel("E0", "E1").with_weight(0.1));
        result.add_relationship(rel("E0", "E2").with_weight(0.9));
        result.add_relationship(rel("E0", "E3").with_weight(0.5));

        apply_extraction_caps_with_strategy(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 7, // 5 ents → at most 2 rels
            },
            CapsSelectionStrategy::RelationAware,
        );

        assert_eq!(result.entities.len(), 5);
        assert_eq!(result.relationships.len(), 2);
        let weights: Vec<f32> = result.relationships.iter().map(|r| r.weight).collect();
        assert!(weights.contains(&0.9));
        assert!(weights.contains(&0.5));
        assert!(!weights.contains(&0.1));
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

        apply_extraction_caps_with_strategy(
            &mut result,
            ExtractionCaps {
                max_entities: 40,
                max_total_records: 15, // 10 ents → at most 5 rels
            },
            CapsSelectionStrategy::Fifo,
        );

        assert_eq!(result.entities.len(), 10);
        assert_eq!(result.relationships.len(), 5);
    }

    #[test]
    fn prompt_section_mentions_defaults_and_ranking() {
        let caps = ExtractionCaps {
            max_entities: 40,
            max_total_records: 100,
        };
        let s = caps.prompt_quantity_limits_section();
        assert!(s.contains("40"));
        assert!(s.contains("100"));
        assert!(s.contains("Quantity Limits"));
        assert!(s.contains("highest-value") || s.contains("relation-bearing"));
    }

    #[test]
    fn resolve_precedence_document_over_workspace_over_env() {
        let ws = ExtractionCaps {
            max_entities: 60,
            max_total_records: 150,
        };
        let doc = ExtractionCaps {
            max_entities: 20,
            max_total_records: 50,
        };
        assert_eq!(ExtractionCaps::resolve(Some(&ws), Some(&doc)), doc);
        assert_eq!(ExtractionCaps::resolve(Some(&ws), None), ws);
        let env = ExtractionCaps::from_env();
        assert_eq!(ExtractionCaps::resolve(None, None), env);
    }

    #[test]
    fn validate_rejects_bad_pairs() {
        assert!(ExtractionCaps::validate(0, 10).is_err());
        assert!(ExtractionCaps::validate(50, 40).is_err());
        assert!(ExtractionCaps::validate(40, 100).is_ok());
    }

    #[test]
    fn metadata_round_trip() {
        let mut meta = HashMap::new();
        assert!(ExtractionCaps::try_from_metadata(&meta).unwrap().is_none());
        meta.insert(META_EXTRACT_MAX_ENTITIES.into(), serde_json::json!(40));
        assert!(ExtractionCaps::try_from_metadata(&meta).is_err());
        meta.insert(META_EXTRACT_MAX_RECORDS.into(), serde_json::json!(100));
        let caps = ExtractionCaps::try_from_metadata(&meta).unwrap().unwrap();
        assert_eq!(caps.max_entities, 40);
        assert_eq!(caps.max_total_records, 100);
    }

    #[test]
    fn resolve_for_ingestion_document_wins_over_workspace() {
        let mut meta = HashMap::new();
        meta.insert(META_EXTRACT_MAX_ENTITIES.into(), serde_json::json!(60));
        meta.insert(META_EXTRACT_MAX_RECORDS.into(), serde_json::json!(150));
        let doc = ExtractionCaps {
            max_entities: 20,
            max_total_records: 50,
        };
        let resolved = ExtractionCaps::resolve_for_ingestion(&meta, Some(doc));
        assert_eq!(resolved, doc);
        let ws_only = ExtractionCaps::resolve_for_ingestion(&meta, None);
        assert_eq!(ws_only.max_entities, 60);
        assert_eq!(ws_only.max_total_records, 150);
    }

    #[test]
    fn null_json_keys_mean_inherit() {
        let value = serde_json::json!({
            "extract_max_entities": null,
            "extract_max_records": null,
            "title": "x",
        });
        assert!(ExtractionCaps::try_from_value(&value).unwrap().is_none());
    }

    #[test]
    fn from_value_reads_document_override() {
        let value = serde_json::json!({
            "extract_max_entities": 20,
            "extract_max_records": 50,
        });
        let caps = ExtractionCaps::from_value(&value).unwrap();
        assert_eq!(caps.max_entities, 20);
        assert_eq!(caps.max_total_records, 50);
    }
}
