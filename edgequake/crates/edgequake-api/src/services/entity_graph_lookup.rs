//! Entity graph node lookup SSOT (SPEC-027 IMP-026 ascending-compat).
//!
//! Resolves user-supplied identifiers (plain names or full `{workspace}::NAME`
//! ids) to graph nodes via [`EntityId::exact_lookup_candidates`].

use edgequake_storage::traits::GraphStorageReadOps;
use edgequake_storage::{EntityId, GraphNode};

use crate::error::{ApiError, ApiResult};
use crate::handlers::isolation::properties_match_tenant_context;
use crate::middleware::TenantContext;
use crate::services::entity_name_normalize::normalize_entity_name;

/// Ordered graph keys to attempt for a path/query entity identifier.
///
/// Same semantics as `resolve_entity_node_exact`: raw → workspace-scoped →
/// legacy bare. Never normalizes a whole `{uuid}::NAME` string.
pub fn entity_lookup_candidates(raw: &str, workspace_id: Option<&str>) -> Vec<String> {
    EntityId::exact_lookup_candidates(raw, workspace_id)
}

/// Load a graph node by user-supplied id/name with tenant isolation (404 on cross-tenant).
pub async fn lookup_entity_node_for_context(
    graph: &dyn GraphStorageReadOps,
    raw_entity_id: &str,
    ctx: &TenantContext,
) -> ApiResult<GraphNode> {
    let candidates = entity_lookup_candidates(raw_entity_id, ctx.workspace_id.as_deref());
    if candidates.is_empty() {
        return Err(ApiError::BadRequest("Entity identifier required".into()));
    }

    for key in &candidates {
        let Some(node) = graph.get_node(key).await? else {
            continue;
        };
        if properties_match_tenant_context(&node.properties, ctx) {
            return Ok(node);
        }
        return Err(ApiError::NotFound(format!(
            "Entity '{}' not found",
            raw_entity_id
        )));
    }

    let normalized = normalize_entity_name(raw_entity_id.trim());
    Err(ApiError::NotFound(format!(
        "Entity '{}' not found (normalized: '{}'). \
         Accepts normalized entity names (UPPERCASE_WITH_UNDERSCORES) or raw graph node ids.",
        raw_entity_id, normalized
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_add_workspace_scope_for_plain_names() {
        let ws = "55a78878-1b2c-4d3e-8f90-0123456789ab";
        let c = entity_lookup_candidates("Marie Curie", Some(ws));
        assert_eq!(
            c,
            vec![
                "Marie Curie".to_string(),
                format!("{ws}::MARIE_CURIE"),
                "MARIE_CURIE".to_string()
            ]
        );
    }

    #[test]
    fn candidates_keep_full_scoped_id_verbatim() {
        let id = "55a78878-1b2c-4d3e-8f90-0123456789ab::MARIE_CURIE";
        let c = entity_lookup_candidates(id, Some("55a78878-1b2c-4d3e-8f90-0123456789ab"));
        assert_eq!(c[0], id, "full id tried first, UUID not uppercased");
        assert!(!c.iter().any(|k| k.contains("55A78878")));
    }

    #[test]
    fn candidates_without_workspace_fall_back_to_bare() {
        let c = entity_lookup_candidates("MACHINE_LEARNING", None);
        assert_eq!(c, vec!["MACHINE_LEARNING"]);
    }

    /// F3: lineage/provenance must resolve a plain name to the scoped node.
    #[tokio::test]
    async fn lookup_resolves_plain_name_to_scoped_node() {
        use edgequake_storage::traits::GraphStorageMutateOps;
        const WS: &str = "79d6e213-032d-402c-9325-aee3483d3185";
        const TENANT: &str = "00000000-0000-0000-0000-000000000002";
        let graph = edgequake_storage::MemoryGraphStorage::new("test");
        let mut props = std::collections::HashMap::new();
        props.insert("tenant_id".to_string(), TENANT.into());
        props.insert("workspace_id".to_string(), WS.into());
        graph
            .upsert_node(&format!("{WS}::MARIE_CURIE"), props)
            .await
            .unwrap();
        let ctx = TenantContext {
            tenant_id: Some(TENANT.into()),
            workspace_id: Some(WS.into()),
            user_id: None,
        };
        for form in ["MARIE_CURIE", "Marie Curie", &format!("{WS}::MARIE_CURIE")] {
            let node = lookup_entity_node_for_context(&graph, form, &ctx)
                .await
                .unwrap_or_else(|e| panic!("{form}: {e}"));
            assert_eq!(node.id, format!("{WS}::MARIE_CURIE"));
        }
    }
}
