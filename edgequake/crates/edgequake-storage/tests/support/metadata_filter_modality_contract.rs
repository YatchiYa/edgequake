//! SPEC-047 MV-32 / MV-23 — modality metadata filter contract (STORE-DRY-004).
//!
//! Single assertion module for [`MetadataFilter::modalities`] parity across vector backends:
//! - MemoryVectorStorage (`query_filtered` + optional emulated FTS)
//! - PgVectorStorage (`query_filtered` + native FTS)

use edgequake_storage::traits::{MetadataFilter, VectorStorage};

/// Chart-only filter used by MV-32 retrieval.
pub fn chart_chunk_filter() -> MetadataFilter {
    MetadataFilter {
        vector_type: Some("chunk".into()),
        modalities: Some(vec!["chart".into()]),
        ..Default::default()
    }
}

fn modality_fixture_embedding(dim: usize, seed: f32) -> Vec<f32> {
    (0..dim)
        .map(|i| ((i as f32 + seed) / 1000.0).sin())
        .collect()
}

/// Seed prose + chart chunks; prose vector scores higher to prove filter beats rank.
pub async fn seed_modality_vector_fixtures<V: VectorStorage + ?Sized>(storage: &V) {
    let dim = storage.dimension();
    storage
        .upsert(&[
            (
                "prose-chunk".into(),
                modality_fixture_embedding(dim, 0.99),
                serde_json::json!({
                    "type": "chunk",
                    "content": "Q4 revenue grew according to the annual narrative report."
                }),
            ),
            (
                "chart-chunk".into(),
                modality_fixture_embedding(dim, 0.95),
                serde_json::json!({
                    "type": "chunk",
                    "modality": "chart",
                    "content": "Q4 Revenue chart value: 42 million USD"
                }),
            ),
            (
                "table-chunk".into(),
                modality_fixture_embedding(dim, 0.90),
                serde_json::json!({
                    "type": "chunk",
                    "modality": "table",
                    "content": "Q4 Revenue table row: 42 million USD"
                }),
            ),
        ])
        .await
        .expect("seed modality fixtures");
}

/// Dense search: `modalities=chart` must return only chart-tagged chunks.
pub async fn assert_query_filtered_chart_modality<V: VectorStorage + ?Sized>(storage: &V) {
    seed_modality_vector_fixtures(storage).await;

    let query = modality_fixture_embedding(storage.dimension(), 1.0);
    let filter = chart_chunk_filter();

    let results = storage
        .query_filtered(&query, 10, None, Some(&filter))
        .await
        .expect("query_filtered chart modality");

    assert_eq!(results.len(), 1, "chart filter must exclude prose/table");
    assert_eq!(results[0].id, "chart-chunk");
    assert_eq!(
        results[0].metadata.get("modality").and_then(|v| v.as_str()),
        Some("chart")
    );
}

/// Strict filter: vectors without `modality` must not match chart filter.
#[allow(dead_code)] // exercised by modality storage contract tests; support shared across crates
pub async fn assert_query_filtered_excludes_missing_modality<V: VectorStorage + ?Sized>(
    storage: &V,
) {
    let dim = storage.dimension();
    storage
        .upsert(&[(
            "no-modality".into(),
            modality_fixture_embedding(dim, 1.0),
            serde_json::json!({
                "type": "chunk",
                "content": "Q4 revenue without modality tag"
            }),
        )])
        .await
        .expect("upsert no-modality chunk");

    let results = storage
        .query_filtered(
            &modality_fixture_embedding(dim, 1.0),
            5,
            None,
            Some(&chart_chunk_filter()),
        )
        .await
        .expect("query_filtered strict modality");

    assert!(
        results.is_empty(),
        "chunks without modality metadata must be excluded when chart filter is active"
    );
}

/// Sparse / FTS path: when native text search is supported, chart filter applies there too.
pub async fn assert_text_search_chart_modality_when_supported<V: VectorStorage + ?Sized>(
    storage: &V,
) {
    if !storage.supports_native_text_search() {
        return;
    }

    seed_modality_vector_fixtures(storage).await;
    let filter = chart_chunk_filter();

    let hits = storage
        .text_search_filtered("Q4 revenue USD", 10, None, Some(&filter))
        .await
        .expect("text_search_filtered chart modality");

    assert!(!hits.is_empty(), "FTS must find chart content");
    assert!(
        hits.iter()
            .all(|h| { h.metadata.get("modality").and_then(|v| v.as_str()) == Some("chart") }),
        "FTS hits must respect modality pre-filter"
    );
    assert_eq!(hits[0].id, "chart-chunk");
}

/// Memory default: FTS unsupported unless emulated — must return empty without error.
#[allow(dead_code)] // exercised when FTS modality filter is unsupported by backend
pub async fn assert_text_search_empty_when_unsupported<V: VectorStorage + ?Sized>(storage: &V) {
    if storage.supports_native_text_search() {
        return;
    }

    seed_modality_vector_fixtures(storage).await;
    let hits = storage
        .text_search_filtered("Q4 revenue", 5, None, Some(&chart_chunk_filter()))
        .await
        .expect("text_search on unsupported backend");
    assert!(hits.is_empty());
}
