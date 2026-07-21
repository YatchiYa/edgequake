//! PathRAG-inspired relationship pruning (SPEC-046 P1.3 / SPEC-001 Phase 1).
//!
//! Drops low-flow relations before token budgeting to cut context tax while
//! keeping high-degree / high-score / query-aligned paths.
//!
//! Optional orphan-entity prune keeps only entities that survive on kept paths
//! (plus a small high-score seed floor), matching PathRAG's "redundancy not
//! insufficiency" thesis ([arXiv:2502.14902](https://arxiv.org/abs/2502.14902)).

use std::collections::HashSet;

use crate::context::{RetrievedEntity, RetrievedRelationship};

/// Configuration for path pruning.
#[derive(Debug, Clone, Copy)]
pub struct PathPruneConfig {
    /// Fraction of lowest-scoring relations to drop (0.0–0.9). Default 0.4.
    pub drop_fraction: f32,
    /// Always keep at least this many relations (when available).
    pub min_keep: usize,
    /// Soft-disable when relation count is below this (avoid over-pruning tiny graphs).
    pub min_input: usize,
    /// Drop entities that do not appear on any kept relationship.
    pub prune_orphan_entities: bool,
    /// Always keep at least this many entities after orphan prune.
    pub entity_min_keep: usize,
}

impl Default for PathPruneConfig {
    fn default() -> Self {
        Self {
            drop_fraction: 0.4,
            min_keep: 3,
            min_input: 5,
            prune_orphan_entities: false,
            entity_min_keep: 4,
        }
    }
}

impl PathPruneConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("EDGEQUAKE_PATH_PRUNE_FRACTION") {
            if let Ok(f) = v.parse::<f32>() {
                cfg.drop_fraction = f.clamp(0.0, 0.9);
            }
        }
        if matches!(
            std::env::var("EDGEQUAKE_PATH_PRUNE")
                .unwrap_or_default()
                .to_ascii_lowercase()
                .as_str(),
            "false" | "0" | "off"
        ) {
            cfg.drop_fraction = 0.0;
        }
        cfg.prune_orphan_entities = env_truthy("EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES");
        if let Ok(v) = std::env::var("EDGEQUAKE_PATH_PRUNE_ENTITY_MIN_KEEP") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.entity_min_keep = n.max(1);
            }
        }
        cfg
    }

    pub fn enabled(&self) -> bool {
        self.drop_fraction > 0.0
    }
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Tokenize query into lowercase alphanumeric tokens (len ≥ 3).
fn query_tokens(query: &str) -> HashSet<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Score a relationship for retention (higher = keep).
///
/// Combines retrieval score with a description-length prior, a mild type bonus,
/// and optional query-token overlap on endpoints / description / type.
fn flow_score(rel: &RetrievedRelationship, query_toks: &HashSet<String>) -> f32 {
    let base = rel.score.max(0.0);
    let desc_bonus = (rel.description.len() as f32).ln_1p() * 0.01;
    let type_bonus = if rel.relation_type.is_empty() {
        0.0
    } else {
        0.05
    };
    let q_bonus = if query_toks.is_empty() {
        0.0
    } else {
        let hay = format!(
            "{} {} {} {}",
            rel.source, rel.target, rel.relation_type, rel.description
        )
        .to_ascii_lowercase();
        let hits = query_toks
            .iter()
            .filter(|t| hay.contains(t.as_str()))
            .count();
        // Strong enough to outrank raw retrieval score on off-query hubs.
        (hits as f32) * 0.35
    };
    base + desc_bonus + type_bonus + q_bonus
}

/// Prune lowest-flow relationships (no query conditioning).
pub fn prune_relationships(
    relationships: Vec<RetrievedRelationship>,
    config: &PathPruneConfig,
) -> Vec<RetrievedRelationship> {
    prune_relationships_for_query(relationships, config, "")
}

/// PathRAG-style prune with query-conditioned flow scores.
pub fn prune_relationships_for_query(
    relationships: Vec<RetrievedRelationship>,
    config: &PathPruneConfig,
    query: &str,
) -> Vec<RetrievedRelationship> {
    if !config.enabled() || relationships.len() < config.min_input {
        return relationships;
    }

    let q_toks = query_tokens(query);
    let n = relationships.len();
    let drop_n = ((n as f32) * config.drop_fraction).floor() as usize;
    let keep = (n - drop_n).max(config.min_keep).min(n);
    if keep >= n {
        return relationships;
    }

    let mut indexed: Vec<(usize, f32, RetrievedRelationship)> = relationships
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let s = flow_score(&r, &q_toks);
            (i, s, r)
        })
        .collect();

    indexed.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    indexed.truncate(keep);
    // Stable-ish: restore original relative order among kept items
    indexed.sort_by_key(|(i, _, _)| *i);
    indexed.into_iter().map(|(_, _, r)| r).collect()
}

/// Drop entities that do not appear on any kept relationship.
///
/// Always retains up to `entity_min_keep` highest-score entities as seeds so
/// sparse graphs do not collapse to empty entity context.
pub fn prune_orphan_entities(
    entities: Vec<RetrievedEntity>,
    relationships: &[RetrievedRelationship],
    config: &PathPruneConfig,
) -> Vec<RetrievedEntity> {
    if !config.prune_orphan_entities || entities.is_empty() {
        return entities;
    }

    let mut on_path: HashSet<String> = HashSet::new();
    for r in relationships {
        on_path.insert(r.source.to_ascii_uppercase());
        on_path.insert(r.target.to_ascii_uppercase());
    }

    if on_path.is_empty() {
        // No paths left — keep top seeds only.
        let mut ranked = entities;
        ranked.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.degree.cmp(&a.degree))
        });
        ranked.truncate(config.entity_min_keep.min(ranked.len()));
        return ranked;
    }

    let mut kept: Vec<RetrievedEntity> = entities
        .iter()
        .filter(|e| on_path.contains(&e.name.to_ascii_uppercase()))
        .cloned()
        .collect();

    if kept.len() >= config.entity_min_keep {
        return kept;
    }

    // Seed floor: add highest-score entities not yet kept.
    let mut ranked = entities;
    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.degree.cmp(&a.degree))
    });
    let mut seen: HashSet<String> = kept
        .iter()
        .map(|e| e.name.to_ascii_uppercase())
        .collect();
    for e in ranked {
        if kept.len() >= config.entity_min_keep {
            break;
        }
        let key = e.name.to_ascii_uppercase();
        if seen.insert(key) {
            kept.push(e);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{RetrievedEntity, RetrievedRelationship};

    fn rel(src: &str, tgt: &str, score: f32, desc: &str) -> RetrievedRelationship {
        RetrievedRelationship::new(src, tgt, "RELATED")
            .with_description(desc)
            .with_score(score)
    }

    fn ent(name: &str, score: f32) -> RetrievedEntity {
        RetrievedEntity::new(name, "CONCEPT", "desc").with_score(score)
    }

    #[test]
    fn prune_drops_bottom_fraction() {
        let rels: Vec<_> = (0..10)
            .map(|i| rel("A", &format!("T{i}"), i as f32 * 0.1, "desc"))
            .collect();
        let cfg = PathPruneConfig {
            drop_fraction: 0.4,
            min_keep: 3,
            min_input: 5,
            ..Default::default()
        };
        let kept = prune_relationships(rels, &cfg);
        assert_eq!(kept.len(), 6);
        // Highest scores should survive
        assert!(kept.iter().any(|r| (r.score - 0.9).abs() < 1e-5));
        assert!(!kept.iter().any(|r| r.score < 0.35));
    }

    #[test]
    fn prune_skips_small_inputs() {
        let rels = vec![rel("A", "B", 1.0, "x"), rel("A", "C", 0.1, "y")];
        let cfg = PathPruneConfig::default();
        let kept = prune_relationships(rels.clone(), &cfg);
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn prune_disabled_when_fraction_zero() {
        let rels: Vec<_> = (0..10)
            .map(|i| rel("A", &format!("T{i}"), i as f32, ""))
            .collect();
        let cfg = PathPruneConfig {
            drop_fraction: 0.0,
            ..Default::default()
        };
        assert_eq!(prune_relationships(rels.clone(), &cfg).len(), 10);
    }

    #[test]
    fn query_conditioned_prefers_endpoint_overlap() {
        // Off-query edges start with higher raw scores; query bonus must flip rank.
        let rels = vec![
            rel("INSULIN", "GLUCOSE", 0.25, "regulates blood sugar"),
            rel("CAR", "WHEEL", 0.55, "mechanical part"),
            rel("DIABETES", "PANCREAS", 0.30, "metabolic disease"),
            rel("TREE", "LEAF", 0.50, "botany"),
            rel("METFORMIN", "LIVER", 0.28, "diabetes treatment drug"),
            rel("BOOK", "PAGE", 0.48, "reading"),
        ];
        let cfg = PathPruneConfig {
            drop_fraction: 0.5,
            min_keep: 2,
            min_input: 4,
            ..Default::default()
        };
        let kept = prune_relationships_for_query(rels, &cfg, "diabetes insulin treatment");
        assert_eq!(kept.len(), 3);
        let names: String = kept
            .iter()
            .flat_map(|r| [r.source.as_str(), r.target.as_str()])
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            names.contains("DIABETES")
                && names.contains("INSULIN")
                && names.contains("METFORMIN"),
            "expected medical endpoints kept, got {names}"
        );
        assert!(
            !names.contains("CAR") && !names.contains("TREE") && !names.contains("BOOK"),
            "off-query hubs should be dropped, got {names}"
        );
    }

    #[test]
    fn orphan_entity_prune_keeps_path_nodes() {
        let ents = vec![
            ent("INSULIN", 0.5),
            ent("GLUCOSE", 0.4),
            ent("ORPHAN_HUB", 0.99),
            ent("SEED", 0.1),
        ];
        let rels = vec![rel("INSULIN", "GLUCOSE", 1.0, "link")];
        let cfg = PathPruneConfig {
            prune_orphan_entities: true,
            entity_min_keep: 2,
            ..Default::default()
        };
        let kept = prune_orphan_entities(ents, &rels, &cfg);
        let names: HashSet<_> = kept.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains("INSULIN"));
        assert!(names.contains("GLUCOSE"));
        assert!(!names.contains("ORPHAN_HUB"));
    }

    #[test]
    fn orphan_entity_seed_floor() {
        let ents = vec![ent("A", 0.9), ent("B", 0.8), ent("C", 0.1)];
        let rels: Vec<RetrievedRelationship> = vec![];
        let cfg = PathPruneConfig {
            prune_orphan_entities: true,
            entity_min_keep: 2,
            ..Default::default()
        };
        let kept = prune_orphan_entities(ents, &rels, &cfg);
        assert_eq!(kept.len(), 2);
        assert!((kept[0].score - 0.9).abs() < 1e-5);
    }
}
