//! D-32: entity_type majority / confidence voting (no silent first-wins).
//!
//! Votes accumulate on node property `entity_type_votes` as
//! `{ "PERSON": 1.5, "ORGANIZATION": 0.8 }`. Resolution prefers the highest
//! confidence sum among non-`OTHER` types; ties keep the previous type.

use std::collections::HashMap;

use serde_json::Value;

/// Property key for accumulated type votes.
pub const ENTITY_TYPE_VOTES_KEY: &str = "entity_type_votes";

/// Normalize a raw type for voting (uppercase, trim; empty → OTHER).
pub fn normalize_type_label(raw: &str) -> String {
    let t = raw.trim().to_uppercase();
    if t.is_empty() {
        "OTHER".to_string()
    } else {
        t
    }
}

/// Read vote map from node properties.
pub fn votes_from_properties(props: &HashMap<String, Value>) -> HashMap<String, f64> {
    props
        .get(ENTITY_TYPE_VOTES_KEY)
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_f64().map(|n| (normalize_type_label(k), n)))
                .collect()
        })
        .unwrap_or_default()
}

/// Seed votes from an existing stored type (confidence 1.0) when no map yet.
pub fn ensure_seed_vote(votes: &mut HashMap<String, f64>, existing_type: &str) {
    if votes.is_empty() {
        let t = normalize_type_label(existing_type);
        if t != "OTHER" || !existing_type.trim().is_empty() {
            votes.insert(t, 1.0);
        }
    }
}

/// Add a weighted vote for `incoming_type` (weight = confidence, clamped).
pub fn add_type_vote(votes: &mut HashMap<String, f64>, incoming_type: &str, confidence: f32) {
    let t = normalize_type_label(incoming_type);
    let w = f64::from(confidence.clamp(0.05, 1.0));
    *votes.entry(t).or_default() += w;
}

/// Resolve winning type from votes.
///
/// Rules:
/// 1. Prefer non-`OTHER` types with the highest vote sum.
/// 2. If only `OTHER` has votes, return `OTHER`.
/// 3. On tie among concrete types, prefer `prefer` when it is among the tied winners.
pub fn resolve_majority_type(votes: &HashMap<String, f64>, prefer: &str) -> String {
    if votes.is_empty() {
        return normalize_type_label(prefer);
    }
    let prefer_n = normalize_type_label(prefer);
    let concrete: Vec<(&String, f64)> = votes
        .iter()
        .filter(|(k, _)| k.as_str() != "OTHER")
        .map(|(k, v)| (k, *v))
        .collect();
    if concrete.is_empty() {
        return "OTHER".to_string();
    }
    let max = concrete
        .iter()
        .map(|(_, v)| *v)
        .fold(f64::NEG_INFINITY, f64::max);
    let winners: Vec<&String> = concrete
        .iter()
        .filter(|(_, v)| (*v - max).abs() < 1e-9)
        .map(|(k, _)| *k)
        .collect();
    if winners.iter().any(|w| w.as_str() == prefer_n) {
        return prefer_n;
    }
    winners
        .into_iter()
        .min_by(|a, b| a.cmp(b))
        .cloned()
        .unwrap_or(prefer_n)
}

/// Apply an incoming type vote, update properties, and return whether the stored
/// `entity_type` changed. Always logs concrete conflicts via `tracing::info!`.
pub fn apply_entity_type_vote(
    props: &mut HashMap<String, Value>,
    entity_name: &str,
    incoming_type: &str,
    confidence: f32,
) -> bool {
    if incoming_type.trim().is_empty() {
        return false;
    }
    let existing_type = props
        .get("entity_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut votes = votes_from_properties(props);
    ensure_seed_vote(&mut votes, &existing_type);
    add_type_vote(&mut votes, incoming_type, confidence);

    let resolved = resolve_majority_type(&votes, &existing_type);
    let incoming_n = normalize_type_label(incoming_type);
    let existing_n = normalize_type_label(&existing_type);

    if !existing_n.is_empty()
        && existing_n != "OTHER"
        && incoming_n != "OTHER"
        && incoming_n != existing_n
    {
        tracing::info!(
            entity = %entity_name,
            existing_type = %existing_n,
            incoming_type = %incoming_n,
            resolved_type = %resolved,
            confidence,
            metric = "entity_type_conflict",
            "D-32: entity_type conflict — majority/confidence resolve"
        );
    }

    props.insert(
        ENTITY_TYPE_VOTES_KEY.to_string(),
        Value::Object(
            votes
                .iter()
                .map(|(k, v)| (k.clone(), Value::from(*v)))
                .collect(),
        ),
    );

    let changed = resolved != existing_n && !(existing_n.is_empty() && resolved == "OTHER");
    if resolved != existing_type {
        props.insert("entity_type".to_string(), Value::String(resolved));
    }
    changed
}

/// In-memory vote merge for within-batch dedup (same EntityId, conflicting types).
pub fn merge_type_into_entity(
    existing_type: &mut String,
    type_votes: &mut HashMap<String, f64>,
    incoming_type: &str,
    confidence: f32,
    entity_name: &str,
) {
    ensure_seed_vote(type_votes, existing_type);
    add_type_vote(type_votes, incoming_type, confidence);
    let resolved = resolve_majority_type(type_votes, existing_type);
    let incoming_n = normalize_type_label(incoming_type);
    let existing_n = normalize_type_label(existing_type);
    if !existing_n.is_empty()
        && existing_n != "OTHER"
        && incoming_n != "OTHER"
        && incoming_n != existing_n
    {
        tracing::info!(
            entity = %entity_name,
            existing_type = %existing_n,
            incoming_type = %incoming_n,
            resolved_type = %resolved,
            confidence,
            metric = "entity_type_conflict",
            "D-32: within-batch entity_type conflict — majority/confidence resolve"
        );
    }
    *existing_type = resolved;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_beats_first_wins() {
        let mut votes = HashMap::new();
        add_type_vote(&mut votes, "PERSON", 0.5);
        add_type_vote(&mut votes, "ORGANIZATION", 0.9);
        add_type_vote(&mut votes, "ORGANIZATION", 0.9);
        assert_eq!(resolve_majority_type(&votes, "PERSON"), "ORGANIZATION");
    }

    #[test]
    fn other_does_not_override_concrete() {
        let mut votes = HashMap::new();
        add_type_vote(&mut votes, "PERSON", 0.5);
        add_type_vote(&mut votes, "OTHER", 2.0);
        assert_eq!(resolve_majority_type(&votes, "PERSON"), "PERSON");
    }

    #[test]
    fn apply_updates_properties_and_logs_path() {
        let mut props = HashMap::new();
        props.insert(
            "entity_type".to_string(),
            Value::String("PERSON".to_string()),
        );
        apply_entity_type_vote(&mut props, "ACME", "ORGANIZATION", 0.9);
        apply_entity_type_vote(&mut props, "ACME", "ORGANIZATION", 0.9);
        assert_eq!(
            props.get("entity_type").and_then(|v| v.as_str()),
            Some("ORGANIZATION")
        );
        assert!(props.contains_key(ENTITY_TYPE_VOTES_KEY));
    }
}
