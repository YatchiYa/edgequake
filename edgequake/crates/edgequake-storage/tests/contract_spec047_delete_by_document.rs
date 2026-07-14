//! SPEC-047 P1a — `VectorStorage::delete_by_document` contract (memory e2e).

#[path = "support/e2e_fixtures.rs"]
mod e2e_fixtures;

use e2e_fixtures::generate_namespace;
use edgequake_storage::{MemoryVectorStorage, VectorStorage};
use serde_json::json;

const TEST_DIM: usize = 64;

fn emb(seed: f32) -> Vec<f32> {
    (0..TEST_DIM).map(|i| seed + i as f32 * 0.01).collect()
}

#[tokio::test]
async fn delete_by_document_removes_chunk_and_metadata_rows() {
    let storage = MemoryVectorStorage::new(generate_namespace(), TEST_DIM);
    storage.initialize().await.unwrap();

    let doc_a = "doc-alpha";
    let doc_b = "doc-beta";

    storage
        .upsert(&[
            (
                format!("{doc_a}-chunk-0"),
                emb(1.0),
                json!({"type": "chunk", "document_id": doc_a}),
            ),
            (
                format!("{doc_a}-chunk-1"),
                emb(2.0),
                json!({"type": "chunk", "source_document_id": doc_a}),
            ),
            (
                format!("{doc_b}-chunk-0"),
                emb(3.0),
                json!({"type": "chunk", "document_id": doc_b}),
            ),
            (
                "ent-shared".to_string(),
                emb(4.0),
                json!({"type": "entity", "entity_name": "FOO"}),
            ),
        ])
        .await
        .unwrap();

    assert_eq!(storage.count().await.unwrap(), 4);

    let deleted = storage.delete_by_document(doc_a).await.unwrap();
    assert_eq!(deleted, 2, "both doc-alpha chunk vectors must be removed");
    assert_eq!(storage.count().await.unwrap(), 2);
    assert!(storage
        .get_by_id(&format!("{doc_a}-chunk-0"))
        .await
        .unwrap()
        .is_none());
    assert!(storage
        .get_by_id(&format!("{doc_b}-chunk-0"))
        .await
        .unwrap()
        .is_some());
    assert!(storage.get_by_id("ent-shared").await.unwrap().is_some());
}

#[tokio::test]
async fn delete_by_document_empty_id_is_noop() {
    let storage = MemoryVectorStorage::new(generate_namespace(), TEST_DIM);
    storage.initialize().await.unwrap();
    storage
        .upsert(&[(
            "x-chunk-0".into(),
            emb(1.0),
            json!({"type": "chunk", "document_id": "x"}),
        )])
        .await
        .unwrap();
    assert_eq!(storage.delete_by_document("").await.unwrap(), 0);
    assert_eq!(storage.count().await.unwrap(), 1);
}
