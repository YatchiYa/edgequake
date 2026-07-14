//! SPEC-047 MV-32 — prefer chart-modality chunks for numeric/chart queries.
//!
//! First principle: when a question targets a chart value, retrieval should
//! search `modality=chart` first (SQL pre-filter) before falling back to all chunks.
//! Shared between dense vector search and Postgres/native FTS (BM25 fusion path).

use std::future::Future;
use std::sync::Arc;

use edgequake_storage::traits::{MetadataFilter, VectorSearchResult, VectorStorage};

use crate::error::Result;

/// Vector metadata value for chart chunks (aligned with MV-23 ingest).
pub const MODALITY_CHART: &str = "chart";

/// Env gate: set `false`/`0`/`off` to disable chart modality pre-filter.
pub fn chart_modality_filter_enabled() -> bool {
    match std::env::var("EDGEQUAKE_CHART_MODALITY_FILTER")
        .ok()
        .as_deref()
        .map(str::trim)
    {
        Some("0") | Some("false") | Some("off") | Some("no") => false,
        Some(_) => true,
        None => true,
    }
}

/// Heuristic: query likely needs chart/numeric evidence (MMLongBench-style).
pub fn query_prefers_chart_modality(query: &str) -> bool {
    let lower = query.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }

    if lower.contains("chart")
        || lower.contains("graph")
        || lower.contains("bar chart")
        || lower.contains("pie chart")
        || lower.contains("infographic")
    {
        return true;
    }

    if lower.contains("q1")
        || lower.contains("q2")
        || lower.contains("q3")
        || lower.contains("q4")
        || lower.contains("quarter")
    {
        return true;
    }

    const METRIC_TERMS: &[&str] = &[
        "revenue",
        "sales",
        "profit",
        "percent",
        "percentage",
        "total",
        "amount",
        "value",
        "cost",
        "price",
        "rate",
        "growth",
        "million",
        "billion",
        "usd",
        "$",
        "€",
        "£",
        "yoy",
        "mom",
    ];

    let has_digit = lower.chars().any(|c| c.is_ascii_digit());
    if has_digit && METRIC_TERMS.iter().any(|t| lower.contains(t)) {
        return true;
    }

    if (lower.contains("how much") || lower.contains("how many"))
        && METRIC_TERMS.iter().any(|t| lower.contains(t))
    {
        return true;
    }

    false
}

/// Merge chart modality constraint into an existing scope filter.
pub fn with_chart_modality_filter(base: Option<MetadataFilter>) -> MetadataFilter {
    let mut mf = base.unwrap_or_default();
    mf.modalities = Some(vec![MODALITY_CHART.to_string()]);
    mf
}

/// Resolved chart-modality filter for dense + sparse retrieval (DRY SSOT).
#[derive(Debug, Clone)]
pub struct ModalityFilterPlan {
    /// True when chart pre-filter is active for this query.
    pub chart_prefilter_active: bool,
    /// Filter tried first (includes `modality=chart` when active).
    pub strict_filter: MetadataFilter,
    /// Scope filter used when strict chart filter returns no hits.
    pub fallback_filter: Option<MetadataFilter>,
}

/// Build the modality filter plan for a query (vector + FTS share this).
pub fn plan_modality_retrieval(
    query_text: &str,
    base_filter: Option<&MetadataFilter>,
) -> ModalityFilterPlan {
    let base = base_filter.cloned().unwrap_or_default();
    if chart_modality_filter_enabled() && query_prefers_chart_modality(query_text) {
        ModalityFilterPlan {
            chart_prefilter_active: true,
            strict_filter: with_chart_modality_filter(Some(base.clone())),
            fallback_filter: Some(base),
        }
    } else {
        ModalityFilterPlan {
            chart_prefilter_active: false,
            strict_filter: base,
            fallback_filter: None,
        }
    }
}

async fn search_with_modality_plan<T, F>(
    plan: &ModalityFilterPlan,
    query_text: &str,
    search: F,
) -> Result<Vec<T>>
where
    F: Fn(MetadataFilter) -> std::pin::Pin<Box<dyn Future<Output = Result<Vec<T>>> + Send>>,
{
    let strict = search(plan.strict_filter.clone()).await?;
    if !strict.is_empty() {
        if plan.chart_prefilter_active {
            tracing::debug!(
                query = %query_text,
                "MV-32: chart modality pre-filter returned results"
            );
        }
        return Ok(strict);
    }

    if let Some(fallback) = plan.fallback_filter.clone() {
        tracing::debug!(
            query = %query_text,
            "MV-32: chart modality filter empty — falling back to unfiltered chunks"
        );
        return search(fallback).await;
    }

    Ok(strict)
}

/// Vector search with chart modality pre-filter when `query_text` warrants it.
pub async fn query_filtered_with_modality_preference(
    vector_storage: &Arc<dyn VectorStorage>,
    query_text: &str,
    query_embedding: &[f32],
    top_k: usize,
    filter_ids: Option<&[String]>,
    base_filter: Option<&MetadataFilter>,
) -> Result<Vec<VectorSearchResult>> {
    let plan = plan_modality_retrieval(query_text, base_filter);
    let storage = Arc::clone(vector_storage);
    let embedding = query_embedding.to_vec();
    let filter_ids = filter_ids.map(|ids| ids.to_vec());
    search_with_modality_plan(&plan, query_text, move |mf| {
        let storage = Arc::clone(&storage);
        let embedding = embedding.clone();
        let filter_ids = filter_ids.clone();
        Box::pin(async move {
            storage
                .query_filtered(&embedding, top_k, filter_ids.as_deref(), Some(&mf))
                .await
                .map_err(Into::into)
        })
    })
    .await
}

/// FTS / sparse search with the same chart modality fail-open semantics as dense search.
pub async fn text_search_with_modality_preference(
    vector_storage: &Arc<dyn VectorStorage>,
    query_text: &str,
    top_k: usize,
    filter_ids: Option<&[String]>,
    base_filter: Option<&MetadataFilter>,
) -> Result<Vec<VectorSearchResult>> {
    let plan = plan_modality_retrieval(query_text, base_filter);
    let storage = Arc::clone(vector_storage);
    let query = query_text.to_string();
    let filter_ids = filter_ids.map(|ids| ids.to_vec());
    search_with_modality_plan(&plan, query_text, move |mf| {
        let storage = Arc::clone(&storage);
        let query = query.clone();
        let filter_ids = filter_ids.clone();
        Box::pin(async move {
            storage
                .text_search_filtered(&query, top_k, filter_ids.as_deref(), Some(&mf))
                .await
                .map_err(Into::into)
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::MemoryVectorStorage;
    use std::sync::Arc;

    #[test]
    fn chart_query_heuristic_covers_bench_style_questions() {
        assert!(query_prefers_chart_modality(
            "What was Q4 revenue in USD millions?"
        ));
        assert!(query_prefers_chart_modality(
            "According to the chart, what is the total sales?"
        ));
        assert!(query_prefers_chart_modality(
            "How much profit grew in 2023?"
        ));
        assert!(!query_prefers_chart_modality(
            "Who is the CEO of the company?"
        ));
        assert!(!query_prefers_chart_modality(
            "Explain the architecture diagram"
        ));
    }

    #[test]
    fn with_chart_modality_filter_sets_modalities() {
        let base = MetadataFilter::from_tenant_workspace_type(None, None, "chunk");
        let mf = with_chart_modality_filter(base);
        assert_eq!(
            mf.modalities.as_deref(),
            Some([MODALITY_CHART.to_string()].as_slice())
        );
        assert_eq!(mf.vector_type.as_deref(), Some("chunk"));
    }

    #[test]
    fn plan_modality_retrieval_marks_chart_queries() {
        let base = MetadataFilter::from_tenant_workspace_type(None, None, "chunk");
        let plan = plan_modality_retrieval("What was Q4 revenue?", base.as_ref());
        assert!(plan.chart_prefilter_active);
        assert_eq!(
            plan.strict_filter.modalities.as_deref(),
            Some([MODALITY_CHART.to_string()].as_slice())
        );
        assert!(plan.fallback_filter.is_some());
    }

    #[tokio::test]
    async fn text_search_with_modality_preference_filters_chart_only() {
        std::env::set_var("EDGEQUAKE_CHART_MODALITY_FILTER", "true");
        let storage =
            Arc::new(MemoryVectorStorage::new("fts-modality", 4).with_emulated_native_fts(true))
                as Arc<dyn VectorStorage>;
        storage
            .upsert(&[
                (
                    "prose-chunk".into(),
                    vec![0.5, 0.5, 0.0, 0.0],
                    serde_json::json!({
                        "type": "chunk",
                        "content": "Q4 revenue overview in narrative prose without chart data"
                    }),
                ),
                (
                    "chart-chunk".into(),
                    vec![0.5, 0.5, 0.0, 0.0],
                    serde_json::json!({
                        "type": "chunk",
                        "modality": "chart",
                        "content": "Q4 Revenue chart value: 42 million USD"
                    }),
                ),
            ])
            .await
            .unwrap();

        let base = MetadataFilter::from_tenant_workspace_type(None, None, "chunk");
        let hits = text_search_with_modality_preference(
            &storage,
            "Q4 revenue USD",
            5,
            None,
            base.as_ref(),
        )
        .await
        .unwrap();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "chart-chunk");
        std::env::remove_var("EDGEQUAKE_CHART_MODALITY_FILTER");
    }
}
