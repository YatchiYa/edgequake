//! Community report vectors for L3 global retrieval (SPEC-046 EQ-046-11).
//!
//! First principle: community **labels** enable expansion; community **reports**
//! are optional summary indexes for thematic queries. Reports are opt-in via
//! `EDGEQUAKE_COMMUNITY_REPORTS=true` and never block ingest.
//!
//! SOLID: this module only builds report text + vector metadata; LLM summary
//! injection is optional via a callback so merger/query stay decoupled.

use serde_json::{json, Map, Value};

use crate::community::CommunityDetectionResult;

/// Vector metadata `type` for community reports.
pub const COMMUNITY_REPORT_VECTOR_TYPE: &str = "community_report";

/// Whether to generate/upsert community report vectors (default: **off**).
pub fn community_reports_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_COMMUNITY_REPORTS")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "true" | "1" | "on"
    )
}

/// Deterministic extractive report from member entity names (no LLM).
///
/// Used as a safe default; callers may replace with LLM summary later.
pub fn build_extractive_community_report(
    community_id: usize,
    member_names: &[String],
    max_members: usize,
) -> String {
    let shown: Vec<&str> = member_names
        .iter()
        .take(max_members.max(1))
        .map(String::as_str)
        .collect();
    let more = member_names.len().saturating_sub(shown.len());
    let list = shown.join(", ");
    if more > 0 {
        format!(
            "Community {community_id} ({n} entities): {list}, and {more} more.",
            n = member_names.len()
        )
    } else {
        format!(
            "Community {community_id} ({n} entities): {list}.",
            n = member_names.len()
        )
    }
}

/// Build vector id for a community report (stable, workspace-agnostic).
pub fn community_report_vector_id(community_id: usize) -> String {
    format!("community_report:{community_id}")
}

/// Metadata for upserting a community report vector (DRY with entity/chunk meta).
pub fn community_report_vector_metadata(
    community_id: usize,
    report_text: &str,
    member_count: usize,
    workspace_id: Option<&str>,
    tenant_id: Option<&str>,
) -> Value {
    let mut map = Map::new();
    map.insert("type".into(), json!(COMMUNITY_REPORT_VECTOR_TYPE));
    map.insert("community_id".into(), json!(community_id));
    map.insert("member_count".into(), json!(member_count));
    map.insert("content".into(), json!(report_text));
    if let Some(ws) = workspace_id {
        map.insert("workspace_id".into(), json!(ws));
    }
    if let Some(t) = tenant_id {
        map.insert("tenant_id".into(), json!(t));
    }
    Value::Object(map)
}

/// Produce (vector_id, report_text, metadata) triples from a detection result.
pub fn build_community_report_records(
    result: &CommunityDetectionResult,
    max_members_in_text: usize,
    workspace_id: Option<&str>,
    tenant_id: Option<&str>,
) -> Vec<(String, String, Value)> {
    let mut by_community: std::collections::BTreeMap<usize, Vec<String>> =
        std::collections::BTreeMap::new();
    for (node_id, &cid) in &result.node_to_community {
        by_community.entry(cid).or_default().push(node_id.clone());
    }

    by_community
        .into_iter()
        .filter(|(_, members)| !members.is_empty())
        .map(|(cid, mut members)| {
            members.sort();
            let text = build_extractive_community_report(cid, &members, max_members_in_text);
            let meta = community_report_vector_metadata(
                cid,
                &text,
                members.len(),
                workspace_id,
                tenant_id,
            );
            (community_report_vector_id(cid), text, meta)
        })
        .collect()
}

/// Pack report records with embeddings into VectorStorage upsert rows (DRY/SOLID).
///
/// Caller owns embedding (DIP): storage never depends on an LLM crate.
/// Lengths must match; extras on either side are ignored.
pub fn pack_community_report_vectors(
    records: &[(String, String, Value)],
    embeddings: &[Vec<f32>],
) -> Vec<(String, Vec<f32>, Value)> {
    records
        .iter()
        .zip(embeddings.iter())
        .filter(|((_, text, _), emb)| !text.is_empty() && !emb.is_empty())
        .map(|((id, _text, meta), emb)| (id.clone(), emb.clone(), meta.clone()))
        .collect()
}

/// Upsert packed community report vectors (non-fatal helper for ingest hooks).
pub async fn upsert_community_report_vectors(
    vector_storage: &dyn crate::traits::VectorStorage,
    rows: &[(String, Vec<f32>, Value)],
) -> crate::error::Result<usize> {
    if rows.is_empty() {
        return Ok(0);
    }
    vector_storage.upsert(rows).await?;
    Ok(rows.len())
}

/// Build extractive reports, embed via [`TextEmbedder`], and upsert vectors.
///
/// First principle: opt-in (`community_reports_enabled`); never blocks callers
/// that skip this path. SOLID: orchestration only — text build / pack / upsert
/// stay in focused helpers (DRY).
pub async fn index_community_reports_with_embedder(
    result: &CommunityDetectionResult,
    vector_storage: &dyn crate::traits::VectorStorage,
    embedder: &dyn crate::traits::TextEmbedder,
    workspace_id: Option<&str>,
    tenant_id: Option<&str>,
) -> crate::error::Result<usize> {
    if !community_reports_enabled() {
        return Ok(0);
    }
    let records = build_community_report_records(result, 24, workspace_id, tenant_id);
    if records.is_empty() {
        return Ok(0);
    }
    let texts: Vec<String> = records.iter().map(|(_, t, _)| t.clone()).collect();
    let embeddings = embedder.embed_texts(&texts).await?;
    if embeddings.len() != records.len() {
        return Err(crate::error::StorageError::InvalidInput(format!(
            "community report embed count mismatch: texts={} embeddings={}",
            records.len(),
            embeddings.len()
        )));
    }
    let packed = pack_community_report_vectors(&records, &embeddings);
    upsert_community_report_vectors(vector_storage, &packed).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::community::{Community, CommunityDetectionResult};
    use std::collections::HashMap;

    #[test]
    fn extractive_report_lists_members() {
        let names = vec!["A".into(), "B".into(), "C".into()];
        let t = build_extractive_community_report(0, &names, 2);
        assert!(t.contains("A"));
        assert!(t.contains("and 1 more"));
    }

    #[test]
    fn builds_records_from_detection() {
        let mut map = HashMap::new();
        map.insert("ALPHA".into(), 0usize);
        map.insert("BETA".into(), 0usize);
        map.insert("GAMMA".into(), 1usize);
        let result = CommunityDetectionResult {
            communities: vec![
                Community {
                    id: 0,
                    members: vec!["ALPHA".into(), "BETA".into()],
                    properties: HashMap::new(),
                },
                Community {
                    id: 1,
                    members: vec!["GAMMA".into()],
                    properties: HashMap::new(),
                },
            ],
            node_to_community: map,
            modularity: 0.0,
            hierarchy_levels: 1,
        };
        let records = build_community_report_records(&result, 10, Some("ws"), None);
        assert_eq!(records.len(), 2);
        assert!(records[0].0.starts_with("community_report:"));
        assert_eq!(
            records[0].2.get("type").and_then(|v| v.as_str()),
            Some(COMMUNITY_REPORT_VECTOR_TYPE)
        );
    }

    #[test]
    fn pack_aligns_embeddings_with_records() {
        let records = vec![(
            "community_report:0".into(),
            "Community 0".into(),
            community_report_vector_metadata(0, "Community 0", 2, None, None),
        )];
        let emb = vec![vec![0.1_f32, 0.2, 0.3]];
        let packed = pack_community_report_vectors(&records, &emb);
        assert_eq!(packed.len(), 1);
        assert_eq!(packed[0].0, "community_report:0");
        assert_eq!(packed[0].1.len(), 3);
    }
}
