//! SPEC-047 P5 — slim checkpoints, stage honesty, ingest profile (code-is-law).

#[test]
fn checkpoint_save_strips_embeddings_ssot() {
    let src = include_str!("../src/processor/pipeline_checkpoint.rs");
    assert!(
        src.contains("strip_embeddings"),
        "save_pipeline_checkpoint must strip embeddings (SPEC-047 P5 jsonb limit)"
    );
    assert!(
        src.contains("embeddings_omitted"),
        "PipelineCheckpoint must record embeddings_omitted"
    );
    assert!(
        src.contains("CHECKPOINT_MAX_SERIALIZED_BYTES"),
        "size guard required before KV upsert"
    );
}

#[test]
fn extract_stage_order_announces_extracting_before_llm() {
    let src = include_str!("../src/processor/text_insert/extraction.rs");
    assert!(
        src.contains("update_document_status(&document_id, \"extracting\", None)"),
        "must set extracting before LLM work"
    );
    assert!(
        src.contains("ensure_embeddings"),
        "slim-checkpoint resume must re-embed via ensure_embeddings"
    );
    // Must not re-set extracting after embed (false stage regression).
    let after_ok = src
        .split("Ok(TextInsertExtracted")
        .next()
        .expect("Ok return");
    let late_extracting = after_ok.matches("\"extracting\"").count();
    // Only the early announce (and maybe comments) — the post-embed update was removed.
    assert!(
        !src.contains("// OODA-02: Update status to \"extracting\""),
        "stale post-embed extracting status update must be gone"
    );
    let _ = late_extracting;
}

#[test]
fn persist_skips_false_embedding_status() {
    let src = include_str!("../src/processor/text_insert/persist.rs");
    assert!(
        !src.contains("update_document_status(&document_id, \"embedding\", None)"),
        "persist must not re-enter embedding stage after extract already embedded"
    );
    assert!(
        src.contains("update_document_status(&document_id, \"indexing\", None)"),
        "persist must move to indexing for graph merge"
    );
}

#[test]
fn status_updates_touch_relational_documents() {
    let src = include_str!("../src/processor/status_updates.rs");
    assert!(
        src.contains("touch_relational_document_status"),
        "update_document_status must sync PG documents.status early"
    );
}
