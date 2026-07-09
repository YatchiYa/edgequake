//! PathRAG-inspired relationship pruning (SPEC-046 P1.3).
//!
//! Drops low-flow relations before token budgeting to cut context tax while
//! keeping high-degree / high-score paths.

use crate::context::RetrievedRelationship;

/// Configuration for path pruning.
#[derive(Debug, Clone, Copy)]
pub struct PathPruneConfig {
    /// Fraction of lowest-scoring relations to drop (0.0–0.9). Default 0.4.
    pub drop_fraction: f32,
    /// Always keep at least this many relations (when available).
    pub min_keep: usize,
    /// Soft-disable when relation count is below this (avoid over-pruning tiny graphs).
    pub min_input: usize,
}

impl Default for PathPruneConfig {
    fn default() -> Self {
        Self {
            drop_fraction: 0.4,
            min_keep: 3,
            min_input: 5,
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
        cfg
    }

    pub fn enabled(&self) -> bool {
        self.drop_fraction > 0.0
    }
}

/// Score a relationship for retention (higher = keep).
///
/// Combines retrieval score with a description-length prior (richer edges tend
/// to carry more evidence) and a mild keyword bonus when description is non-empty.
fn flow_score(rel: &RetrievedRelationship) -> f32 {
    let base = rel.score.max(0.0);
    let desc_bonus = (rel.description.len() as f32).ln_1p() * 0.01;
    let type_bonus = if rel.relation_type.is_empty() {
        0.0
    } else {
        0.05
    };
    base + desc_bonus + type_bonus
}

/// Prune lowest-flow relationships. Preserves input order among survivors
/// relative to original score ranking (re-sorts by flow desc then truncates).
pub fn prune_relationships(
    relationships: Vec<RetrievedRelationship>,
    config: &PathPruneConfig,
) -> Vec<RetrievedRelationship> {
    if !config.enabled() || relationships.len() < config.min_input {
        return relationships;
    }

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
            let s = flow_score(&r);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RetrievedRelationship;

    fn rel(src: &str, tgt: &str, score: f32, desc: &str) -> RetrievedRelationship {
        RetrievedRelationship::new(src, tgt, "RELATED")
            .with_description(desc)
            .with_score(score)
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
}
