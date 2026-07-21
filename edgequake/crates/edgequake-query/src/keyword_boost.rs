//! Lexical keyword boost for entity retrieval (022 P3 / LightRAG HL-LL parity).
//!
//! When `EDGEQUAKE_KEYWORD_LEXICAL_BOOST=1`, bump entity scores when the entity
//! name overlaps extracted low/high-level keywords (substring, case-insensitive).

use crate::context::RetrievedEntity;

/// Env gate (default off — labeled Acc ladder only).
pub fn keyword_lexical_boost_enabled() -> bool {
    matches!(
        std::env::var("EDGEQUAKE_KEYWORD_LEXICAL_BOOST")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn norm(s: &str) -> String {
    s.trim().to_ascii_uppercase().replace('_', " ")
}

fn overlaps(entity_name: &str, keyword: &str) -> bool {
    let e = norm(entity_name);
    let k = norm(keyword);
    if e.is_empty() || k.is_empty() {
        return false;
    }
    e.contains(&k) || k.contains(&e)
}

/// Additive boost applied when a keyword matches (clamped to ≤ 1.0 after).
pub fn boost_amount() -> f32 {
    let v: f32 = std::env::var("EDGEQUAKE_KEYWORD_LEXICAL_BOOST_AMOUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.15);
    v.clamp(0.0, 1.0)
}

/// Boost entity scores in-place when names overlap keywords.
pub fn boost_entities_by_keywords(entities: &mut [RetrievedEntity], keywords: &[String]) {
    if keywords.is_empty() || entities.is_empty() {
        return;
    }
    let bump = boost_amount();
    for e in entities.iter_mut() {
        if keywords.iter().any(|k| overlaps(&e.name, k)) {
            e.score = (e.score + bump).min(1.0);
        }
    }
    // Re-sort by score desc so Mix / prompt see boosted entities first when
    // entity_rank=retrieval or query_score.
    entities.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Popular-node fallback gate. Default **on** for product; Acc sets `0`.
pub fn popular_node_fallback_enabled() -> bool {
    match std::env::var("EDGEQUAKE_POPULAR_NODE_FALLBACK")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "0" | "false" | "off" | "no" => false,
        "1" | "true" | "yes" | "on" => true,
        // Unset → enabled (historical product behavior).
        "" => true,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boosts_matching_entity() {
        let mut ents = vec![
            RetrievedEntity::new("ESOPHAGEAL_CANCER", "DISEASE", "d").with_score(0.5),
            RetrievedEntity::new("OTHER", "T", "d").with_score(0.8),
        ];
        boost_entities_by_keywords(&mut ents, &["esophageal cancer".into()]);
        assert!(ents[0].name == "ESOPHAGEAL_CANCER" || ents[0].score >= 0.65);
        let cancer = ents.iter().find(|e| e.name == "ESOPHAGEAL_CANCER").unwrap();
        assert!((cancer.score - 0.65).abs() < 1e-4);
    }

    #[test]
    fn popular_fallback_respects_env() {
        std::env::set_var("EDGEQUAKE_POPULAR_NODE_FALLBACK", "0");
        assert!(!popular_node_fallback_enabled());
        std::env::set_var("EDGEQUAKE_POPULAR_NODE_FALLBACK", "1");
        assert!(popular_node_fallback_enabled());
        std::env::remove_var("EDGEQUAKE_POPULAR_NODE_FALLBACK");
        assert!(popular_node_fallback_enabled());
    }
}
