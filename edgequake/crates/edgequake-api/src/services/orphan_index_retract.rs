//! SPEC-059: retract searchable indexes for orphaned incomplete documents.
//!
//! When a process dies mid-saga, indexes may remain searchable. On recover
//! (manual-resume / failed path) we unindex; auto-resume leaves indexes for
//! checkpoint resume.

use std::sync::Arc;

use edgequake_storage::traits::{GraphStorage, VectorStorage};

use super::retract_document_indexes::retract_document_indexes;

/// `EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER` — default **on** (SPEC-059).
pub fn orphan_retract_on_recover_enabled() -> bool {
    match std::env::var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER") {
        Ok(v) => !matches!(
            v.to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        Err(_) => true,
    }
}

/// Stages that may have written graph/vector indexes before crash.
pub fn is_post_graph_incomplete_stage(stage: &str) -> bool {
    matches!(
        stage,
        "extracting"
            | "gleaning"
            | "merging"
            | "summarizing"
            | "embedding"
            | "storing"
            | "indexing"
            | "processing"
    )
}

/// Best-effort retract for each document id (idempotent).
pub async fn retract_indexes_for_orphan_docs(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    document_ids: &[String],
) -> usize {
    if !orphan_retract_on_recover_enabled() || document_ids.is_empty() {
        return 0;
    }
    let mut n = 0usize;
    for document_id in document_ids {
        let stats = retract_document_indexes(graph, vector, None, document_id).await;
        tracing::info!(
            document_id = %document_id,
            embeddings_deleted = stats.embeddings_deleted,
            entities_removed = stats.entities_removed,
            "SPEC-059: retracted indexes for orphaned incomplete document"
        );
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage, VectorStorage};

    #[test]
    fn post_graph_stages_include_merging_embedding() {
        assert!(is_post_graph_incomplete_stage("merging"));
        assert!(is_post_graph_incomplete_stage("embedding"));
        assert!(!is_post_graph_incomplete_stage("uploading"));
        assert!(!is_post_graph_incomplete_stage("converting"));
    }

    #[tokio::test]
    async fn retract_removes_orphan_chunk_vectors() {
        let prev = std::env::var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER").ok();
        std::env::set_var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER", "1");
        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("orphan-retract"));
        let vector: Arc<dyn VectorStorage> =
            Arc::new(MemoryVectorStorage::new("orphan-retract", 4));
        graph.initialize().await.unwrap();
        vector.initialize().await.unwrap();
        let doc = "orphan-doc-1";
        vector
            .upsert(&[(
                format!("{doc}-chunk-0"),
                vec![0.1, 0.2, 0.3, 0.4],
                serde_json::json!({"type": "chunk", "document_id": doc}),
            )])
            .await
            .unwrap();
        let n = retract_indexes_for_orphan_docs(&graph, &vector, &[doc.to_string()]).await;
        assert_eq!(n, 1);
        assert!(vector
            .get_by_id(&format!("{doc}-chunk-0"))
            .await
            .unwrap()
            .is_none());
        match prev {
            Some(v) => std::env::set_var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER", v),
            None => std::env::remove_var("EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER"),
        }
    }
}
