//! Document-scoped graph cascade — SPEC-006 P1 (SRP/DRY).
//!
//! Bounded graph mutations and lineage reads keyed by document source prefixes.
//! Never loads the full workspace graph into handler memory.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use edgequake_storage::traits::{
    collect_source_references, EdgeListFilter, GraphEdge, GraphNode, GraphStorage, NodeListFilter,
    VectorStorage,
};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;

/// Document source scope for bounded graph operations.
#[derive(Debug, Clone)]
pub struct DocumentSourceScope {
    pub document_id: String,
    pub key_prefix: String,
    pub source_prefixes: Vec<String>,
}

impl DocumentSourceScope {
    pub fn from_document_id(document_id: impl Into<String>) -> Self {
        let document_id = document_id.into();
        Self {
            key_prefix: document_id.clone(),
            source_prefixes: vec![document_id.clone()],
            document_id,
        }
    }

    pub fn with_key_prefix(document_id: String, key_prefix: String) -> Self {
        let source_prefixes = if key_prefix != document_id {
            vec![key_prefix.clone(), document_id.clone()]
        } else {
            vec![document_id.clone()]
        };
        Self {
            document_id,
            key_prefix,
            source_prefixes,
        }
    }

    pub fn chunk_prefix(&self) -> String {
        format!("{}-chunk-", self.key_prefix)
    }
}

/// Statistics from cascade remove or impact analysis.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CascadeStats {
    pub entities_removed: usize,
    pub entities_updated: usize,
    pub relationships_removed: usize,
    pub relationships_updated: usize,
    pub embeddings_deleted: usize,
}

pub fn node_list_filter(tenant_ctx: Option<&TenantContext>) -> NodeListFilter {
    match tenant_ctx {
        Some(ctx) => NodeListFilter {
            tenant_id: ctx.tenant_id.clone(),
            workspace_id: ctx.workspace_id.clone(),
            entity_type: None,
            search: None,
            community_ids: None,
        },
        None => NodeListFilter::default(),
    }
}

/// Discovery filter for document cascade / deletion-impact (issue #305).
///
/// WHY workspace-only (no `tenant_id`): source prefixes already bound the
/// document set. Requiring `tenant_id` equality excludes legacy AGE nodes that
/// lack tenant props — impact preview used unfiltered `None` and disagreed
/// with the deletion worker, leaving orphan KG entities after doc delete.
pub fn node_list_filter_for_document_scope(tenant_ctx: Option<&TenantContext>) -> NodeListFilter {
    match tenant_ctx {
        Some(ctx) => NodeListFilter {
            tenant_id: None,
            workspace_id: ctx.workspace_id.clone(),
            entity_type: None,
            search: None,
            community_ids: None,
        },
        None => NodeListFilter::default(),
    }
}

pub fn edge_list_filter(tenant_ctx: Option<&TenantContext>) -> EdgeListFilter {
    match tenant_ctx {
        Some(ctx) => EdgeListFilter {
            tenant_id: ctx.tenant_id.clone(),
            workspace_id: ctx.workspace_id.clone(),
            relationship_type: None,
        },
        None => EdgeListFilter::default(),
    }
}

/// Edge discovery filter aligned with [`node_list_filter_for_document_scope`].
pub fn edge_list_filter_for_document_scope(tenant_ctx: Option<&TenantContext>) -> EdgeListFilter {
    match tenant_ctx {
        Some(ctx) => EdgeListFilter {
            tenant_id: None,
            workspace_id: ctx.workspace_id.clone(),
            relationship_type: None,
        },
        None => EdgeListFilter::default(),
    }
}

pub fn source_belongs_to_document(source: &str, scope: &DocumentSourceScope) -> bool {
    scope.source_prefixes.iter().any(|p| {
        source.starts_with(p.as_str())
            || source.starts_with(&edgequake_storage::kv_keys::doc_chunk_prefix(p))
            || source == p.as_str()
    })
}

pub fn remaining_sources_after_removal(
    properties: &HashMap<String, serde_json::Value>,
    scope: &DocumentSourceScope,
) -> Vec<String> {
    collect_source_references(properties)
        .into_iter()
        .filter(|s| !source_belongs_to_document(s, scope))
        .collect()
}

pub fn sources_for_document(
    properties: &HashMap<String, serde_json::Value>,
    scope: &DocumentSourceScope,
) -> Vec<String> {
    collect_source_references(properties)
        .into_iter()
        .filter(|s| source_belongs_to_document(s, scope))
        .collect()
}

/// Find nodes whose sources reference this document (bounded push-down).
pub async fn find_document_nodes(
    graph: &Arc<dyn GraphStorage>,
    tenant_ctx: Option<&TenantContext>,
    scope: &DocumentSourceScope,
) -> ApiResult<Vec<GraphNode>> {
    let filter = node_list_filter_for_document_scope(tenant_ctx);
    graph
        .find_nodes_by_source_prefixes(&filter, &scope.source_prefixes)
        .await
        .map_err(ApiError::from)
}

/// Find edges whose sources reference this document (bounded push-down).
pub async fn find_document_edges(
    graph: &Arc<dyn GraphStorage>,
    tenant_ctx: Option<&TenantContext>,
    scope: &DocumentSourceScope,
) -> ApiResult<Vec<GraphEdge>> {
    let filter = edge_list_filter_for_document_scope(tenant_ctx);
    graph
        .find_edges_by_source_prefixes(&filter, &scope.source_prefixes)
        .await
        .map_err(ApiError::from)
}

fn edge_key(edge: &GraphEdge) -> (String, String) {
    (edge.source.clone(), edge.target.clone())
}

/// Cascade remove document sources from graph entities and relationships.
///
/// # First principles (reliable delete)
///
/// - Collect exclusive vs shared entities in one pass, then **batch** mutate:
///   `delete_nodes_batch` (DETACH removes incident edges) + `upsert_nodes_batch`.
/// - Edge membership uses the in-memory `deleted_node_ids` set — never N×`has_node`.
/// - Edges touching a deleted node are already gone via DETACH; only surviving
///   endpoints need `delete_edges_batch` / `upsert_edges_batch`.
/// - Exclusive entity embeddings: one `delete_entities_batch` (not per-entity).
pub async fn cascade_remove_document_sources(
    graph: &Arc<dyn GraphStorage>,
    vector_storage: Option<&Arc<dyn VectorStorage>>,
    tenant_ctx: Option<&TenantContext>,
    scope: &DocumentSourceScope,
) -> ApiResult<CascadeStats> {
    let mut stats = CascadeStats::default();
    let affected_nodes = find_document_nodes(graph, tenant_ctx, scope).await?;

    let mut node_ids_to_delete: Vec<String> = Vec::new();
    let mut nodes_to_update: Vec<(String, HashMap<String, serde_json::Value>)> = Vec::new();

    for node in affected_nodes {
        let sources = collect_source_references(&node.properties);
        if sources.is_empty() {
            continue;
        }
        let remaining = remaining_sources_after_removal(&node.properties, scope);
        if remaining.is_empty() {
            node_ids_to_delete.push(node.id);
            stats.entities_removed += 1;
        } else if remaining.len() < sources.len() {
            let mut updated_props = node.properties.clone();
            // SPEC-046 EQ-046-12: rebuild description + align source_ids
            crate::services::knowledge_rebuild::apply_rebuild_to_properties(
                &mut updated_props,
                &remaining,
            );
            nodes_to_update.push((node.id, updated_props));
            stats.entities_updated += 1;
        }
    }

    let deleted_node_ids: HashSet<String> = node_ids_to_delete.iter().cloned().collect();

    // Snapshot edges BEFORE DETACH so we can count incident edges and update
    // surviving shared relationships (find after delete would miss DETACH'd rows).
    let mut edges_to_process: HashMap<(String, String), GraphEdge> = HashMap::new();
    for edge in find_document_edges(graph, tenant_ctx, scope).await? {
        edges_to_process.insert(edge_key(&edge), edge);
    }
    if !deleted_node_ids.is_empty() {
        let ids: Vec<String> = deleted_node_ids.iter().cloned().collect();
        for edge in graph
            .get_edges_for_nodes_batch(&ids)
            .await
            .map_err(ApiError::from)?
        {
            edges_to_process.insert(edge_key(&edge), edge);
        }
    }

    if !node_ids_to_delete.is_empty() {
        graph
            .delete_nodes_batch(&node_ids_to_delete)
            .await
            .map_err(ApiError::from)?;
        if let Some(vs) = vector_storage {
            match vs.delete_entities_batch(&node_ids_to_delete).await {
                Ok(n) => stats.embeddings_deleted += n,
                Err(e) => {
                    tracing::warn!(
                        document_id = %scope.document_id,
                        error = %e,
                        "Batch entity embedding delete failed (non-fatal)"
                    );
                }
            }
        }
    }

    if !nodes_to_update.is_empty() {
        graph
            .upsert_nodes_batch(&nodes_to_update)
            .await
            .map_err(ApiError::from)?;
    }

    let mut edges_to_delete: Vec<(String, String)> = Vec::new();
    let mut edges_to_update: Vec<(String, String, HashMap<String, serde_json::Value>)> = Vec::new();

    for edge in edges_to_process.into_values() {
        let source_deleted = deleted_node_ids.contains(&edge.source);
        let target_deleted = deleted_node_ids.contains(&edge.target);
        if source_deleted || target_deleted {
            // DETACH already removed the edge; count for stats parity with the
            // historical path that explicitly deleted dangling edges.
            stats.relationships_removed += 1;
            continue;
        }

        let sources = collect_source_references(&edge.properties);
        if sources.is_empty() {
            continue;
        }
        let remaining = remaining_sources_after_removal(&edge.properties, scope);
        if remaining.is_empty() {
            edges_to_delete.push((edge.source, edge.target));
            stats.relationships_removed += 1;
        } else if remaining.len() < sources.len() {
            let mut updated_props = edge.properties.clone();
            crate::services::knowledge_rebuild::apply_rebuild_to_properties(
                &mut updated_props,
                &remaining,
            );
            edges_to_update.push((edge.source, edge.target, updated_props));
            stats.relationships_updated += 1;
        }
    }

    if !edges_to_delete.is_empty() {
        graph
            .delete_edges_batch(&edges_to_delete)
            .await
            .map_err(ApiError::from)?;
    }

    if !edges_to_update.is_empty() {
        graph
            .upsert_edges_batch(&edges_to_update)
            .await
            .map_err(ApiError::from)?;
    }

    Ok(stats)
}

/// Read-only impact preview (same bounded scope as cascade).
pub async fn analyze_deletion_impact_stats(
    graph: &Arc<dyn GraphStorage>,
    tenant_ctx: Option<&TenantContext>,
    scope: &DocumentSourceScope,
) -> ApiResult<CascadeStats> {
    let mut stats = CascadeStats::default();

    for node in find_document_nodes(graph, tenant_ctx, scope).await? {
        let sources = collect_source_references(&node.properties);
        if sources.is_empty() {
            continue;
        }
        let remaining = remaining_sources_after_removal(&node.properties, scope);
        if remaining.is_empty() {
            stats.entities_removed += 1;
        } else if remaining.len() < sources.len() {
            stats.entities_updated += 1;
        }
    }

    for edge in find_document_edges(graph, tenant_ctx, scope).await? {
        let sources = collect_source_references(&edge.properties);
        if sources.is_empty() {
            continue;
        }
        let remaining = remaining_sources_after_removal(&edge.properties, scope);
        if remaining.is_empty() {
            stats.relationships_removed += 1;
        } else if remaining.len() < sources.len() {
            stats.relationships_updated += 1;
        }
    }

    Ok(stats)
}

/// Collect graph edges attributable to a document for lineage visualization.
///
/// First principle: if both endpoints belong to the document's entity set, the
/// relationship is document-scoped — even when edge properties lack `source_ids`
/// after graph merge (nodes retain provenance; edges often do not).
pub async fn find_relationships_for_document_lineage(
    graph: &Arc<dyn GraphStorage>,
    tenant_ctx: Option<&TenantContext>,
    scope: &DocumentSourceScope,
    document_entity_ids: &[String],
) -> ApiResult<Vec<GraphEdge>> {
    if document_entity_ids.is_empty() {
        return Ok(Vec::new());
    }

    let entity_set: HashSet<&str> = document_entity_ids.iter().map(String::as_str).collect();
    let mut edges: HashMap<(String, String), GraphEdge> = HashMap::new();

    for edge in graph
        .get_edges_for_nodes_batch(document_entity_ids)
        .await
        .map_err(ApiError::from)?
    {
        if entity_set.contains(edge.source.as_str()) && entity_set.contains(edge.target.as_str()) {
            edges.insert(edge_key(&edge), edge);
        }
    }

    // Merge source-prefix hits (may carry richer chunk provenance on edge props).
    for edge in find_document_edges(graph, tenant_ctx, scope).await? {
        if sources_for_document(&edge.properties, scope).is_empty() {
            continue;
        }
        edges.entry(edge_key(&edge)).or_insert(edge);
    }

    Ok(edges.into_values().collect())
}

/// Statistics from document graph data cleanup.
#[derive(Debug, Default, Clone)]
pub struct CleanupStats {
    pub entities_removed: usize,
    pub entities_updated: usize,
    pub relationships_removed: usize,
    pub relationships_updated: usize,
    pub embeddings_deleted: usize,
}

/// Clean up graph data for a document without deleting KV entries.
pub async fn cleanup_document_graph_data(
    document_id: &str,
    graph_storage: &Arc<dyn GraphStorage>,
    vector_storage: Option<&Arc<dyn VectorStorage>>,
) -> ApiResult<CleanupStats> {
    let scope = DocumentSourceScope::from_document_id(document_id);
    let cascade_stats =
        cascade_remove_document_sources(graph_storage, vector_storage, None, &scope).await?;

    tracing::info!(
        document_id = %document_id,
        entities_removed = cascade_stats.entities_removed,
        entities_updated = cascade_stats.entities_updated,
        relationships_removed = cascade_stats.relationships_removed,
        relationships_updated = cascade_stats.relationships_updated,
        embeddings_deleted = cascade_stats.embeddings_deleted,
        "Document graph data cleanup completed"
    );

    Ok(CleanupStats {
        entities_removed: cascade_stats.entities_removed,
        entities_updated: cascade_stats.entities_updated,
        relationships_removed: cascade_stats.relationships_removed,
        relationships_updated: cascade_stats.relationships_updated,
        embeddings_deleted: cascade_stats.embeddings_deleted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_discovery_filter_is_workspace_only() {
        let ctx = TenantContext {
            tenant_id: Some("tenant-a".into()),
            workspace_id: Some("ws-a".into()),
            user_id: None,
        };
        let filter = node_list_filter_for_document_scope(Some(&ctx));
        assert!(filter.tenant_id.is_none());
        assert_eq!(filter.workspace_id.as_deref(), Some("ws-a"));
        let edge = edge_list_filter_for_document_scope(Some(&ctx));
        assert!(edge.tenant_id.is_none());
        assert_eq!(edge.workspace_id.as_deref(), Some("ws-a"));
    }

    #[test]
    fn source_belongs_matches_chunk_and_doc_id() {
        let scope = DocumentSourceScope::from_document_id("doc-abc");
        assert!(source_belongs_to_document("doc-abc", &scope));
        assert!(source_belongs_to_document("doc-abc-chunk-0", &scope));
        assert!(!source_belongs_to_document("other-doc", &scope));
    }

    #[test]
    fn remaining_sources_filters_document_refs() {
        let scope = DocumentSourceScope::from_document_id("doc-1");
        let mut props = HashMap::new();
        props.insert(
            "source_ids".to_string(),
            serde_json::json!(["doc-1-chunk-0", "doc-2-chunk-1"]),
        );
        let remaining = remaining_sources_after_removal(&props, &scope);
        assert_eq!(remaining, vec!["doc-2-chunk-1"]);
    }

    #[test]
    fn legacy_pipe_source_id_matches_document_scope() {
        let scope = DocumentSourceScope::from_document_id("doc-legacy");
        let mut props = HashMap::new();
        props.insert(
            "source_id".to_string(),
            serde_json::json!("doc-legacy-chunk-0|other-doc-chunk-1"),
        );
        let remaining = remaining_sources_after_removal(&props, &scope);
        assert_eq!(remaining, vec!["other-doc-chunk-1"]);
    }

    #[test]
    fn key_prefix_scope_includes_both_prefixes() {
        let scope = DocumentSourceScope::with_key_prefix(
            "doc-uuid".to_string(),
            "kv-key-prefix".to_string(),
        );
        assert_eq!(scope.source_prefixes.len(), 2);
        assert!(source_belongs_to_document("kv-key-prefix-chunk-0", &scope));
        assert!(source_belongs_to_document("doc-uuid-chunk-1", &scope));
    }

    #[tokio::test]
    async fn batch_cascade_removes_exclusive_and_rebuilds_shared() {
        use edgequake_storage::{MemoryGraphStorage, MemoryVectorStorage};

        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("cascade-batch"));
        let vectors: Arc<dyn VectorStorage> =
            Arc::new(MemoryVectorStorage::new("cascade-batch", 8));
        let doc_a = "doc-a";
        let doc_b = "doc-b";
        let scope = DocumentSourceScope::from_document_id(doc_a);

        let mut exclusive = HashMap::new();
        exclusive.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("{doc_a}-chunk-0")]),
        );
        graph
            .upsert_node("EXCLUSIVE", exclusive)
            .await
            .expect("exclusive");

        let mut shared = HashMap::new();
        shared.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("{doc_a}-chunk-0"), format!("{doc_b}-chunk-0")]),
        );
        graph.upsert_node("SHARED", shared).await.expect("shared");

        let mut edge_props = HashMap::new();
        edge_props.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("{doc_a}-chunk-0")]),
        );
        graph
            .upsert_edge("EXCLUSIVE", "SHARED", edge_props)
            .await
            .expect("edge");

        let stats = cascade_remove_document_sources(&graph, Some(&vectors), None, &scope)
            .await
            .expect("cascade");

        assert_eq!(stats.entities_removed, 1);
        assert_eq!(stats.entities_updated, 1);
        assert!(stats.relationships_removed >= 1);
        assert!(!graph.has_node("EXCLUSIVE").await.unwrap());
        assert!(graph.has_node("SHARED").await.unwrap());
        let shared_node = graph.get_node("SHARED").await.unwrap().unwrap();
        let remaining = remaining_sources_after_removal(&shared_node.properties, &scope);
        assert_eq!(remaining, vec![format!("{doc_b}-chunk-0")]);
    }

    #[tokio::test]
    async fn lineage_relationships_include_entity_adjacency_without_edge_source_ids() {
        use edgequake_storage::MemoryGraphStorage;

        let graph: Arc<dyn GraphStorage> = Arc::new(MemoryGraphStorage::new("lineage-test"));
        let doc_id = "doc-lightrag";
        let scope = DocumentSourceScope::from_document_id(doc_id);

        let mut alice_props = HashMap::new();
        alice_props.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("{doc_id}-chunk-0")]),
        );
        alice_props.insert("entity_type".to_string(), serde_json::json!("concept"));
        graph
            .upsert_node("LIGHTRAG", alice_props)
            .await
            .expect("alice");

        let mut bob_props = HashMap::new();
        bob_props.insert(
            "source_ids".to_string(),
            serde_json::json!([format!("{doc_id}-chunk-0")]),
        );
        bob_props.insert("entity_type".to_string(), serde_json::json!("technology"));
        graph
            .upsert_node("RETRIEVAL", bob_props)
            .await
            .expect("bob");

        let mut edge_props = HashMap::new();
        edge_props.insert("keywords".to_string(), serde_json::json!("uses"));
        graph
            .upsert_edge("LIGHTRAG", "RETRIEVAL", edge_props)
            .await
            .expect("edge");

        let entity_ids = vec!["LIGHTRAG".to_string(), "RETRIEVAL".to_string()];
        let edges = find_relationships_for_document_lineage(&graph, None, &scope, &entity_ids)
            .await
            .expect("lineage edges");

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, "LIGHTRAG");
        assert_eq!(edges[0].target, "RETRIEVAL");
    }
}
