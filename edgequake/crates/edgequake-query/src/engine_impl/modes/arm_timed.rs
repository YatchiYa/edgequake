//! Shared timed arm runner for Mix/Hybrid (SPEC-046 OPS-P1 — DRY).
//!
//! OPS-P3 (OPS-24): wraps each arm in a GenAI retrieval span via
//! `edgequake-observability::with_rag_retrieval_span` (SOLID: query calls
//! helpers; observability owns attribute mapping).

use crate::context::QueryContext;
use crate::error::Result;
use edgequake_observability::{
    query_preview, record_rag_retrieval_outcome, with_rag_retrieval_span, RagRetrievalAttrs,
};
use std::time::Instant;

/// Run an arm or return empty context; always records wall time (skipped arms ≈ 0).
///
/// When `run` is true, the arm future executes inside a `rag.retrieval` span
/// labeled with `arm` / `mode`. Outcome flags (`empty_result`) are recorded
/// after the arm completes.
///
/// WHY `Box::pin`: Mix/Hybrid `tokio::join!` three arms. Without boxing, the
/// combined Future state machine (local+global+naive retrieval) overflows the
/// default tokio worker stack in debug builds (SPEC-047 smoke crash).
pub(super) async fn run_arm_timed<F, Fut>(
    run: bool,
    arm: &'static str,
    mode: &'static str,
    query_text: &str,
    top_k: usize,
    f: F,
) -> Result<(QueryContext, u64)>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<QueryContext>>,
{
    let start = Instant::now();
    let ctx = if run {
        Box::pin(with_rag_retrieval_span(
            RagRetrievalAttrs {
                data_source_id: Some("edgequake"),
                top_k: Some(top_k),
                arm: Some(arm),
                mode: Some(mode),
                query_preview: Some(query_preview(query_text, 64)),
            },
            async {
                let ctx = f().await?;
                record_rag_retrieval_outcome(
                    ctx.chunks.is_empty() && ctx.entities.is_empty(),
                    false,
                    None,
                );
                Ok::<QueryContext, crate::error::QueryError>(ctx)
            },
        ))
        .await?
    } else {
        QueryContext::new()
    };
    Ok((ctx, start.elapsed().as_millis() as u64))
}
