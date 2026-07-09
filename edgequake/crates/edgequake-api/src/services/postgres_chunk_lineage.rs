//! Document lineage from PostgreSQL chunk link tables (migration 066).
//!
//! Fallback when AGE graph nodes lack `source_id` / `source_ids` properties
//! (common for documents ingested before graph provenance was wired).

#[cfg(feature = "postgres")]
use std::collections::HashSet;

#[cfg(feature = "postgres")]
use edgequake_storage::traits::GraphStorageReadOps;
#[cfg(feature = "postgres")]
use sqlx::PgPool;

#[cfg(feature = "postgres")]
use crate::error::{ApiError, ApiResult};
#[cfg(feature = "postgres")]
use crate::handlers::lineage_types::{EntitySummaryResponse, RelationshipSummaryResponse};
#[cfg(feature = "postgres")]
use crate::middleware::TenantContext;
#[cfg(feature = "postgres")]
use crate::services::entity_graph_lookup::lookup_entity_node_for_context;

/// Chunk ID prefix pattern for a document (`{document_id}-chunk-`).
pub fn document_chunk_id_pattern(document_id: &str) -> String {
    format!("{document_id}-chunk-%")
}

/// Load document-scoped entities/relationships from `chunk_entity_links` /
/// `chunk_relation_links` when graph source-prefix queries return nothing.
#[cfg(feature = "postgres")]
pub async fn load_document_lineage_from_chunk_links(
    pool: &PgPool,
    tenant_ctx: &TenantContext,
    document_id: &str,
    graph: &dyn GraphStorageReadOps,
) -> ApiResult<(Vec<EntitySummaryResponse>, Vec<RelationshipSummaryResponse>)> {
    let workspace_id = tenant_ctx.workspace_id.as_deref().ok_or_else(|| {
        ApiError::BadRequest("Workspace context required for document lineage".into())
    })?;

    let chunk_pattern = document_chunk_id_pattern(document_id);

    let entity_rows: Vec<(String, Vec<String>)> = sqlx::query_as(
        "SELECT entity_name, array_agg(DISTINCT chunk_id ORDER BY chunk_id) AS source_chunks
         FROM chunk_entity_links
         WHERE workspace_id = $1 AND chunk_id LIKE $2
         GROUP BY entity_name
         ORDER BY entity_name",
    )
    .bind(workspace_id)
    .bind(&chunk_pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(format!("chunk_entity_links query failed: {e}")))?;

    if entity_rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let entity_names: Vec<String> = entity_rows.iter().map(|(n, _)| n.clone()).collect();

    let shared_entities: HashSet<String> = sqlx::query_scalar(
        "SELECT entity_name
         FROM chunk_entity_links
         WHERE workspace_id = $1 AND entity_name = ANY($2)
         GROUP BY entity_name
         HAVING COUNT(DISTINCT split_part(chunk_id, '-chunk-', 1)) > 1",
    )
    .bind(workspace_id)
    .bind(&entity_names)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(format!("shared entity query failed: {e}")))?
    .into_iter()
    .collect();

    let mut entities = Vec::with_capacity(entity_rows.len());
    for (name, source_chunks) in entity_rows {
        let entity_type = match lookup_entity_node_for_context(graph, &name, tenant_ctx).await {
            Ok(node) => node
                .properties
                .get("entity_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            Err(_) => sqlx::query_scalar::<_, String>(
                "SELECT entity_type FROM entities
                     WHERE workspace_id = $1::uuid AND name = $2
                     LIMIT 1",
            )
            .bind(workspace_id)
            .bind(&name)
            .fetch_optional(pool)
            .await
            .map_err(|e| ApiError::Internal(format!("entities lookup failed: {e}")))?
            .unwrap_or_else(|| "unknown".to_string()),
        };

        entities.push(EntitySummaryResponse {
            name: name.clone(),
            entity_type,
            source_chunks,
            is_shared: shared_entities.contains(&name),
        });
    }

    let relation_rows: Vec<(String, String, Vec<String>)> = sqlx::query_as(
        "SELECT source_entity, target_entity,
                array_agg(DISTINCT chunk_id ORDER BY chunk_id) AS source_chunks
         FROM chunk_relation_links
         WHERE workspace_id = $1 AND chunk_id LIKE $2
         GROUP BY source_entity, target_entity
         ORDER BY source_entity, target_entity",
    )
    .bind(workspace_id)
    .bind(&chunk_pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Internal(format!("chunk_relation_links query failed: {e}")))?;

    let mut relationships = Vec::with_capacity(relation_rows.len());
    for (source, target, source_chunks) in relation_rows {
        let keywords = graph
            .get_edge(&source, &target)
            .await
            .ok()
            .flatten()
            .and_then(|edge| {
                edge.properties
                    .get("keywords")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        relationships.push(RelationshipSummaryResponse {
            source,
            target,
            keywords,
            source_chunks,
        });
    }

    Ok((entities, relationships))
}

#[cfg(test)]
mod tests {
    use super::document_chunk_id_pattern;

    #[test]
    fn chunk_pattern_matches_document_chunks() {
        let pattern = document_chunk_id_pattern("019f419f-e29e-7cc9-ae40-cb6b28286f45");
        assert_eq!(pattern, "019f419f-e29e-7cc9-ae40-cb6b28286f45-chunk-%");
    }
}
