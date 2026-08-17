//! Prompt entity ordering for Mix / KG context.
//!
//! LightRAG keeps VDB / cosine order. EdgeQuake historically sorted by graph
//! degree (hubs first), which can bury query-relevant entities for Complex
//! Reasoning. Acc-win E2: `EDGEQUAKE_ENTITY_RANK=query_score`.
//! 022 P2: `EDGEQUAKE_ENTITY_RANK=retrieval` preserves arm merge order (no resort).

use crate::context::RetrievedEntity;

/// How to order entities before prompt formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityRankMode {
<<<<<<< HEAD
    /// Degree descending (historical EdgeQuake default).
    #[default]
=======
    /// Degree descending (historical EdgeQuake; labeled ablation).
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    Degree,
    /// Retrieval / VDB score descending; degree as tie-break.
    QueryScore,
    /// Preserve current list order (LightRAG VDB / merge order parity).
<<<<<<< HEAD
=======
    /// Product / Acc E2-occ default (SPEC-086).
    #[default]
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    Retrieval,
}

impl EntityRankMode {
    pub fn from_env() -> Self {
        match std::env::var("EDGEQUAKE_ENTITY_RANK")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "query_score" | "score" | "cosine" | "vdb" => Self::QueryScore,
<<<<<<< HEAD
            "retrieval" | "preserve" | "none" | "as_is" => Self::Retrieval,
            _ => Self::Degree,
=======
            "degree" | "hub" | "hubs" => Self::Degree,
            // Empty / LightRAG aliases → product default (E2-occ).
            "retrieval" | "preserve" | "none" | "as_is" | "" => Self::Retrieval,
            _ => Self::Retrieval,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Degree => "degree",
            Self::QueryScore => "query_score",
            Self::Retrieval => "retrieval",
        }
    }
}

/// Rank entities for prompt emission.
///
/// - [`EntityRankMode::Degree`]: higher graph degree first.
/// - [`EntityRankMode::QueryScore`]: higher `score` first, then degree.
/// - [`EntityRankMode::Retrieval`]: leave order unchanged.
pub fn rank_entities_for_prompt(entities: &mut [RetrievedEntity], mode: EntityRankMode) {
    match mode {
        EntityRankMode::Degree => {
            entities.sort_by_key(|b| std::cmp::Reverse(b.degree));
        }
        EntityRankMode::QueryScore => {
            entities.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.degree.cmp(&a.degree))
            });
        }
        EntityRankMode::Retrieval => {
            // LightRAG-parity: keep fusion / VDB order.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ent(name: &str, score: f32, degree: usize) -> RetrievedEntity {
        RetrievedEntity::new(name, "T", "d")
            .with_score(score)
            .with_degree(degree)
    }

    #[test]
    fn degree_mode_puts_hub_first() {
        let mut ents = vec![ent("low_deg_high_score", 0.9, 1), ent("hub", 0.1, 50)];
        rank_entities_for_prompt(&mut ents, EntityRankMode::Degree);
        assert_eq!(ents[0].name, "hub");
    }

    #[test]
    fn query_score_mode_puts_relevant_first() {
        let mut ents = vec![ent("low_deg_high_score", 0.9, 1), ent("hub", 0.1, 50)];
        rank_entities_for_prompt(&mut ents, EntityRankMode::QueryScore);
        assert_eq!(ents[0].name, "low_deg_high_score");
        assert_eq!(ents[1].name, "hub");
    }

    #[test]
    fn query_score_ties_break_on_degree() {
        let mut ents = vec![ent("a", 0.5, 2), ent("b", 0.5, 10)];
        rank_entities_for_prompt(&mut ents, EntityRankMode::QueryScore);
        assert_eq!(ents[0].name, "b");
    }

    #[test]
<<<<<<< HEAD
    fn default_mode_is_degree() {
        assert_eq!(EntityRankMode::default().as_str(), "degree");
=======
    fn default_mode_is_retrieval() {
        assert_eq!(EntityRankMode::default().as_str(), "retrieval");
        assert_eq!(EntityRankMode::Degree.as_str(), "degree");
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        assert_eq!(EntityRankMode::QueryScore.as_str(), "query_score");
        assert_eq!(EntityRankMode::Retrieval.as_str(), "retrieval");
    }

    #[test]
    fn retrieval_mode_preserves_order() {
        let mut ents = vec![ent("first", 0.1, 1), ent("second", 0.9, 50)];
        rank_entities_for_prompt(&mut ents, EntityRankMode::Retrieval);
        assert_eq!(ents[0].name, "first");
        assert_eq!(ents[1].name, "second");
    }
}
