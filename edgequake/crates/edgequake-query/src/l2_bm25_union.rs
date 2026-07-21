//! L2-only BM25∪CE citation list (027b).
//!
//! Prompt stays CE-ordered (Acc). API `sources` / L2 blob use BM25-rescored
//! Mix[:K] ∪ CE_final so Fact evidence_recall can approach BM25 Acc (~0.95)
//! without the Acc tax of Fact→BM25 on the LLM prompt (T032829Z).

use std::collections::HashSet;

use crate::context::RetrievedChunk;

fn env_flag_on(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// `EDGEQUAKE_L2_BM25_UNION=1` enables BM25∪CE citation chunks.
pub fn l2_bm25_union_enabled() -> bool {
    env_flag_on("EDGEQUAKE_L2_BM25_UNION")
}

pub fn l2_bm25_mix_top_k() -> usize {
    std::env::var("EDGEQUAKE_L2_BM25_MIX_TOP_K")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
        .clamp(1, 100)
}

/// Lexical BM25 reorder of Mix candidates (sync; uses `BM25Reranker::for_rag`).
pub async fn bm25_order_chunks(
    query: &str,
    chunks: &[RetrievedChunk],
    top_k: usize,
) -> Vec<RetrievedChunk> {
    if chunks.is_empty() {
        return Vec::new();
    }
    use edgequake_llm::Reranker;
    let reranker = edgequake_llm::BM25Reranker::for_rag();
    let docs: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let Ok(results) = reranker.rerank(query, &docs, Some(top_k)).await else {
        return chunks.iter().take(top_k).cloned().collect();
    };
    let mut scored: Vec<(f32, RetrievedChunk)> = results
        .into_iter()
        .filter_map(|r| {
            chunks.get(r.index).map(|c| {
                let mut out = c.clone();
                out.score = r.relevance_score as f32;
                (out.score, out)
            })
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(top_k).map(|(_, c)| c).collect()
}

/// How to build L2 citation chunks from BM25(Mix) and CE prompt set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L2Bm25Mode {
    /// BM25 Mix first, then CE fill (judge-friendly; Fact gold early).
    UnionBm25First,
    /// BM25 Mix only — matches BM25 Acc L2 membership (Acc prompt still CE).
    Replace,
    /// BM25 replace only when query intent is Factual (else CE prompt set).
    FactReplace,
}

pub fn l2_bm25_mode() -> L2Bm25Mode {
    match std::env::var("EDGEQUAKE_L2_BM25_MODE")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "replace" | "bm25" | "bm25_only" => L2Bm25Mode::Replace,
        "fact_replace" | "fact" => L2Bm25Mode::FactReplace,
        _ => L2Bm25Mode::UnionBm25First,
    }
}

/// Build citation chunks. Default: **BM25 first**, CE fill (026 CE-first buried Fact gold).
pub fn union_bm25_ce_chunks(
    bm25_mix: &[RetrievedChunk],
    ce_final: &[RetrievedChunk],
    mode: L2Bm25Mode,
) -> Vec<RetrievedChunk> {
    let mut out = Vec::with_capacity(ce_final.len().saturating_add(bm25_mix.len()));
    let mut seen: HashSet<String> = HashSet::new();
    // FactReplace is resolved in the query pipeline before calling this helper.
    let include_ce_fill = matches!(mode, L2Bm25Mode::UnionBm25First);
    for chunk in bm25_mix {
        if seen.insert(chunk.id.clone()) {
            out.push(chunk.clone());
        }
    }
    if include_ce_fill {
        for chunk in ce_final {
            if seen.insert(chunk.id.clone()) {
                out.push(chunk.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::RetrievedChunk;

    #[test]
    fn union_bm25_first_then_ce_fill() {
        let bm25 = vec![
            RetrievedChunk::new("a", "a", 1.0),
            RetrievedChunk::new("b", "b", 0.9),
        ];
        let ce = vec![
            RetrievedChunk::new("b", "b", 0.5),
            RetrievedChunk::new("c", "c", 0.4),
        ];
        let u = union_bm25_ce_chunks(&bm25, &ce, L2Bm25Mode::UnionBm25First);
        let ids: Vec<&str> = u.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
    }

    #[test]
    fn replace_is_bm25_only() {
        let bm25 = vec![RetrievedChunk::new("a", "a", 1.0)];
        let ce = vec![RetrievedChunk::new("c", "c", 0.4)];
        let u = union_bm25_ce_chunks(&bm25, &ce, L2Bm25Mode::Replace);
        assert_eq!(
            u.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            vec!["a"]
        );
    }
}
