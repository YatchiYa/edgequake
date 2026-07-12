//! MetadataFilter predicate parity (SPEC-017 STORE-DRY-001).

use edgequake_storage::adapters::memory::MemoryVectorStorage;
use edgequake_storage::traits::{MetadataFilter, VectorStorage};

#[tokio::test]
async fn memory_query_filtered_uses_shared_predicate() {
    let storage = MemoryVectorStorage::new("mf-test", 3);

    storage
        .upsert(&[(
            "chunk-a".into(),
            vec![1.0, 0.0, 0.0],
            serde_json::json!({
                "type": "chunk",
                "tenant_id": "t1",
                "workspace_id": "ws1",
                "document_id": "doc-a"
            }),
        )])
        .await
        .unwrap();
    storage
        .upsert(&[(
            "chunk-b".into(),
            vec![0.9, 0.1, 0.0],
            serde_json::json!({
                "type": "chunk",
                "tenant_id": "t1",
                "workspace_id": "ws2",
                "document_id": "doc-b"
            }),
        )])
        .await
        .unwrap();

    let filter =
        MetadataFilter::from_tenant_workspace_type(Some("t1".into()), Some("ws1".into()), "chunk")
            .unwrap();

    let results = storage
        .query_filtered(&[1.0, 0.0, 0.0], 10, None, Some(&filter))
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "chunk-a");
}

#[tokio::test]
async fn memory_query_filtered_chart_modality_predicate() {
    use edgequake_storage::MetadataFilter;

    let storage = MemoryVectorStorage::new("mf-modality", 3);
    storage
        .upsert(&[
            (
                "prose".into(),
                vec![0.99, 0.01, 0.0],
                serde_json::json!({"type": "chunk", "content": "revenue prose"}),
            ),
            (
                "chart".into(),
                vec![0.95, 0.05, 0.0],
                serde_json::json!({"type": "chunk", "modality": "chart", "content": "Q4: 42"}),
            ),
        ])
        .await
        .unwrap();

    let filter = MetadataFilter {
        vector_type: Some("chunk".into()),
        modalities: Some(vec!["chart".into()]),
        ..Default::default()
    };

    let results = storage
        .query_filtered(&[1.0, 0.0, 0.0], 10, None, Some(&filter))
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "chart");
}
