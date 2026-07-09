//! Personalized PageRank over entity subgraphs (SPEC-046 P1.1 / HippoRAG-inspired).
//!
//! Pure in-process PPR on an adjacency list built from AGE/memory graph edges.
//! Dual-node lite: after ranking entities, callers map scores onto passage/chunk
//! IDs via `source_chunk_ids` (no separate passage graph required).
//!
//! Config: `EDGEQUAKE_GRAPH_WALK=bfs|ppr` (default `bfs` until eval gates pass).

use std::collections::{HashMap, HashSet};

use edgequake_storage::traits::GraphEdge;

/// How local/global modes expand the entity neighborhood.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GraphWalkMode {
    /// Classic BFS hop expansion (`edges_within_depth`).
    #[default]
    Bfs,
    /// Personalized PageRank on the fetched subgraph (HippoRAG-style).
    Ppr,
}

impl GraphWalkMode {
    /// Read from `EDGEQUAKE_GRAPH_WALK` (`bfs` | `ppr`). Default: Bfs.
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_GRAPH_WALK")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "ppr" | "pagerank" | "personalized_pagerank" => Self::Ppr,
            _ => Self::Bfs,
        }
    }
}

/// Tunables for power-iteration PPR (HippoRAG default damping = 0.5).
#[derive(Debug, Clone, Copy)]
pub struct PprConfig {
    /// Teleport / restart probability toward seeds (HippoRAG: 0.5).
    pub damping: f32,
    /// Max power-iteration steps.
    pub max_iterations: usize,
    /// L1 convergence threshold.
    pub tolerance: f32,
}

impl Default for PprConfig {
    fn default() -> Self {
        Self {
            damping: 0.5,
            max_iterations: 40,
            tolerance: 1e-6,
        }
    }
}

impl PprConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("EDGEQUAKE_PPR_DAMPING") {
            if let Ok(d) = v.parse::<f32>() {
                cfg.damping = d.clamp(0.05, 0.95);
            }
        }
        if let Ok(v) = std::env::var("EDGEQUAKE_PPR_MAX_ITERS") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_iterations = n.clamp(5, 200);
            }
        }
        cfg
    }
}

/// Build an undirected adjacency list from graph edges (entity–entity).
pub fn adjacency_from_edges(edges: &[GraphEdge]) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for e in edges {
        adj.entry(e.source.clone())
            .or_default()
            .push(e.target.clone());
        adj.entry(e.target.clone())
            .or_default()
            .push(e.source.clone());
    }
    // Deduplicate neighbors
    for neighbors in adj.values_mut() {
        neighbors.sort();
        neighbors.dedup();
    }
    adj
}

/// Run Personalized PageRank with equal mass on `seed_ids`.
///
/// Returns scores for all nodes that appear in `adjacency` or seeds.
/// Empty seeds → empty map. Isolated seeds keep their teleport mass.
pub fn personalized_pagerank(
    adjacency: &HashMap<String, Vec<String>>,
    seed_ids: &[String],
    config: &PprConfig,
) -> HashMap<String, f32> {
    if seed_ids.is_empty() {
        return HashMap::new();
    }

    let mut nodes: HashSet<String> = adjacency.keys().cloned().collect();
    for s in seed_ids {
        nodes.insert(s.clone());
    }
    if nodes.is_empty() {
        return HashMap::new();
    }

    let node_list: Vec<String> = nodes.into_iter().collect();
    let n = node_list.len();
    let index: HashMap<&str, usize> = node_list
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    // Personalization vector: uniform over seeds present in the node set
    let mut personal = vec![0.0f32; n];
    let mut seed_count = 0usize;
    for s in seed_ids {
        if let Some(&i) = index.get(s.as_str()) {
            personal[i] = 1.0;
            seed_count += 1;
        }
    }
    if seed_count == 0 {
        // Seeds not in subgraph — inject them as isolated nodes with full mass
        return seed_ids
            .iter()
            .map(|s| (s.clone(), 1.0 / seed_ids.len() as f32))
            .collect();
    }
    let inv = 1.0 / seed_count as f32;
    for p in &mut personal {
        *p *= inv;
    }

    let mut rank = personal.clone();
    let alpha = config.damping;

    for _ in 0..config.max_iterations {
        let mut next = vec![0.0f32; n];
        // Teleport
        for i in 0..n {
            next[i] = alpha * personal[i];
        }
        // Walk
        for (i, id) in node_list.iter().enumerate() {
            let neighbors = adjacency.get(id).map(|v| v.as_slice()).unwrap_or(&[]);
            if neighbors.is_empty() {
                // Dangling: redistribute mass via teleport (already in next via personal)
                // Also push remaining walk mass back to personalization
                for j in 0..n {
                    next[j] += (1.0 - alpha) * rank[i] * personal[j];
                }
                continue;
            }
            let share = (1.0 - alpha) * rank[i] / neighbors.len() as f32;
            for nb in neighbors {
                if let Some(&j) = index.get(nb.as_str()) {
                    next[j] += share;
                }
            }
        }

        let mut diff = 0.0f32;
        for i in 0..n {
            diff += (next[i] - rank[i]).abs();
        }
        rank = next;
        if diff < config.tolerance {
            break;
        }
    }

    node_list
        .into_iter()
        .zip(rank)
        .collect()
}

/// Rank seed-neighborhood edges by the sum of endpoint PPR scores (desc).
pub fn rank_edges_by_ppr(
    edges: &[GraphEdge],
    scores: &HashMap<String, f32>,
    max_edges: usize,
) -> Vec<GraphEdge> {
    if max_edges == 0 || edges.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(f32, &GraphEdge)> = edges
        .iter()
        .map(|e| {
            let s = scores.get(&e.source).copied().unwrap_or(0.0)
                + scores.get(&e.target).copied().unwrap_or(0.0);
            (s, e)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
        .into_iter()
        .take(max_edges)
        .map(|(_, e)| e.clone())
        .collect()
}

/// Aggregate entity PPR mass onto chunk IDs (dual-node lite).
pub fn chunk_scores_from_entity_ppr(
    entity_to_chunks: &HashMap<String, Vec<String>>,
    entity_scores: &HashMap<String, f32>,
) -> HashMap<String, f32> {
    let mut chunk_scores: HashMap<String, f32> = HashMap::new();
    for (entity, score) in entity_scores {
        if let Some(chunks) = entity_to_chunks.get(entity) {
            if chunks.is_empty() {
                continue;
            }
            let share = score / chunks.len() as f32;
            for c in chunks {
                *chunk_scores.entry(c.clone()).or_insert(0.0) += share;
            }
        }
    }
    chunk_scores
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::traits::GraphEdge;
    use std::collections::HashMap;

    fn edge(a: &str, b: &str) -> GraphEdge {
        GraphEdge {
            source: a.to_string(),
            target: b.to_string(),
            properties: HashMap::new(),
        }
    }

    #[test]
    fn ppr_concentrates_on_seed_neighborhood() {
        let edges = vec![edge("A", "B"), edge("B", "C"), edge("X", "Y")];
        let adj = adjacency_from_edges(&edges);
        let scores = personalized_pagerank(&adj, &["A".to_string()], &PprConfig::default());
        assert!(scores.get("A").copied().unwrap_or(0.0) > 0.0);
        assert!(
            scores.get("B").copied().unwrap_or(0.0) >= scores.get("Y").copied().unwrap_or(0.0),
            "seed neighborhood should outrank disconnected component"
        );
    }

    #[test]
    fn ppr_empty_seeds() {
        let adj = adjacency_from_edges(&[edge("A", "B")]);
        let scores = personalized_pagerank(&adj, &[], &PprConfig::default());
        assert!(scores.is_empty());
    }

    #[test]
    fn rank_edges_prefers_high_ppr_endpoints() {
        let edges = vec![edge("A", "B"), edge("X", "Y")];
        let mut scores = HashMap::new();
        scores.insert("A".to_string(), 0.5);
        scores.insert("B".to_string(), 0.4);
        scores.insert("X".to_string(), 0.01);
        scores.insert("Y".to_string(), 0.01);
        let ranked = rank_edges_by_ppr(&edges, &scores, 1);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].source, "A");
    }

    #[test]
    fn dual_node_chunk_aggregation() {
        let mut entity_chunks = HashMap::new();
        entity_chunks.insert("A".to_string(), vec!["c1".to_string(), "c2".to_string()]);
        let mut scores = HashMap::new();
        scores.insert("A".to_string(), 1.0);
        let chunk_scores = chunk_scores_from_entity_ppr(&entity_chunks, &scores);
        assert!((chunk_scores["c1"] - 0.5).abs() < 1e-5);
        assert!((chunk_scores["c2"] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn graph_walk_mode_env_default_bfs() {
        // Do not assert env mutation in parallel tests; just parse known strings.
        assert_eq!(GraphWalkMode::default(), GraphWalkMode::Bfs);
    }
}
