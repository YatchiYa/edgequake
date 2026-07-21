//! LightRAG Local relation selection: incident edges → sort `(rank, weight)`.
//!
//! # Law (LightRAG `operate.py` `_find_most_related_edges_from_entities`)
//!
//! 1. Collect all undirected edges incident to retrieved entity nodes.
//! 2. `rank = node_degree(src) + node_degree(tgt)`.
//! 3. Sort by `(rank, weight)` descending.
//! 4. Truncate to `max_relationships` / token budget later.
//!
//! EdgeQuake default keeps PPR/BFS expansion (`EDGEQUAKE_GRAPH_WALK`). Opt in with
//! `EDGEQUAKE_RELATION_SELECT=lightrag` (051).

use std::collections::{HashMap, HashSet};

use edgequake_storage::traits::{GraphEdge, GraphReadView};

/// How Local/Global neighborhood edges are selected before prompt truncation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationSelectMode {
    /// Existing PPR/BFS via [`crate::graph_expand::expand_neighborhood_edges`].
    #[default]
    Default,
    /// LightRAG: seed-incident edges sorted by `(deg_src+deg_tgt, weight)`.
    LightRag,
}

impl RelationSelectMode {
    pub fn from_env() -> Self {
        parse_relation_select_mode(&std::env::var("EDGEQUAKE_RELATION_SELECT").unwrap_or_default())
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::LightRag => "lightrag",
        }
    }
}

/// Pure parser for `EDGEQUAKE_RELATION_SELECT`.
pub fn parse_relation_select_mode(raw: &str) -> RelationSelectMode {
    match raw.trim().to_ascii_lowercase().as_str() {
        "lightrag" | "lr" | "rank_weight" | "degree_weight" => RelationSelectMode::LightRag,
        _ => RelationSelectMode::Default,
    }
}

/// Edge weight from AGE properties (LightRAG default 1.0 when missing).
pub fn edge_weight(props: &HashMap<String, serde_json::Value>) -> f64 {
    props
        .get("weight")
        .and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_i64().map(|i| i as f64))
                .or_else(|| v.as_u64().map(|u| u as f64))
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .unwrap_or(1.0)
}

/// Sort edges by `(rank, weight)` descending (LightRAG reverse=True).
pub fn sort_edges_by_rank_weight(edges: &mut [GraphEdge], degrees: &HashMap<String, usize>) {
    edges.sort_by(|a, b| {
        let rank_a = degrees.get(&a.source).copied().unwrap_or(0)
            + degrees.get(&a.target).copied().unwrap_or(0);
        let rank_b = degrees.get(&b.source).copied().unwrap_or(0)
            + degrees.get(&b.target).copied().unwrap_or(0);
        rank_b.cmp(&rank_a).then_with(|| {
            edge_weight(&b.properties)
                .partial_cmp(&edge_weight(&a.properties))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
}

/// LightRAG Local edge select: all seed-incident edges → rank/weight sort → top-N.
pub async fn select_edges_lightrag(
    graph: &GraphReadView<'_>,
    seed_ids: &[String],
    max_edges: usize,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> edgequake_storage::error::Result<Vec<GraphEdge>> {
    if seed_ids.is_empty() || max_edges == 0 {
        return Ok(Vec::new());
    }

    let incident = graph
        .get_incident_edges_batch(seed_ids, tenant_id, workspace_id)
        .await?;

    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut edges = Vec::with_capacity(incident.len());
    for edge in incident {
        // Undirected dedupe (LightRAG sorted_edge).
        let key = if edge.source <= edge.target {
            (edge.source.clone(), edge.target.clone())
        } else {
            (edge.target.clone(), edge.source.clone())
        };
        if !seen.insert(key) {
            continue;
        }
        edges.push(edge);
    }

    if edges.is_empty() {
        return Ok(edges);
    }

    let mut endpoint_ids: Vec<String> = Vec::new();
    let mut seen_nodes: HashSet<String> = HashSet::new();
    for e in &edges {
        if seen_nodes.insert(e.source.clone()) {
            endpoint_ids.push(e.source.clone());
        }
        if seen_nodes.insert(e.target.clone()) {
            endpoint_ids.push(e.target.clone());
        }
    }

    let degrees: HashMap<String, usize> = graph
        .node_degrees_batch(&endpoint_ids)
        .await?
        .into_iter()
        .collect();

    sort_edges_by_rank_weight(&mut edges, &degrees);
    edges.truncate(max_edges);
    Ok(edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::adapters::memory::MemoryGraphStorage;
    use edgequake_storage::traits::{GraphReadView, GraphStorage, GraphStorageMutateOps};

    #[test]
    fn parse_lightrag_aliases() {
        assert_eq!(
            parse_relation_select_mode("lightrag"),
            RelationSelectMode::LightRag
        );
        assert_eq!(
            parse_relation_select_mode("rank_weight"),
            RelationSelectMode::LightRag
        );
        assert_eq!(parse_relation_select_mode(""), RelationSelectMode::Default);
    }

    #[test]
    fn sort_prefers_higher_rank_then_weight() {
        let mut degrees = HashMap::new();
        degrees.insert("A".into(), 10);
        degrees.insert("B".into(), 1);
        degrees.insert("C".into(), 1);
        degrees.insert("H".into(), 50);

        let mut edges = vec![
            GraphEdge::with_properties(
                "A",
                "B",
                HashMap::from([("weight".into(), serde_json::json!(9.0))]),
            ),
            GraphEdge::with_properties(
                "H",
                "C",
                HashMap::from([("weight".into(), serde_json::json!(0.1))]),
            ),
            GraphEdge::with_properties(
                "A",
                "C",
                HashMap::from([("weight".into(), serde_json::json!(1.0))]),
            ),
        ];
        // ranks: A-B=11, H-C=51, A-C=11 → H-C first; A-B before A-C by weight
        sort_edges_by_rank_weight(&mut edges, &degrees);
        assert_eq!(edges[0].source, "H");
        assert_eq!(edges[1].source, "A");
        assert_eq!(edges[1].target, "B");
        assert_eq!(edges[2].target, "C");
    }

    #[tokio::test]
    async fn select_edges_lightrag_returns_highest_rank_first() {
        let graph = MemoryGraphStorage::new("rel-select-lr");
        graph.initialize().await.unwrap();
        // Hub H connected to many; leaf L only to seed S
        for i in 0..5 {
            graph
                .upsert_edge("H", &format!("N{i}"), HashMap::new())
                .await
                .unwrap();
        }
        graph
            .upsert_edge(
                "S",
                "H",
                HashMap::from([("weight".into(), serde_json::json!(1.0))]),
            )
            .await
            .unwrap();
        graph
            .upsert_edge(
                "S",
                "L",
                HashMap::from([("weight".into(), serde_json::json!(9.0))]),
            )
            .await
            .unwrap();

        let view = GraphReadView::new(&graph);
        let edges = select_edges_lightrag(&view, &["S".into()], 1, None, None)
            .await
            .unwrap();
        assert_eq!(edges.len(), 1);
        // S-H rank ≫ S-L even though L has higher weight
        assert!(
            edges[0].source == "S" && edges[0].target == "H"
                || edges[0].source == "H" && edges[0].target == "S",
            "expected S-H first, got {}->{}",
            edges[0].source,
            edges[0].target
        );
    }
}
