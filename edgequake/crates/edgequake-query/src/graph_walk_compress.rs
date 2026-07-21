//! Graph-walk context compression (022 P1 / arXiv:2603.14045).
//!
//! Post-retrieval filter: keep a query-seeded subgraph of entities/relations
//! and chunks that mention kept entities (plus a naive-rank protect floor).
//! Zero LLM / embed cost. Env: `EDGEQUAKE_GRAPH_WALK_COMPRESS=1`.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::context::{QueryContext, RetrievedChunk, RetrievedEntity, RetrievedRelationship};

/// Env gate for graph-walk compression (default off).
pub fn graph_walk_compress_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_GRAPH_WALK_COMPRESS")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn max_hops() -> usize {
    std::env::var("EDGEQUAKE_GRAPH_WALK_COMPRESS_HOPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
        .clamp(1, 4)
}

fn naive_protect() -> usize {
    std::env::var("EDGEQUAKE_GRAPH_WALK_COMPRESS_NAIVE_PROTECT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8)
        .max(1)
}

fn seed_cap() -> usize {
    std::env::var("EDGEQUAKE_GRAPH_WALK_COMPRESS_SEED_CAP")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(12)
        .max(1)
}

fn norm(s: &str) -> String {
    s.trim().to_ascii_uppercase().replace(' ', "_")
}

/// Collect seed entity names from keywords ∩ retrieved entities, else top scores.
pub fn select_seed_entities(
    entities: &[RetrievedEntity],
    keywords: &[String],
    seed_cap: usize,
) -> Vec<String> {
    if entities.is_empty() {
        return Vec::new();
    }
    let kw: HashSet<String> = keywords.iter().map(|k| norm(k)).collect();
    let mut seeds: Vec<String> = Vec::new();
    if !kw.is_empty() {
        for e in entities {
            let n = norm(&e.name);
            if kw.iter().any(|k| n.contains(k) || k.contains(&n)) {
                seeds.push(e.name.clone());
            }
        }
    }
    if seeds.is_empty() {
        let mut ranked: Vec<&RetrievedEntity> = entities.iter().collect();
        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.degree.cmp(&a.degree))
        });
        for e in ranked.into_iter().take(seed_cap) {
            seeds.push(e.name.clone());
        }
    } else if seeds.len() > seed_cap {
        seeds.truncate(seed_cap);
    }
    seeds
}

/// BFS over retrieved relations from seeds; return kept entity name set.
pub fn expand_entity_neighborhood(
    seeds: &[String],
    relationships: &[RetrievedRelationship],
    hops: usize,
) -> HashSet<String> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for r in relationships {
        let a = r.source.clone();
        let b = r.target.clone();
        adj.entry(a.clone()).or_default().push(b.clone());
        adj.entry(b).or_default().push(a);
    }
    let mut kept: HashSet<String> = HashSet::new();
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    for s in seeds {
        kept.insert(s.clone());
        q.push_back((s.clone(), 0));
    }
    while let Some((node, depth)) = q.pop_front() {
        if depth >= hops {
            continue;
        }
        if let Some(neis) = adj.get(&node) {
            for n in neis {
                if kept.insert(n.clone()) {
                    q.push_back((n.clone(), depth + 1));
                }
            }
        }
    }
    kept
}

fn chunk_mentions_entity(chunk: &RetrievedChunk, entities: &HashSet<String>) -> bool {
    let content_u = chunk.content.to_ascii_uppercase();
    entities.iter().any(|e| {
        let n = norm(e);
        let spaced = n.replace('_', " ");
        content_u.contains(&n) || (!spaced.is_empty() && content_u.contains(&spaced))
    })
}

/// Apply graph-walk compression to a retrieved context.
pub fn apply_graph_walk_compress(
    mut context: QueryContext,
    keywords: &[String],
) -> QueryContext {
    if context.entities.is_empty() && context.relationships.is_empty() {
        return context;
    }
    let hops = max_hops();
    let protect = naive_protect();
    let cap = seed_cap();
    let seeds = select_seed_entities(&context.entities, keywords, cap);
    if seeds.is_empty() {
        return context;
    }
    let kept_names = expand_entity_neighborhood(&seeds, &context.relationships, hops);
    if kept_names.is_empty() {
        return context;
    }

    let before_e = context.entities.len();
    let before_r = context.relationships.len();
    let before_c = context.chunks.len();

    context.entities.retain(|e| kept_names.contains(&e.name));
    context
        .relationships
        .retain(|r| kept_names.contains(&r.source) && kept_names.contains(&r.target));

    // Protect top naive/global-ranked chunks by score so Fact Acc is not cliffed.
    let mut scored: Vec<(usize, f32)> = context
        .chunks
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.score))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut keep_idx: HashSet<usize> = scored.into_iter().take(protect).map(|(i, _)| i).collect();
    for (i, c) in context.chunks.iter().enumerate() {
        if chunk_mentions_entity(c, &kept_names) {
            keep_idx.insert(i);
        }
    }
    context.chunks = context
        .chunks
        .into_iter()
        .enumerate()
        .filter(|(i, _)| keep_idx.contains(i))
        .map(|(_, c)| c)
        .collect();

    context.metadata.insert(
        "graph_walk_compress".into(),
        serde_json::json!({
            "enabled": true,
            "hops": hops,
            "seeds": seeds.len(),
            "kept_entities": context.entities.len(),
            "before_entities": before_e,
            "before_relationships": before_r,
            "before_chunks": before_c,
            "after_chunks": context.chunks.len(),
        }),
    );
    tracing::debug!(
        seeds = seeds.len(),
        kept_entities = context.entities.len(),
        kept_rels = context.relationships.len(),
        kept_chunks = context.chunks.len(),
        "Applied graph-walk context compression"
    );
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{RetrievedChunk, RetrievedEntity, RetrievedRelationship};

    fn ent(name: &str, score: f32) -> RetrievedEntity {
        RetrievedEntity::new(name, "T", "d").with_score(score)
    }

    #[test]
    fn seeds_prefer_keyword_overlap() {
        let ents = vec![ent("ALPHA", 0.1), ent("BETA", 0.9), ent("GAMMA", 0.5)];
        let seeds = select_seed_entities(&ents, &["alpha".into()], 4);
        assert_eq!(seeds, vec!["ALPHA".to_string()]);
    }

    #[test]
    fn seeds_fallback_to_score() {
        let ents = vec![ent("A", 0.1), ent("B", 0.9)];
        let seeds = select_seed_entities(&ents, &[], 1);
        assert_eq!(seeds, vec!["B".to_string()]);
    }

    #[test]
    fn bfs_keeps_one_hop_neighbors() {
        let rels = vec![
            RetrievedRelationship::new("A", "B", "R"),
            RetrievedRelationship::new("B", "C", "R"),
            RetrievedRelationship::new("X", "Y", "R"),
        ];
        let kept = expand_entity_neighborhood(&["A".into()], &rels, 1);
        assert!(kept.contains("A"));
        assert!(kept.contains("B"));
        assert!(!kept.contains("C"));
        assert!(!kept.contains("X"));
    }

    #[test]
    fn compress_drops_unrelated_subgraph() {
        std::env::set_var("EDGEQUAKE_GRAPH_WALK_COMPRESS_NAIVE_PROTECT", "1");
        let mut ctx = QueryContext::new();
        ctx.add_entity(ent("A", 0.9));
        ctx.add_entity(ent("B", 0.8));
        ctx.add_entity(ent("X", 0.7));
        ctx.add_relationship(RetrievedRelationship::new("A", "B", "R"));
        ctx.add_relationship(RetrievedRelationship::new("X", "Y", "R"));
        ctx.add_chunk(RetrievedChunk::new("c1", "mentions A and B here", 0.5));
        ctx.add_chunk(RetrievedChunk::new("c2", "unrelated noise about Z", 0.4));
        ctx.add_chunk(RetrievedChunk::new("c3", "top score protect", 0.99));
        let out = apply_graph_walk_compress(ctx, &["A".into()]);
        assert!(out.entities.iter().any(|e| e.name == "A"));
        assert!(out.entities.iter().any(|e| e.name == "B"));
        assert!(!out.entities.iter().any(|e| e.name == "X"));
        assert!(out.chunks.iter().any(|c| c.id == "c1"));
        assert!(out.chunks.iter().any(|c| c.id == "c3")); // naive protect
        std::env::remove_var("EDGEQUAKE_GRAPH_WALK_COMPRESS_NAIVE_PROTECT");
    }
}
