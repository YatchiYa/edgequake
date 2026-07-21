//! SPEC-059 Wave 2 — cancel facade retracts ANN indexes (no worker checkpoint).

use edgequake_api::services::cancel_track_with_doc_and_pdf_chain;
use edgequake_storage::{
    GraphStorage, KVStorage, MemoryGraphStorage, MemoryKVStorage, MemoryVectorStorage,
    VectorStorage,
};
use edgequake_tasks::{
    memory::MemoryTaskStorage, CancellationRegistry, SharedTaskStorage, Task, TaskType,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec059_cancel_facade_unindexes_document_vectors() {
    let storage: SharedTaskStorage = Arc::new(MemoryTaskStorage::new());
    let registry = CancellationRegistry::new();
    let kv: Arc<dyn KVStorage> = Arc::new(MemoryKVStorage::new("e2e059-cancel"));
    let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("e2e059-cancel"));
    let vector: Arc<dyn VectorStorage> = Arc::new(MemoryVectorStorage::new("e2e059-cancel", 4));
    graph.initialize().await.unwrap();
    vector.initialize().await.unwrap();
    kv.initialize().await.unwrap();

    let doc_a = "doc-a-sole";
    let doc_b = "doc-b-shared";
    let emb = vec![0.1, 0.2, 0.3, 0.4];
    vector
        .upsert(&[
            (
                format!("{doc_a}-chunk-0"),
                emb.clone(),
                serde_json::json!({"type": "chunk", "document_id": doc_a}),
            ),
            (
                "ent:SHARED".to_string(),
                emb.clone(),
                serde_json::json!({
                    "type": "entity",
                    "document_id": doc_b,
                    "source_ids": [doc_a, doc_b]
                }),
            ),
        ])
        .await
        .unwrap();

    let task = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "existing_document_id": doc_a, "text": "x" }),
    );
    let track = task.track_id.clone();
    storage.create_task(&task).await.unwrap();

    let applied =
        cancel_track_with_doc_and_pdf_chain(&storage, &registry, kv, &graph, &vector, &track)
            .await
            .expect("cancel");
    assert!(applied.cancelled);

    assert!(
        vector
            .get_by_id(&format!("{doc_a}-chunk-0"))
            .await
            .unwrap()
            .is_none(),
        "sole-doc chunk must be retracted on HTTP/facade cancel"
    );
    // Shared entity keyed by doc_b metadata may remain (delete_by_document is doc-scoped).
    // Presence of shared embedding is acceptable; absence of doc_a chunks is the gate.
}

#[tokio::test]
async fn contract_cancel_facade_source_calls_retract_helper() {
    // Source contract: cancel_facade module wires retract_indexes_for_task.
    let src = include_str!("../src/services/cancel_facade.rs");
    assert!(
        src.contains("retract_indexes_for_task"),
        "cancel_facade must call retract_indexes_for_task"
    );
    let pipeline = include_str!("../src/handlers/pipeline.rs");
    assert!(
        pipeline.contains("retract_indexes_for_task"),
        "pipeline cancel must retract"
    );
    let pdf = include_str!("../src/handlers/pdf_upload/operations.rs");
    assert!(
        pdf.contains("retract_indexes_for_task") || pdf.contains("retract_indexes_for_document_id"),
        "PDF cancel must retract"
    );
}
