//! SPEC-047 MV-23 — retrieval modality on chunk KV + vector metadata.

use edgequake_pipeline::chunk_storage::build_chunk_vector_metadata;
use edgequake_pipeline::chunker::TextChunk;
use edgequake_pipeline::{
    build_chunk_kv_records, resolve_retrieval_modality_from_content,
    stamp_retrieval_modality_on_chunks, ChunkVectorBuildOptions, IngestionPersistContext,
    MmChunkSidecarMeta, MmSidecarBlock, ProcessingResult, MODALITY_CHART, MODALITY_TABLE,
};

#[test]
fn resolve_modality_from_lightrag_and_vlm_shapes() {
    assert_eq!(
        resolve_retrieval_modality_from_content("[Chart Name]rev\n[Image Type]Chart\n\nQ4: 42"),
        Some(MODALITY_CHART)
    );
    assert_eq!(
        resolve_retrieval_modality_from_content("# rev\n\n**Type:** Chart\n\n- Q4: 42"),
        Some(MODALITY_CHART)
    );
    assert_eq!(
        resolve_retrieval_modality_from_content("[Table Name]t1\n\n| A | B |"),
        Some(MODALITY_TABLE)
    );
}

#[test]
fn stamp_and_persist_modality_on_chart_chunk() {
    let body = "# rev q4\n\n**Type:** Chart\n\n**Key values:**\n- Q4: 42";
    let mut chunks = vec![TextChunk {
        id: "doc-chunk-0".into(),
        content: body.into(),
        index: 0,
        start_offset: 0,
        end_offset: 0,
        start_line: 1,
        end_line: 1,
        token_count: 12,
        embedding: None,
        section: None,
        page_start: Some(2),
        page_end: Some(2),
        modality: None,
    }];
    stamp_retrieval_modality_on_chunks(&mut chunks, &[]);
    assert_eq!(chunks[0].modality.as_deref(), Some(MODALITY_CHART));

    let kv_records = build_chunk_kv_records(
        "doc",
        Some("chart.pdf"),
        &ProcessingResult {
            document_id: "doc".into(),
            chunks: chunks.clone(),
            extractions: vec![],
            stats: Default::default(),
            lineage: None,
        },
    );
    let kv = &kv_records[0].1;
    assert_eq!(kv.get("modality").and_then(|v| v.as_str()), Some("chart"));

    let ctx = IngestionPersistContext::new("doc", None, Some("ws".into()));
    let meta = build_chunk_vector_metadata(&chunks[0], &ctx, ChunkVectorBuildOptions::STANDARD);
    assert_eq!(meta.get("modality").and_then(|v| v.as_str()), Some("chart"));
    assert_eq!(meta.get("page_start").and_then(|v| v.as_u64()), Some(2));
}

#[test]
fn stamp_prefers_mm_sidecar_for_chart_label() {
    let text = "[Chart Name]rev\n[Image Type]Chart\n\nQ4: 42".to_string();
    let mm = MmChunkSidecarMeta {
        item_id: "d1".into(),
        modality: "drawing".into(),
        text: text.clone(),
        sidecar: MmSidecarBlock {
            sidecar_type: "drawing".into(),
            id: "d1".into(),
            refs: vec![],
        },
        heading: None,
        llm_cache_list: vec![],
    };
    let mut chunks = vec![TextChunk {
        id: "c0".into(),
        content: text,
        index: 0,
        start_offset: 0,
        end_offset: 0,
        start_line: 1,
        end_line: 1,
        token_count: 8,
        embedding: None,
        section: None,
        page_start: None,
        page_end: None,
        modality: None,
    }];
    stamp_retrieval_modality_on_chunks(&mut chunks, &[mm]);
    assert_eq!(chunks[0].modality.as_deref(), Some(MODALITY_CHART));
}
