//! Last-write-wins dedupe for graph batch upserts.
//!
//! # First Principles
//!
//! PostgreSQL `INSERT … ON CONFLICT DO UPDATE` is deterministic: a single
//! statement must not propose two rows that collide on the arbiter unique key
//! ([Postgres INSERT docs](https://www.postgresql.org/docs/current/sql-insert.html)).
//! Native AGE upserts use:
//! - Node: unique on `properties->>'node_id'`
//! - Edge: unique on `(properties->>'source_id', properties->>'target_id')`
//!
//! Callers (merger, community persist, etc.) may still emit duplicates. This
//! module is the **single** place that collapses a batch to one row per key
//! before any adapter SQL/Cypher runs (DRY + SRP).
//!
//! # Policy
//!
//! Last-write-wins on the full property map (matches Cypher `SET` / native
//! `DO UPDATE SET properties = EXCLUDED.properties`).

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

/// Collapse `(source, target, properties)` so each endpoint pair appears once
/// (last wins). Matches `idx_edge_source_target_unique`.
pub fn dedupe_edges_by_endpoints(
    edges: &[(String, String, HashMap<String, serde_json::Value>)],
) -> Vec<(String, String, HashMap<String, serde_json::Value>)> {
    let mut order: Vec<(String, String)> = Vec::new();
    let mut map: HashMap<(String, String), HashMap<String, serde_json::Value>> = HashMap::new();
    for (src, tgt, props) in edges {
        let key = (src.clone(), tgt.clone());
        if map.insert(key.clone(), props.clone()).is_none() {
            order.push(key);
        }
    }
    order
        .into_iter()
        .filter_map(|key| {
            let (src, tgt) = key.clone();
            map.remove(&key).map(|props| (src, tgt, props))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(label: &str) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("label".to_string(), serde_json::json!(label));
        m
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
        assert_eq!(out[0].2.get("label").and_then(|v| v.as_str()), Some("third"));
        assert_eq!(out[1].0, "C");
        assert_eq!(out[1].1, "D");
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
