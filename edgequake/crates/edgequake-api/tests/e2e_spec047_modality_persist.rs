//! SPEC-047 MV-23 — persister writes modality to KV + vector (in-process).

use std::sync::Arc;

use edgequake_pipeline::{
    build_chunk_kv_records, resolve_retrieval_modality_from_content,
    stamp_retrieval_modality_on_chunks, ChunkVectorBuildOptions, DefaultIngestionPersister,
    IngestionPersistContext, IngestionPersistSettings, IngestionPersister, NoopEntitySink,
    ProcessingResult, TextChunk, MODALITY_CHART,
};
use edgequake_storage::{
    KVStorage, MemoryGraphStorage, MemoryKVStorage, MemoryVectorStorage, VectorStorage,
};

#[tokio::test]
async fn persister_writes_chart_modality_to_kv_and_vector() {
    let body = "# rev q4\n\n**Type:** Chart\n\n**Key values:**\n- Q4: 42";
    assert_eq!(
        resolve_retrieval_modality_from_content(body),
        Some(MODALITY_CHART)
    );

    let mut chunks = vec![TextChunk {
        id: "doc-mod-chunk-0".into(),
        content: body.into(),
        index: 0,
        start_offset: 0,
        end_offset: 0,
        start_line: 1,
        end_line: 1,
        token_count: 12,
        embedding: Some(vec![0.1; 4]),
        section: None,
        page_start: Some(1),
        page_end: Some(1),
        modality: None,
    }];
    stamp_retrieval_modality_on_chunks(&mut chunks, &[]);

    let result = ProcessingResult {
        document_id: "doc-mod".into(),
        chunks,
        extractions: vec![],
        stats: Default::default(),
        lineage: None,
    };

    let kv = Arc::new(MemoryKVStorage::new("modality-kv"));
    let vector = Arc::new(MemoryVectorStorage::new("modality-vec", 4));
    vector.initialize().await.unwrap();
    let graph = Arc::new(MemoryGraphStorage::new("modality-graph"));

    let persister = DefaultIngestionPersister::from_settings(
        graph,
        vector.clone(),
        IngestionPersistSettings::default(),
        Arc::new(NoopEntitySink),
        None,
        Some(kv.clone()),
    );

    let ctx = IngestionPersistContext::new("doc-mod", None, Some("ws".into()));
    persister
        .persist(&ctx, &result, ChunkVectorBuildOptions::STANDARD)
        .await
        .expect("persist");

    let kv_records = build_chunk_kv_records("doc-mod", Some("chart.md"), &result);
    assert_eq!(
        kv_records[0].1.get("modality").and_then(|v| v.as_str()),
        Some(MODALITY_CHART)
    );

    let stored = kv.get_by_id("doc-mod-chunk-0").await.unwrap().unwrap();
    assert_eq!(
        stored.get("modality").and_then(|v| v.as_str()),
        Some(MODALITY_CHART)
    );

    let hits = vector
        .query(&[0.1; 4], 4, None)
        .await
        .expect("vector query");
    assert!(hits
        .iter()
        .any(|h| { h.metadata.get("modality").and_then(|v| v.as_str()) == Some(MODALITY_CHART) }));
}
