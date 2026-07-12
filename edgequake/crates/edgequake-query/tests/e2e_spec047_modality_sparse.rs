//! SPEC-047 MV-32 — BM25/FTS fusion respects chart modality filter.

use std::sync::Arc;

use edgequake_query::sparse_retrieval;
use edgequake_query::QueryEngineConfig;
use edgequake_storage::traits::VectorSearchResult;
use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{MemoryVectorStorage, MetadataFilter};

#[tokio::test]
async fn e2e_sparse_fusion_prefers_chart_modality_under_fts() {
    std::env::set_var("EDGEQUAKE_CHART_MODALITY_FILTER", "true");
    std::env::set_var("EDGEQUAKE_BM25_RETRIEVAL", "true");
    std::env::remove_var("EDGEQUAKE_SPARSE_FUSION");

    let storage =
        Arc::new(MemoryVectorStorage::new("mv32-sparse", 4).with_emulated_native_fts(true))
            as Arc<dyn VectorStorage>;

    storage
        .upsert(&[
            (
                "prose-chunk".into(),
                vec![0.99, 0.01, 0.0, 0.0],
                serde_json::json!({
                    "type": "chunk",
                    "content": "Q4 revenue grew strongly according to the annual narrative report."
                }),
            ),
            (
                "chart-chunk".into(),
                vec![0.55, 0.45, 0.0, 0.0],
                serde_json::json!({
                    "type": "chunk",
                    "modality": "chart",
                    "content": "Q4 Revenue chart: 42 million USD"
                }),
            ),
        ])
        .await
        .unwrap();

    let vector_results = vec![VectorSearchResult {
        id: "chart-chunk".into(),
        score: 0.55,
        metadata: serde_json::json!({
            "type": "chunk",
            "modality": "chart",
            "content": "Q4 Revenue chart: 42 million USD"
        }),
    }];

    let base = MetadataFilter::from_tenant_workspace_type(None, None, "chunk");
    let config = QueryEngineConfig::default();

    let (chunks, outcome) = sparse_retrieval::fuse_vector_and_bm25_chunks(
        "What was Q4 revenue in USD?",
        &vector_results,
        &storage,
        base.as_ref(),
        None,
        None,
        &config,
    )
    .await;

    assert_eq!(
        outcome,
        sparse_retrieval::SparseRetrievalOutcome::PostgresFts,
        "emulated native FTS must drive sparse fusion"
    );
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].id, "chart-chunk");
    assert_eq!(chunks[0].modality.as_deref(), Some("chart"));

    std::env::remove_var("EDGEQUAKE_CHART_MODALITY_FILTER");
    std::env::remove_var("EDGEQUAKE_BM25_RETRIEVAL");
}
