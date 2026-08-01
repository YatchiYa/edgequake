//! Last-write-wins dedupe for graph batch upserts.
//!
//! # First Principles
//!
//! PostgreSQL `INSERT … ON CONFLICT DO UPDATE` is deterministic: a single
//! statement must not propose two rows that collide on the arbiter unique key
//! ([Postgres INSERT docs](https://www.postgresql.org/docs/current/sql-insert.html)).
//! Native AGE upserts use:
//! - Node: unique on `eq_node_id`
//! - Edge: unique on `(eq_source_id, eq_target_id, eq_rel_type)` (SPEC-083 D-30)
//!
//! Callers (merger, community persist, etc.) may still emit duplicates. This
//! module is the **single** place that collapses a batch to one row per key
//! before any adapter SQL/Cypher runs (DRY + SRP).
//!
//! # Policy
//!
//! Within a single batch: last-write-wins on the full property map (matches
//! Cypher `SET` / pre-SPEC-058 native LWW). Across concurrent statements,
//! Postgres native upsert uses `eq_merge_graph_properties` (SPEC-058) so
//! `source_ids` / `source_chunk_ids` are unioned rather than replaced.

use std::collections::HashMap;

/// Collapse `(node_id, properties)` so each `node_id` appears once (last wins).
pub fn dedupe_nodes_by_id(
    nodes: &[(String, HashMap<String, serde_json::Value>)],
) -> Vec<(String, HashMap<String, serde_json::Value>)> {
    let mut order: Vec<String> = Vec::new();
    let mut map: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();
    for (id, props) in nodes {
        if map.insert(id.clone(), props.clone()).is_none() {
            order.push(id.clone());
        }
    }
    order
        .into_iter()
        .filter_map(|id| map.remove(&id).map(|props| (id, props)))
        .collect()
}

/// Normalize a relation-type label (empty/whitespace → RELATED_TO, else ASCII upper).
///
/// SPEC-098 LAW-098-3: SSOT for sink, vector ids, and fleet FK lookup.
pub fn normalize_relation_type_str(relation_type: &str) -> String {
    let trimmed = relation_type.trim();
    if trimmed.is_empty() {
        "RELATED_TO".to_string()
    } else {
        trimmed.to_ascii_uppercase()
    }
}

/// Normalize relation type for multigraph keys (empty → RELATED_TO).
pub fn normalize_rel_type(props: &HashMap<String, serde_json::Value>) -> String {
    props
        .get("relation_type")
        .and_then(|v| v.as_str())
        .map(normalize_relation_type_str)
        .unwrap_or_else(|| "RELATED_TO".to_string())
}

/// Collapse `(source, target, properties)` so each `(src, tgt, rel_type)` appears
/// once (last wins). Matches `idx_edge_eq_source_target_rel` (D-30).
pub fn dedupe_edges_by_endpoints(
    edges: &[(String, String, HashMap<String, serde_json::Value>)],
) -> Vec<(String, String, HashMap<String, serde_json::Value>)> {
    let mut order: Vec<(String, String, String)> = Vec::new();
    let mut map: HashMap<(String, String, String), HashMap<String, serde_json::Value>> =
        HashMap::new();
    for (src, tgt, props) in edges {
        let rel = normalize_rel_type(props);
        let key = (src.clone(), tgt.clone(), rel);
        if map.insert(key.clone(), props.clone()).is_none() {
            order.push(key);
        }
    }
    order
        .into_iter()
        .filter_map(|key| {
            let (src, tgt, _) = key.clone();
            map.remove(&key).map(|props| (src, tgt, props))
        })
        .collect()
}

/// SPEC-047 P7f — default native/Cypher UNWIND chunk size (rows per statement).
pub const DEFAULT_GRAPH_UPSERT_CHUNK: usize = 500;
const MIN_GRAPH_UPSERT_CHUNK: usize = 50;
const MAX_GRAPH_UPSERT_CHUNK: usize = 2_000;

/// Tunable graph upsert chunk size (`EDGEQUAKE_GRAPH_UPSERT_CHUNK`).
///
/// Caps adaptive UNWIND / native `unnest` batches. Larger = fewer round-trips;
/// smaller = shorter locks / safer statement size.
pub fn graph_upsert_chunk_size() -> usize {
    parse_graph_upsert_chunk(&std::env::var("EDGEQUAKE_GRAPH_UPSERT_CHUNK").unwrap_or_default())
        .unwrap_or(DEFAULT_GRAPH_UPSERT_CHUNK)
}

/// Pure parser for `EDGEQUAKE_GRAPH_UPSERT_CHUNK` (testable).
pub fn parse_graph_upsert_chunk(raw: &str) -> Option<usize> {
    raw.trim()
        .parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
        .map(|n| n.clamp(MIN_GRAPH_UPSERT_CHUNK, MAX_GRAPH_UPSERT_CHUNK))
}

/// Combine adaptive size estimate with env cap (P7f SSOT).
///
/// When env is set, use `min(adaptive, env)`. When unset, clamp adaptive to
/// [`DEFAULT_GRAPH_UPSERT_CHUNK`].
pub fn resolve_graph_upsert_chunk(adaptive: usize) -> usize {
    let env_cap = parse_graph_upsert_chunk(
        &std::env::var("EDGEQUAKE_GRAPH_UPSERT_CHUNK").unwrap_or_default(),
    );
    match env_cap {
        Some(cap) => adaptive.min(cap).max(MIN_GRAPH_UPSERT_CHUNK),
        None => adaptive.clamp(MIN_GRAPH_UPSERT_CHUNK, DEFAULT_GRAPH_UPSERT_CHUNK),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(label: &str) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("label".to_string(), serde_json::json!(label));
        m
    }

    fn props_rel(label: &str, rel: &str) -> HashMap<String, serde_json::Value> {
        let mut m = props(label);
        m.insert("relation_type".to_string(), serde_json::json!(rel));
        m
    }

    #[test]
    fn parse_graph_upsert_chunk_clamps() {
        assert_eq!(parse_graph_upsert_chunk("100"), Some(100));
        assert_eq!(parse_graph_upsert_chunk("10"), Some(50));
        assert_eq!(parse_graph_upsert_chunk("99999"), Some(2000));
        assert_eq!(parse_graph_upsert_chunk(""), None);
        assert_eq!(parse_graph_upsert_chunk("0"), None);
    }

    #[test]
    fn spec098_normalize_relation_type_str() {
        assert_eq!(normalize_relation_type_str(""), "RELATED_TO");
        assert_eq!(normalize_relation_type_str("  "), "RELATED_TO");
        assert_eq!(normalize_relation_type_str("Works_With"), "WORKS_WITH");
        assert_eq!(normalize_relation_type_str("knows"), "KNOWS");
    }

    #[test]
    fn resolve_respects_adaptive_without_env() {
        assert_eq!(resolve_graph_upsert_chunk(250), 250);
        assert_eq!(resolve_graph_upsert_chunk(10), 50);
    }

    #[test]
    fn dedupe_edges_keeps_last_write_wins_and_order() {
        let edges = vec![
            ("A".into(), "B".into(), props("first")),
            ("C".into(), "D".into(), props("only")),
            ("A".into(), "B".into(), props("second")),
            ("A".into(), "B".into(), props("third")),
        ];
        let out = dedupe_edges_by_endpoints(&edges);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0, "A");
        assert_eq!(out[0].1, "B");
        assert_eq!(
            out[0].2.get("label").and_then(|v| v.as_str()),
            Some("third")
        );
        assert_eq!(out[1].0, "C");
        assert_eq!(out[1].1, "D");
    }

    #[test]
    fn dedupe_edges_multigraph_keeps_distinct_rel_types() {
        // D-30: KNOWS and WORKS_WITH between same endpoints both persist.
        let edges = vec![
            ("A".into(), "B".into(), props_rel("knows", "KNOWS")),
            ("A".into(), "B".into(), props_rel("works", "WORKS_WITH")),
        ];
        let out = dedupe_edges_by_endpoints(&edges);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn dedupe_nodes_keeps_last_write_wins() {
        let nodes = vec![
            ("N1".into(), props("a")),
            ("N2".into(), props("b")),
            ("N1".into(), props("c")),
        ];
        let out = dedupe_nodes_by_id(&nodes);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].0, "N1");
        assert_eq!(out[0].1.get("label").and_then(|v| v.as_str()), Some("c"));
        assert_eq!(out[1].0, "N2");
    }

    #[test]
    fn dedupe_edges_empty_and_unique_passthrough() {
        assert!(dedupe_edges_by_endpoints(&[]).is_empty());
        let edges = vec![("A".into(), "B".into(), props("x"))];
        assert_eq!(dedupe_edges_by_endpoints(&edges).len(), 1);
    }
}
