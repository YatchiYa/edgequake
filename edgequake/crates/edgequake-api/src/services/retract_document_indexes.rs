//! SPEC-058: retract searchable indexes for a document (cancel / saga SSOT).
//!
//! DRY: delete and cancel-before-completed share this helper so cancelled
//! content never remains ANN/graph-searchable. Does **not** delete document
//! metadata KV (status sync owns that).
//!
//! SPEC-119: `retract_document_indexes_checked` fails closed on graph cascade
//! discovery timeouts (reprocess path). Best-effort `retract_document_indexes`
//! remains for cancel/orphan recovery.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use edgequake_storage::traits::{GraphStorage, VectorStorage};

use crate::error::ApiResult;
use crate::middleware::TenantContext;
use crate::services::document_graph_cascade::{
    cascade_remove_document_sources, CascadeStats, DocumentSourceScope,
};
use crate::services::graph_cleanup_timeout::{
    is_source_discovery_timeout, log_graph_cleanup_timeout, map_cascade_error_for_reprocess,
    GraphCleanupAction,
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

async fn wipe_vectors(
    vector: &Arc<dyn VectorStorage>,
    document_id: &str,
    stats: &mut CascadeStats,
) {
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
}

fn merge_cascade(stats: &mut CascadeStats, cascade: CascadeStats) {
    stats.entities_removed += cascade.entities_removed;
    stats.entities_updated += cascade.entities_updated;
    stats.relationships_removed += cascade.relationships_removed;
    stats.relationships_updated += cascade.relationships_updated;
    stats.embeddings_deleted += cascade.embeddings_deleted;
}

/// Remove chunk/entity vectors and prune graph sources for `document_id`.
///
/// Best-effort: vector wipe and graph cascade errors are logged; never panics.
/// Prefer [`retract_document_indexes_checked`] for reprocess admit (SPEC-119).
pub async fn retract_document_indexes(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    tenant_ctx: Option<&TenantContext>,
    document_id: &str,
) -> CascadeStats {
    RETRACT_ON_CANCEL_TOTAL.fetch_add(1, Ordering::Relaxed);
    edgequake_observability::record_retract_on_cancel();

    let mut stats = CascadeStats::default();
    wipe_vectors(vector, document_id, &mut stats).await;

    let scope = DocumentSourceScope::from_document_id(document_id);
    match cascade_remove_document_sources(graph, Some(vector), tenant_ctx, &scope).await {
        Ok(cascade) => merge_cascade(&mut stats, cascade),
        Err(e) => {
            let detail = e.to_string();
            if is_source_discovery_timeout(&detail) {
                log_graph_cleanup_timeout(document_id, GraphCleanupAction::Reprocess, &detail);
            } else {
                tracing::warn!(
                    document_id = %document_id,
                    error = %detail,
                    "SPEC-058: graph cascade failed during retract (best-effort)"
                );
            }
        }
    }

    stats
}

/// SPEC-119: same retract SSOT as [`retract_document_indexes`], but fail-closed
/// on graph cascade errors (mapped to product timeout copy when discovery timed out).
pub async fn retract_document_indexes_checked(
    graph: &Arc<dyn GraphStorage>,
    vector: &Arc<dyn VectorStorage>,
    tenant_ctx: Option<&TenantContext>,
    document_id: &str,
) -> ApiResult<CascadeStats> {
    RETRACT_ON_CANCEL_TOTAL.fetch_add(1, Ordering::Relaxed);
    edgequake_observability::record_retract_on_cancel();

    let mut stats = CascadeStats::default();
    wipe_vectors(vector, document_id, &mut stats).await;

    let scope = DocumentSourceScope::from_document_id(document_id);
    match cascade_remove_document_sources(graph, Some(vector), tenant_ctx, &scope).await {
        Ok(cascade) => {
            merge_cascade(&mut stats, cascade);
            Ok(stats)
        }
        Err(e) => Err(map_cascade_error_for_reprocess(document_id, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgequake_storage::traits::{
        EdgeListFilter, GraphScanOps, GraphStorage, GraphStorageMutateOps,
    };
    use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage, VectorStorage};
    use serde_json::json;
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

    /// SPEC-119 EC-04/EC-10: reprocess retract clears Symptom F singular-only edges.
    #[tokio::test]
    async fn retract_checked_removes_singular_only_citation_edges() {
        reset_retract_on_cancel_total_for_tests();
        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("retract-singular"));
        let vector: Arc<dyn VectorStorage> =
            Arc::new(MemoryVectorStorage::new("retract-singular", 4));
        graph.initialize().await.unwrap();
        vector.initialize().await.unwrap();

        let doc = "doc-singular-retract";
        let chunk = format!("{doc}-chunk-0");
        graph
            .upsert_node(
                "SA",
                HashMap::from([
                    ("entity_type".into(), json!("CONCEPT")),
                    ("tenant_id".into(), json!("t")),
                    ("workspace_id".into(), json!("w")),
                ]),
            )
            .await
            .unwrap();
        graph
            .upsert_node(
                "SB",
                HashMap::from([
                    ("entity_type".into(), json!("CONCEPT")),
                    ("tenant_id".into(), json!("t")),
                    ("workspace_id".into(), json!("w")),
                ]),
            )
            .await
            .unwrap();

        let mut ep = HashMap::new();
        ep.insert("tenant_id".into(), json!("t"));
        ep.insert("workspace_id".into(), json!("w"));
        ep.insert("relation_type".into(), json!("RELATED"));
        ep.insert("source_chunk_id".into(), json!(&chunk));
        ep.insert("source_document_id".into(), json!(doc));
        // Intentionally no source_ids[] — Symptom F shape.
        graph.upsert_edge("SA", "SB", ep).await.unwrap();

        let filter = EdgeListFilter {
            tenant_id: Some("t".into()),
            workspace_id: Some("w".into()),
            relationship_type: None,
        };
        let before = graph
            .find_edges_by_source_prefixes(&filter, &[doc.to_string()])
            .await
            .unwrap();
        assert_eq!(before.len(), 1, "singular probe must find the edge");

        let stats = retract_document_indexes_checked(&graph, &vector, None, doc)
            .await
            .expect("checked retract");
        assert!(
            stats.relationships_removed >= 1,
            "expected singular edge removed, stats={stats:?}"
        );

        let after = graph
            .find_edges_by_source_prefixes(&filter, &[doc.to_string()])
            .await
            .unwrap();
        assert!(
            after.is_empty(),
            "singular citation edges must be gone after reprocess retract"
        );
    }
}
