//! Dual-list L2 sources: Mix∪CE while prompt stays CE-ordered (026).
//!
//! GraphRAG-Bench evidence_recall scores API `sources`, which previously mirrored
//! post-CE `context.chunks` only. CE admission can drop Mix Fact gold from that
//! set. This module builds a citation list that unions Mix top-K with the final
//! CE prompt chunks without changing the LLM prompt order.

use std::collections::HashSet;

use crate::context::RetrievedChunk;

fn env_flag_on(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// `EDGEQUAKE_L2_SOURCES_UNION=1` enables Mix∪CE citation chunks.
pub fn l2_sources_union_enabled() -> bool {
    env_flag_on("EDGEQUAKE_L2_SOURCES_UNION")
}

/// How many Mix first-stage chunks to retain for L2 union (default 30).
pub fn l2_sources_mix_top_k() -> usize {
    std::env::var("EDGEQUAKE_L2_SOURCES_MIX_TOP_K")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
        .clamp(1, 100)
}

/// Build citation chunks: CE prompt order first, then Mix[:K] fill for recall.
///
/// Deduplicates by chunk id. Prompt generation must keep using `ce_final` only.
pub fn union_mix_ce_chunks(
    mix_pre_ce: &[RetrievedChunk],
    ce_final: &[RetrievedChunk],
    mix_top_k: usize,
) -> Vec<RetrievedChunk> {
    let mut out = Vec::with_capacity(ce_final.len().saturating_add(mix_top_k));
    let mut seen: HashSet<String> = HashSet::new();
    for chunk in ce_final {
        if seen.insert(chunk.id.clone()) {
            out.push(chunk.clone());
        }
    }
    for chunk in mix_pre_ce.iter().take(mix_top_k) {
        if seen.insert(chunk.id.clone()) {
            out.push(chunk.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RetrievedChunk;

    #[test]
    fn union_keeps_ce_order_and_adds_mix_only() {
        let mix = vec![
            RetrievedChunk::new("a", "a", 1.0),
            RetrievedChunk::new("b", "b", 0.9),
            RetrievedChunk::new("c", "c", 0.8),
        ];
        let ce = vec![
            RetrievedChunk::new("b", "b", 0.95),
            RetrievedChunk::new("d", "d", 0.5),
        ];
        let u = union_mix_ce_chunks(&mix, &ce, 3);
        let ids: Vec<&str> = u.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["b", "d", "a", "c"]);
    }
}
