//! Map engine [`edgequake_query::QueryStats`] → HTTP [`QueryStats`].
//!
//! SOLID: single responsibility — one place owns the API stats projection.
//! DRY: `/query`, chat completion, and tests all call [`from_engine_stats`].
//! SPEC-047 W0b: surface `context_empty`, arm timings, and arm chunk counts
//! so harness diagnostics are law (engine fields), not heuristics.

use crate::handlers::query_types::QueryStats;
use edgequake_query::{QueryContext, QueryStats as EngineQueryStats};

/// Build HTTP query stats from engine stats + retrieved context sizes.
pub fn from_engine_stats(
    engine: &EngineQueryStats,
    context: &QueryContext,
    llm_provider: Option<String>,
    llm_model: Option<String>,
) -> QueryStats {
    let tokens_used = if engine.generated_tokens > 0 {
        Some(engine.generated_tokens)
    } else {
        None
    };

    let tokens_per_second = if engine.generation_time_ms > 0 && engine.generated_tokens > 0 {
        Some((engine.generated_tokens as f32) / (engine.generation_time_ms as f32) * 1000.0)
    } else {
        None
    };

    QueryStats {
        embedding_time_ms: engine.embedding_time_ms,
        keyword_time_ms: engine.keyword_time_ms,
        retrieval_time_ms: engine.retrieval_time_ms,
        generation_time_ms: engine.generation_time_ms,
        ttft_ms: engine.ttft_ms,
        answer_cache_hit: engine.answer_cache_hit,
        total_time_ms: engine.total_time_ms,
        sources_retrieved: context.chunks.len()
            + context.entities.len()
            + context.relationships.len(),
        rerank_time_ms: engine.rerank_time_ms,
        tokens_used,
        tokens_per_second,
        llm_provider,
        llm_model,
        context_empty: engine.context_empty,
        context_truncated: engine.context_truncated,
        arm_local_ms: engine.arm_local_ms,
        arm_global_ms: engine.arm_global_ms,
        arm_naive_ms: engine.arm_naive_ms,
        arm_local_chunks: engine.arm_local_chunks,
        arm_global_chunks: engine.arm_global_chunks,
        arm_naive_chunks: engine.arm_naive_chunks,
        arms_run: engine.arms_run.clone(),
        arms_gated: engine.arms_gated,
        query_intent: engine.query_intent.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_query::QueryStats as EngineQueryStats;

    #[test]
    fn maps_context_empty_and_arms() {
        let engine = EngineQueryStats {
            context_empty: true,
            arms_run: Some("local,naive".into()),
            arms_gated: Some(true),
            arm_local_ms: Some(11),
            arm_naive_ms: Some(22),
            arm_local_chunks: Some(3),
            arm_naive_chunks: Some(5),
            ..Default::default()
        };
        let ctx = QueryContext::new();
        let stats = from_engine_stats(&engine, &ctx, Some("mistral".into()), Some("small".into()));
        assert!(stats.context_empty);
        assert_eq!(stats.arms_run.as_deref(), Some("local,naive"));
        assert_eq!(stats.arms_gated, Some(true));
        assert_eq!(stats.arm_local_chunks, Some(3));
        assert_eq!(stats.arm_naive_chunks, Some(5));
        assert_eq!(stats.sources_retrieved, 0);
        assert_eq!(stats.llm_provider.as_deref(), Some("mistral"));
    }
}
