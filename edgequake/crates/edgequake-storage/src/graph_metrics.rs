//! Graph quality metrics for ingest observability (SPEC-046 P0.2).
//!
//! GraphRAG-Bench (ICLR 2026): average degree and clustering mediate retrieval
//! power. We expose cheap structural metrics after merge so ops can alert on
//! sparse / empty graphs without blocking the persist hot path.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::traits::{GraphEdge, GraphNode, GraphReadView, GraphStorage};

/// Snapshot of knowledge-graph structural quality.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GraphQualityMetrics {
    /// Node count `|V|`.
    pub node_count: usize,
    /// Edge count `|E|`.
    pub edge_count: usize,
    /// `2|E| / |V|` (0 when empty).
    pub avg_degree: f64,
    /// Fraction of nodes with degree 0.
    pub orphan_rate: f64,
    /// Fraction of nodes with empty / missing description property.
    pub empty_description_rate: f64,
    /// Optional workspace scope (string form of UUID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl GraphQualityMetrics {
    /// True when the graph looks too sparse for useful multi-hop retrieval.
    ///
    /// Threshold mirrors SPEC-046 ops guidance (alert if avg_degree < 2.0 on
    /// non-trivial graphs).
    pub fn is_sparse(&self) -> bool {
        self.node_count >= 10 && self.avg_degree < 2.0
    }

    /// Compute metrics from in-memory node/edge lists (unit-test / offline).
    pub fn from_nodes_and_edges(nodes: &[GraphNode], edges: &[GraphEdge]) -> Self {
        let node_count = nodes.len();
        let edge_count = edges.len();
        let avg_degree = if node_count == 0 {
            0.0
        } else {
            (2.0 * edge_count as f64) / node_count as f64
        };

        let mut degree: HashMap<&str, usize> = HashMap::new();
        for n in nodes {
            degree.insert(n.id.as_str(), 0);
        }
        for e in edges {
            *degree.entry(e.source.as_str()).or_insert(0) += 1;
            *degree.entry(e.target.as_str()).or_insert(0) += 1;
        }

        let orphan_count = degree.values().filter(|&&d| d == 0).count();
        let orphan_rate = if node_count == 0 {
            0.0
        } else {
            orphan_count as f64 / node_count as f64
        };

        let empty_desc = nodes
            .iter()
            .filter(|n| {
                n.properties
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
            })
            .count();
        let empty_description_rate = if node_count == 0 {
            0.0
        } else {
            empty_desc as f64 / node_count as f64
        };

        Self {
            node_count,
            edge_count,
            avg_degree,
            orphan_rate,
            empty_description_rate,
            workspace_id: None,
        }
    }
}

/// Collect metrics for a workspace (or entire graph when `workspace_id` is None).
///
/// Uses fast counts when available; samples node properties for description /
/// orphan rates via popular-node scan capped at `sample_limit`.
pub async fn collect_graph_quality_metrics(
    graph: &dyn GraphStorage,
    workspace_id: Option<&uuid::Uuid>,
    sample_limit: usize,
) -> Result<GraphQualityMetrics> {
    let view = GraphReadView::new(graph);
    let (node_count, edge_count) = if let Some(ws) = workspace_id {
        (
            view.node_count_by_workspace(ws).await?,
            view.edge_count_by_workspace(ws).await?,
        )
    } else {
        (view.node_count_fast().await?, view.edge_count_fast().await?)
    };

    let avg_degree = if node_count == 0 {
        0.0
    } else {
        (2.0 * edge_count as f64) / node_count as f64
    };

    let sample_cap = sample_limit.clamp(1, 5_000);
    let popular = view
        .get_popular_nodes_with_degree(
            sample_cap,
            None,
            None,
            None,
            workspace_id.map(|u| u.to_string()).as_deref(),
        )
        .await
        .unwrap_or_default();

    let sample_n = popular.len().max(1);
    let orphan_in_sample = popular.iter().filter(|(_, d)| *d == 0).count();
    // Popular scan biases toward high degree — orphan_rate from this sample is
    // a lower bound. Prefer degree==0 count when we have full sample coverage.
    let orphan_rate = if node_count == 0 {
        0.0
    } else if popular.len() >= node_count {
        orphan_in_sample as f64 / node_count as f64
    } else {
        // Extrapolate cautiously: use sample orphan fraction * (1 - coverage)
        // only as soft signal; prefer 0 when sample has no orphans.
        orphan_in_sample as f64 / sample_n as f64
    };

    let empty_desc = popular
        .iter()
        .filter(|(n, _)| {
            n.properties
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
        })
        .count();
    let empty_description_rate = if popular.is_empty() {
        0.0
    } else {
        empty_desc as f64 / popular.len() as f64
    };

    Ok(GraphQualityMetrics {
        node_count,
        edge_count,
        avg_degree,
        orphan_rate,
        empty_description_rate,
        workspace_id: workspace_id.map(|u| u.to_string()),
    })
}

/// Emit structured tracing for ops dashboards; never fails the caller.
pub fn log_graph_quality(metrics: &GraphQualityMetrics, document_id: Option<&str>) {
    if metrics.is_sparse() {
        tracing::warn!(
            node_count = metrics.node_count,
            edge_count = metrics.edge_count,
            avg_degree = metrics.avg_degree,
            orphan_rate = metrics.orphan_rate,
            empty_description_rate = metrics.empty_description_rate,
            workspace_id = ?metrics.workspace_id,
            document_id,
            "SPEC-046: graph quality sparse (avg_degree < 2.0)"
        );
    } else {
        tracing::info!(
            node_count = metrics.node_count,
            edge_count = metrics.edge_count,
            avg_degree = metrics.avg_degree,
            orphan_rate = metrics.orphan_rate,
            empty_description_rate = metrics.empty_description_rate,
            workspace_id = ?metrics.workspace_id,
            document_id,
            "SPEC-046: graph quality metrics"
        );
    }
}

/// Quick metrics from merge artifacts alone (no extra graph round-trip).
///
/// Useful immediately after merge when full graph scan is expensive; treats
/// created nodes/edges as the session delta (not global totals).
pub fn metrics_from_merge_delta(
    nodes_created: &[String],
    edges_created: &[(String, String)],
) -> GraphQualityMetrics {
    let node_set: HashSet<&str> = nodes_created.iter().map(|s| s.as_str()).collect();
    let mut all_nodes: HashSet<String> = nodes_created.iter().cloned().collect();
    for (s, t) in edges_created {
        all_nodes.insert(s.clone());
        all_nodes.insert(t.clone());
    }
    let node_count = all_nodes.len();
    let edge_count = edges_created.len();
    let avg_degree = if node_count == 0 {
        0.0
    } else {
        (2.0 * edge_count as f64) / node_count as f64
    };
    let touched: HashSet<&str> = edges_created
        .iter()
        .flat_map(|(s, t)| [s.as_str(), t.as_str()])
        .collect();
    let orphan_count = node_set.iter().filter(|n| !touched.contains(*n)).count();
    let orphan_rate = if node_count == 0 {
        0.0
    } else {
        orphan_count as f64 / node_count as f64
    };

    GraphQualityMetrics {
        node_count,
        edge_count,
        avg_degree,
        orphan_rate,
        empty_description_rate: 0.0,
        workspace_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::GraphNode;
    use std::collections::HashMap;

    #[test]
    fn avg_degree_on_triangle() {
        let nodes: Vec<_> = ["A", "B", "C"]
            .iter()
            .map(|id| {
                GraphNode::with_properties(
                    (*id).to_string(),
                    HashMap::from([("description".into(), serde_json::json!("x"))]),
                )
            })
            .collect();
        let edges = vec![
            GraphEdge::new("A", "B"),
            GraphEdge::new("B", "C"),
            GraphEdge::new("C", "A"),
        ];
        let m = GraphQualityMetrics::from_nodes_and_edges(&nodes, &edges);
        assert_eq!(m.node_count, 3);
        assert_eq!(m.edge_count, 3);
        assert!((m.avg_degree - 2.0).abs() < 1e-9);
        assert!((m.orphan_rate - 0.0).abs() < 1e-9);
        assert!(!m.is_sparse()); // only 3 nodes
    }

    #[test]
    fn sparse_alert_requires_min_nodes() {
        let mut m = GraphQualityMetrics {
            node_count: 20,
            edge_count: 5,
            avg_degree: 0.5,
            ..Default::default()
        };
        assert!(m.is_sparse());
        m.node_count = 5;
        assert!(!m.is_sparse());
    }

    #[test]
    fn merge_delta_orphans() {
        let nodes = vec!["A".into(), "B".into(), "LONER".into()];
        let edges = vec![("A".into(), "B".into())];
        let m = metrics_from_merge_delta(&nodes, &edges);
        assert_eq!(m.node_count, 3);
        assert_eq!(m.edge_count, 1);
        assert!((m.orphan_rate - (1.0 / 3.0)).abs() < 1e-9);
    }
}
