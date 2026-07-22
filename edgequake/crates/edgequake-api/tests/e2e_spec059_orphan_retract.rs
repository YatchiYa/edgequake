//! SPEC-059 Wave 3 — orphan recover retracts mid-saga indexes.

use edgequake_api::services::{
    is_post_graph_incomplete_stage, orphan_retract_on_recover_enabled,
    retract_indexes_for_orphan_docs,
};
use edgequake_storage::{GraphStorage, MemoryGraphStorage, MemoryVectorStorage, VectorStorage};
use std::sync::Arc;

#[tokio::test]
async fn e2e_spec059_orphan_retract_clears_vectors_keeps_other_doc() {
    std::env::set_var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER", "1");
    assert!(orphan_retract_on_recover_enabled());

    let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("e2e059-orphan"));
    let vector: Arc<dyn VectorStorage> = Arc::new(MemoryVectorStorage::new("e2e059-orphan", 4));
    graph.initialize().await.unwrap();
    vector.initialize().await.unwrap();

    let orphan = "doc-orphan-merging";
    let neighbor = "doc-neighbor";
    let emb = vec![0.1, 0.2, 0.3, 0.4];
    vector
        .upsert(&[
            (
                format!("{orphan}-chunk-0"),
                emb.clone(),
                serde_json::json!({"type": "chunk", "document_id": orphan}),
            ),
            (
                format!("{neighbor}-chunk-0"),
                emb,
                serde_json::json!({"type": "chunk", "document_id": neighbor}),
            ),
        ])
        .await
        .unwrap();

    assert!(is_post_graph_incomplete_stage("merging"));
    let n = retract_indexes_for_orphan_docs(&graph, &vector, &[orphan.to_string()]).await;
    assert_eq!(n, 1);
    assert!(vector
        .get_by_id(&format!("{orphan}-chunk-0"))
        .await
        .unwrap()
        .is_none());
    assert!(
        vector
            .get_by_id(&format!("{neighbor}-chunk-0"))
            .await
            .unwrap()
            .is_some(),
        "neighbor document vectors must survive orphan retract"
    );
}
