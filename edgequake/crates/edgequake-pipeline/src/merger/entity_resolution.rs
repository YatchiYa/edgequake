//! SPEC-091 RM2 — entity resolution ladder (LAW-RM7 / RM-AC-07).
//!
//! Stages (conservative, precision over recall):
//! 1. Exact normalized name-key (always)
//! 2. Optional embedding similarity when `EDGEQUAKE_ENTITY_EMBED_ER=on`
//! 3. Optional LLM adjudicate when `EDGEQUAKE_ER_LLM=on` (default off)
//!
//! String fuzzy (`EDGEQUAKE_ENTITY_FUZZY`) remains a prefilter only.

use std::sync::atomic::{AtomicU64, Ordering};

pub const ENTITY_EMBED_ER_ENV: &str = "EDGEQUAKE_ENTITY_EMBED_ER";
pub const ER_LLM_ENV: &str = "EDGEQUAKE_ER_LLM";

/// Default cosine similarity threshold for embed ER (conservative).
pub const DEFAULT_EMBED_ER_THRESHOLD: f32 = 0.92;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErDecision {
    ExactMatch,
    EmbedMerge,
    LlmMerge,
    CreateNew,
}

static MERGE_EXACT: AtomicU64 = AtomicU64::new(0);
static MERGE_EMBED: AtomicU64 = AtomicU64::new(0);
static MERGE_LLM: AtomicU64 = AtomicU64::new(0);
static CREATE_NEW: AtomicU64 = AtomicU64::new(0);

#[allow(dead_code)] // metrics for /health / ops dashboards
pub fn er_merge_exact_total() -> u64 {
    MERGE_EXACT.load(Ordering::Relaxed)
}
#[allow(dead_code)]
pub fn er_merge_embed_total() -> u64 {
    MERGE_EMBED.load(Ordering::Relaxed)
}
#[allow(dead_code)]
pub fn er_merge_llm_total() -> u64 {
    MERGE_LLM.load(Ordering::Relaxed)
}
#[allow(dead_code)]
pub fn er_create_new_total() -> u64 {
    CREATE_NEW.load(Ordering::Relaxed)
}

pub fn entity_embed_er_enabled() -> bool {
    matches!(
        std::env::var(ENTITY_EMBED_ER_ENV)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "on" | "1" | "true" | "yes"
    )
}

pub fn er_llm_enabled() -> bool {
    matches!(
        std::env::var(ER_LLM_ENV)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "on" | "1" | "true" | "yes"
    )
}

/// Cosine similarity for equal-length embeddings.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-12);
    dot / denom
}

/// Resolve identity after an exact name-key miss.
///
/// `candidates` are (entity_key, embedding) pairs already blocked (e.g. same
/// type / fuzzy prefilter). Returns the key to merge into, or `None` → create.
pub fn resolve_after_exact_miss(
    mention_embedding: Option<&[f32]>,
    candidates: &[(String, Vec<f32>)],
    threshold: f32,
    llm_says_same: Option<bool>,
) -> (ErDecision, Option<String>) {
    if entity_embed_er_enabled() {
        if let Some(emb) = mention_embedding {
            let mut best: Option<(f32, &str)> = None;
            for (key, cand) in candidates {
                let sim = cosine_similarity(emb, cand);
                if sim >= threshold {
                    match best {
                        Some((s, _)) if sim <= s => {}
                        _ => best = Some((sim, key.as_str())),
                    }
                }
            }
            if let Some((_, key)) = best {
                MERGE_EMBED.fetch_add(1, Ordering::Relaxed);
                return (ErDecision::EmbedMerge, Some(key.to_string()));
            }
        }
    }

    if er_llm_enabled() {
        if let Some(true) = llm_says_same {
            if let Some((key, _)) = candidates.first() {
                MERGE_LLM.fetch_add(1, Ordering::Relaxed);
                return (ErDecision::LlmMerge, Some(key.clone()));
            }
        }
    }

    CREATE_NEW.fetch_add(1, Ordering::Relaxed);
    (ErDecision::CreateNew, None)
}

pub fn record_exact_merge() {
    MERGE_EXACT.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_spec091_er_ladder_exact_prefer_create_when_off() {
        std::env::remove_var(ENTITY_EMBED_ER_ENV);
        std::env::remove_var(ER_LLM_ENV);
        let (d, k) = resolve_after_exact_miss(
            Some(&[1.0, 0.0]),
            &[("OTHER".into(), vec![1.0, 0.0])],
            DEFAULT_EMBED_ER_THRESHOLD,
            None,
        );
        assert_eq!(d, ErDecision::CreateNew);
        assert!(k.is_none());
    }

    #[test]
    fn contract_spec091_er_ladder_embed_merge() {
        std::env::set_var(ENTITY_EMBED_ER_ENV, "on");
        std::env::remove_var(ER_LLM_ENV);
        let (d, k) = resolve_after_exact_miss(
            Some(&[1.0, 0.0]),
            &[("ACME".into(), vec![0.99, 0.01])],
            0.9,
            None,
        );
        assert_eq!(d, ErDecision::EmbedMerge);
        assert_eq!(k.as_deref(), Some("ACME"));
        std::env::remove_var(ENTITY_EMBED_ER_ENV);
    }

    #[test]
    fn contract_spec091_er_llm_default_off() {
        std::env::remove_var(ER_LLM_ENV);
        assert!(!er_llm_enabled());
    }
}
