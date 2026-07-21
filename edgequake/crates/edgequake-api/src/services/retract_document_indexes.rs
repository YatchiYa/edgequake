//! SPEC-058: retract searchable indexes for a document (cancel / saga SSOT).
//!
//! DRY: delete and cancel-before-completed share this helper so cancelled
//! content never remains ANN/graph-searchable. Does **not** delete document
//! metadata KV (status sync owns that).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use edgequake_storage::traits::{GraphStorage, VectorStorage};

use crate::middleware::TenantContext;
use crate::services::document_graph_cascade::{
    cascade_remove_document_sources, CascadeStats, DocumentSourceScope,
};

static RETRACT_ON_CANCEL_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Process-local count of cancel/retract unindex operations.
pub fn retract_on_cancel_total() -> u64 {
    RETRACT_ON_CANCEL_TOTAL.load(Ordering::Relaxed)
}

#[cfg(test)]
pub fn reset_retract_on_cancel_total_for_tests() {
    RETRACT_ON_CANCEL_TOTAL.store(0, Ordering::Relaxed);
}

/// Remove chunk/entity vectors and prune graph sources for `document_id`.
///
/// Best-effort: vector wipe and graph cascade errors are logged; never panics.
pub async fn retract_document_indexes(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    tenant_ctx: Option<&TenantContext>,
    document_id: &str,
) -> CascadeStats {
    RETRACT_ON_CANCEL_TOTAL.fetch_add(1, Ordering::Relaxed);
    edgequake_observability::record_retract_on_cancel();

    let mut stats = CascadeStats::default();

    match vector.delete_by_document(document_id).await {
        Ok(n) => {
            stats.embeddings_deleted += n;
            tracing::info!(
                document_id = %document_id,
                vectors_deleted = n,
                "SPEC-058: retract_document_indexes deleted vectors by document"
            );
        }
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "SPEC-058: delete_by_document failed during retract (continuing to graph cascade)"
            );
        }
    }

    let scope = DocumentSourceScope::from_document_id(document_id);
    match cascade_remove_document_sources(graph, Some(vector), tenant_ctx, &scope).await {
        Ok(cascade) => {
            stats.entities_removed += cascade.entities_removed;
            stats.entities_updated += cascade.entities_updated;
            stats.relationships_removed += cascade.relationships_removed;
            stats.relationships_updated += cascade.relationships_updated;
            stats.embeddings_deleted += cascade.embeddings_deleted;
        }
        Err(e) => {
            tracing::warn!(
                document_id = %document_id,
                error = %e,
                "SPEC-058: graph cascade failed during retract (best-effort)"
            );
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::{
        GraphStorageMutateOps, GraphStorageReadOps, MemoryGraphStorage, MemoryVectorStorage,
        VectorStorage,
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn retract_removes_chunk_vectors_and_sole_source_nodes() {
        reset_retract_on_cancel_total_for_tests();
        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("retract"));
        let vector: Arc<dyn VectorStorage> = Arc::new(MemoryVectorStorage::new("retract", 4));
        graph.initialize().await.unwrap();
        vector.initialize().await.unwrap();

        let doc = "doc-retract-1";
        vector
            .upsert(&[(
                format!("{doc}-chunk-0"),
                vec![0.1; 4],
                serde_json::json!({"type": "chunk", "document_id": doc}),
            )])
            .await
            .unwrap();

        graph
            .upsert_node(
                "ONLY_IN_DOC",
                HashMap::from([
                    ("entity_type".into(), serde_json::json!("PERSON")),
                    ("source_ids".into(), serde_json::json!([doc])),
                ]),
            )
            .await
            .unwrap();

        let stats = retract_document_indexes(&graph, &vector, None, doc).await;
        assert!(stats.embeddings_deleted > 0 || stats.entities_removed > 0);
        assert!(vector
            .get_by_id(&format!("{doc}-chunk-0"))
            .await
            .unwrap()
            .is_none());
        assert!(graph.get_node("ONLY_IN_DOC").await.unwrap().is_none());
        assert!(retract_on_cancel_total() >= 1);
    }

    #[tokio::test]
    async fn retract_preserves_shared_entity_with_other_sources() {
        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("retract-share"));
        let vector: Arc<dyn VectorStorage> = Arc::new(MemoryVectorStorage::new("retract-share", 4));
        graph.initialize().await.unwrap();
        vector.initialize().await.unwrap();

        graph
            .upsert_node(
                "SHARED",
                HashMap::from([
                    ("entity_type".into(), serde_json::json!("PERSON")),
                    ("source_ids".into(), serde_json::json!(["doc-a", "doc-b"])),
                ]),
            )
            .await
            .unwrap();

        let _ = retract_document_indexes(&graph, &vector, None, "doc-b").await;
        let node = graph.get_node("SHARED").await.unwrap().expect("kept");
        let sources = node
            .properties
            .get("source_ids")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].as_str(), Some("doc-a"));
    }
}
