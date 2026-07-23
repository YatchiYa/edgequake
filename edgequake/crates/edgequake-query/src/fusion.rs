//! Retrieval fusion helpers (SPEC-023 I5).
//!
//! Mix mode supports max-after-minmax blending, Reciprocal Rank Fusion (RRF),
//! or LightRAG-style round-robin merge.
//! Default: RRF. Set `EDGEQUAKE_MIX_FUSION=max_after_minmax` (legacy alias:
//! `weighted`) or `round_robin` to ablate.
//!
//! SPEC-083 D-35: the historical "weighted" label implied a weighted sum; the
//! implementation takes the **max** of per-arm weighted min-max contributions.

use std::collections::HashMap;

use crate::context::RetrievedChunk;

/// How Mix mode combines Local, Global, and Naive chunk lists.
///
/// Note (D-36): when reused by `sparse_retrieval`, [`MixFusionMode::MaxAfterMinMax`]
/// means **sparse_first** (sparse rank order), not Mix max-after-minmax fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixFusionMode {
    /// Mix: min-max normalize per arm, weight, then take **max** contribution
    /// across arms (not a weighted sum — D-35).
    /// Sparse path (`EDGEQUAKE_SPARSE_FUSION`): **sparse_first** order (D-36).
    MaxAfterMinMax,
    /// Reciprocal Rank Fusion across ranked ID lists.
    Rrf,
    /// LightRAG-style round-robin interleave (local → global → naive).
    RoundRobin,
}

/// Read fusion mode from environment (default: RRF per SPEC-024 2.1).
pub fn mix_fusion_mode_from_env() -> MixFusionMode {
    match std::env::var("EDGEQUAKE_MIX_FUSION")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        // D-35: honest name; `weighted` kept as legacy alias.
        "weighted" | "max_after_minmax" | "max-after-minmax" | "max" => {
            MixFusionMode::MaxAfterMinMax
        }
        "round_robin" | "round-robin" | "rr" | "lightrag" => MixFusionMode::RoundRobin,
        _ => MixFusionMode::Rrf,
    }
}

/// Operator-visible label for health / dashboards (D-35 honesty).
pub fn mix_fusion_mode_label(mode: MixFusionMode) -> &'static str {
    match mode {
        MixFusionMode::MaxAfterMinMax => "max_after_minmax",
        MixFusionMode::Rrf => "rrf",
        MixFusionMode::RoundRobin => "round_robin",
    }
}

/// Standard RRF constant (Cormack et al.).
pub const RRF_K: f32 = 60.0;

/// Weighted Reciprocal Rank Fusion over ranked chunk ID lists.
///
/// Each list contributes `weight / (k + rank + 1)` to a chunk's fused score.
pub fn reciprocal_rank_fusion(
    ranked_lists: &[Vec<String>],
    weights: &[f32],
    k: f32,
) -> Vec<(String, f32)> {
    let mut scores: HashMap<String, f32> = HashMap::new();
    for (list_idx, list) in ranked_lists.iter().enumerate() {
        let weight = weights.get(list_idx).copied().unwrap_or(1.0);
        if weight <= 0.0 {
            continue;
        }
        for (rank, id) in list.iter().enumerate() {
            *scores.entry(id.clone()).or_insert(0.0) += weight / (k + rank as f32 + 1.0);
        }
    }
    let mut ranked: Vec<(String, f32)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

/// Build chunk list from RRF-ranked IDs, preserving first-seen chunk payloads.
pub fn chunks_from_rrf_ranking(
    ranked_ids: &[(String, f32)],
    chunk_lookup: &HashMap<String, RetrievedChunk>,
    max_chunks: usize,
) -> Vec<RetrievedChunk> {
    ranked_ids
        .iter()
        .filter_map(|(id, score)| {
            chunk_lookup.get(id).map(|chunk| {
                let mut c = chunk.clone();
                c.score = *score;
                c
            })
        })
        .take(max_chunks)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_promotes_consensus_across_lists() {
        let lists = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["b".to_string(), "a".to_string()],
        ];
        let weights = [1.0, 1.0];
        let fused = reciprocal_rank_fusion(&lists, &weights, RRF_K);
        assert_eq!(fused.len(), 2);
        // Symmetric ranks → tied scores; order is stable-sort arbitrary.
        assert_eq!(fused[0].1, fused[1].1);
        assert!(fused.iter().any(|(id, _)| id == "a"));
        assert!(fused.iter().any(|(id, _)| id == "b"));
    }

    #[test]
    fn zero_weight_skips_arm() {
        let lists = vec![
            vec!["only_local".to_string()],
            vec!["only_global".to_string()],
        ];
        let weights = [1.0, 0.0];
        let fused = reciprocal_rank_fusion(&lists, &weights, RRF_K);
        assert_eq!(fused.len(), 1);
        assert_eq!(fused[0].0, "only_local");
    }

    #[test]
    fn contract_mix_fusion_semantics_documented() {
        // D-35: operator label must not claim "weighted" for max-after-minmax.
        assert_eq!(
            mix_fusion_mode_label(MixFusionMode::MaxAfterMinMax),
            "max_after_minmax"
        );
        std::env::set_var("EDGEQUAKE_MIX_FUSION", "weighted");
        assert_eq!(
            mix_fusion_mode_from_env(),
            MixFusionMode::MaxAfterMinMax,
            "legacy weighted alias still selects max-after-minmax"
        );
        std::env::set_var("EDGEQUAKE_MIX_FUSION", "max_after_minmax");
        assert_eq!(mix_fusion_mode_from_env(), MixFusionMode::MaxAfterMinMax);
        std::env::remove_var("EDGEQUAKE_MIX_FUSION");
    }
}
