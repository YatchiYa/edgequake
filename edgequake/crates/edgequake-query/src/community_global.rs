//! Global query community expansion (SPEC-023 I6, SPEC-025 6.3 / SPEC-046 EQ-046-11).
//!
//! Uses index-time `community_id` labels — no Louvain at query time.
//! Resolves co-community nodes via push-down `list_nodes_filtered`, not popular scan.
//!
//! When `EDGEQUAKE_COMMUNITY_REPORTS=true`, also injects extractive
//! `community_report` node properties as thematic context chunks (L3).

use std::collections::HashSet;

use edgequake_storage::community_persist::community_features_enabled;
use edgequake_storage::community_reports::{
    community_report_vector_id, community_reports_enabled,
};
use edgequake_storage::traits::{GraphReadView, NodeListFilter};

use crate::context::{QueryContext, RetrievedChunk};
use crate::engine_impl::QueryEngineConfig;
use crate::error::Result;
use crate::helpers::build_entity_from_node;

/// Expand global context with co-community entities (same `community_id` property).
pub async fn expand_global_context_with_communities(
    config: &QueryEngineConfig,
    context: &mut QueryContext,
    entity_ids: &mut Vec<String>,
    graph: GraphReadView<'_>,
    tenant_id: Option<String>,
    workspace_id: Option<String>,
) -> Result<()> {
    if !config.enable_community_global || !community_features_enabled() || entity_ids.is_empty() {
        return Ok(());
    }

    let nodes_map = graph.get_nodes_batch(entity_ids).await?;
    let seed_communities: HashSet<u64> = nodes_map
        .values()
        .filter_map(|n| n.properties.get("community_id").and_then(|v| v.as_u64()))
        .collect();

    if seed_communities.is_empty() {
        return Ok(());
    }

    let filter = NodeListFilter {
        tenant_id,
        workspace_id,
        community_ids: Some(seed_communities.into_iter().collect()),
        ..Default::default()
    };

    let limit = config.max_entities.saturating_mul(2).max(1);
    let page = graph.list_nodes_filtered(&filter, 0, limit).await?;

    let page_ids: Vec<String> = page.items.iter().map(|n| n.id.clone()).collect();
    let degrees: std::collections::HashMap<String, usize> = graph
        .node_degrees_batch(&page_ids)
        .await?
        .into_iter()
        .collect();

    let mut seen: HashSet<String> = context.entities.iter().map(|e| e.name.clone()).collect();

    let mut report_cids: HashSet<u64> = HashSet::new();
    let reports_on = community_reports_enabled();

    for node in page.items {
        if reports_on {
            if let (Some(cid), Some(report)) = (
                node.properties
                    .get("community_id")
                    .and_then(|v| v.as_u64()),
                node.properties
                    .get("community_report")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty()),
            ) {
                if report_cids.insert(cid) {
                    let chunk = RetrievedChunk::new(
                        community_report_vector_id(cid as usize),
                        report,
                        0.55,
                    );
                    context.add_chunk(chunk);
                }
            }
        }
        if !seen.insert(node.id.clone()) {
            continue;
        }
        let degree = degrees.get(&node.id).copied().unwrap_or(0);
        let entity = build_entity_from_node(&node.id, &node.properties, degree, 0.5);
        context.add_entity(entity);
        if !entity_ids.contains(&node.id) {
            entity_ids.push(node.id.clone());
        }
        if context.entities.len() >= config.max_entities {
            break;
        }
    }

    Ok(())
}

/// Append community-report vector hits as thematic chunks (SPEC-046 EQ-046-11).
///
/// SOLID: pure merge of already-filtered vector hits — no storage I/O here.
pub fn append_community_report_vector_chunks(
    context: &mut QueryContext,
    report_hits: &[edgequake_storage::VectorSearchResult],
    max_reports: usize,
) {
    if max_reports == 0 || report_hits.is_empty() {
        return;
    }
    let mut seen: HashSet<String> = context.chunks.iter().map(|c| c.id.clone()).collect();
    for hit in report_hits.iter().take(max_reports) {
        if !seen.insert(hit.id.clone()) {
            continue;
        }
        let content = hit
            .metadata
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if content.is_empty() {
            continue;
        }
        context.add_chunk(RetrievedChunk::new(hit.id.clone(), content, hit.score));
    }
}
