//! Unified neighborhood expansion: BFS or Personalized PageRank (SPEC-046).
//!
//! Single entry point for local/global modes so walk strategy stays DRY.

use edgequake_storage::traits::{GraphEdge, GraphReadView};

use crate::graph_hops::edges_within_depth;
use crate::graph_ppr::{
    adjacency_from_edges, personalized_pagerank, rank_edges_by_ppr, GraphWalkMode, PprConfig,
};

/// Expand edges from seed entity IDs using the configured walk mode.
///
/// - **Bfs**: classic hop expansion (`edges_within_depth`).
/// - **Ppr**: fetch a generous BFS envelope, then re-rank edges by PPR mass
///   on that subgraph (HippoRAG-inspired; dual-node chunk mapping happens later).
pub async fn expand_neighborhood_edges(
    graph: &GraphReadView<'_>,
    seed_ids: &[String],
    depth: usize,
    max_edges: usize,
    walk: GraphWalkMode,
    tenant_id: Option<&str>,
    workspace_id: Option<&str>,
) -> edgequake_storage::error::Result<Vec<GraphEdge>> {
    if seed_ids.is_empty() || max_edges == 0 {
        return Ok(Vec::new());
    }

    match walk {
        GraphWalkMode::Bfs => {
            edges_within_depth(graph, seed_ids, depth, max_edges, tenant_id, workspace_id).await
        }
        GraphWalkMode::Ppr => {
            // Envelope: deeper / wider than final max so PPR has room to flow
            let envelope_depth = depth.max(2);
            let envelope_cap = max_edges.saturating_mul(4).max(max_edges).min(2_000);
            let envelope = edges_within_depth(
                graph,
                seed_ids,
                envelope_depth,
                envelope_cap,
                tenant_id,
                workspace_id,
            )
            .await?;
            if envelope.is_empty() {
                return Ok(Vec::new());
            }
            let adj = adjacency_from_edges(&envelope);
            let scores = personalized_pagerank(&adj, seed_ids, &PprConfig::from_env());
            Ok(rank_edges_by_ppr(&envelope, &scores, max_edges))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use edgequake_storage::adapters::memory::MemoryGraphStorage;
    use edgequake_storage::traits::{GraphReadView, GraphStorage, GraphStorageMutateOps};

    use super::*;

    #[tokio::test]
    async fn ppr_walk_returns_seed_adjacent_edges() {
        let graph = MemoryGraphStorage::new("ppr-expand");
        graph.initialize().await.unwrap();
        graph.upsert_edge("A", "B", HashMap::new()).await.unwrap();
        graph.upsert_edge("B", "C", HashMap::new()).await.unwrap();
        graph.upsert_edge("X", "Y", HashMap::new()).await.unwrap();

        let view = GraphReadView::new(&graph);
        let edges = expand_neighborhood_edges(
            &view,
            &["A".to_string()],
            2,
            10,
            GraphWalkMode::Ppr,
            None,
            None,
        )
        .await
        .unwrap();
        assert!(!edges.is_empty());
        assert!(
            edges.iter().any(|e| e.source == "A" || e.target == "A"),
            "PPR should retain seed-adjacent edges"
        );
    }
}
