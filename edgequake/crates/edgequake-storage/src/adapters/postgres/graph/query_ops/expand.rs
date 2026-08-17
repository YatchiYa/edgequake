<<<<<<< HEAD
//! Graph expand / neighbors — variable-length Cypher (SPEC-054).
=======
//! Graph expand / neighbors — native BFS (SPEC-054 / IMP-031-04).
//!
//! First principles: request-path expand must be **O(depth × F log E + K log N)**
//! via indexed incident-edge batches + node batch fetch — never variable-length Cypher.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

use sqlx::Row;

use super::super::helpers::EdgeTenantFilterMode;
use super::super::PostgresAGEGraphStorage;
use crate::error::{Result, StorageError};
<<<<<<< HEAD
use crate::traits::{EdgeListFilter, GraphEdge, GraphNode, KnowledgeGraph, NodeListFilter};

impl PostgresAGEGraphStorage {
=======
use crate::traits::{
    edge_matches_list_filter, node_matches_list_filter, EdgeListFilter, GraphEdge, GraphNode,
    KnowledgeGraph, NodeListFilter,
};
use std::collections::{HashSet, VecDeque};

impl PostgresAGEGraphStorage {
    /**
     * @dataop      DATA-AGE-GRAPH-GET-KNOWLEDGE-GRAPH-038
     * @engine      apache_age (native BFS; IMP-031-04)
     * @intent      Bounded k-hop subgraph from start node; tenant/ws optional.
     * @complexity  time: O(depth × F × log E + K log N); space: O(K + E′)
     * @limits      max_depth / max_nodes hard caps; no unbounded MATCH
     */
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    pub(in crate::adapters::postgres::graph) async fn pg_get_knowledge_graph(
        &self,
        start_node: &str,
        max_depth: usize,
        max_nodes: usize,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<KnowledgeGraph> {
<<<<<<< HEAD
        if let (Some(tenant), Some(workspace)) = (tenant_id, workspace_id) {
            return self
                .pg_get_knowledge_graph_scoped(start_node, max_depth, max_nodes, tenant, workspace)
                .await;
        }

        let escaped_id = Self::escape_cypher_string(start_node);

        // Use AGE's variable-length path traversal
        let cypher = format!(
            "MATCH p = (start:Node {{node_id: '{}'}})-[*0..{}]-(connected) \
             RETURN DISTINCT connected LIMIT {}",
            escaped_id, max_depth, max_nodes
        );

        let rows = self.cypher_query(&cypher, &["connected"]).await?;

        let mut kg = KnowledgeGraph::new();
        let mut node_ids: Vec<String> = Vec::new();

        for row in &rows {
            let json_value: serde_json::Value = row.get("connected");
            let agtype_str = json_value.to_string();
            if let Some(node) = Self::parse_vertex(&agtype_str) {
                node_ids.push(node.id.clone());
                kg.add_node(node);
            }
        }

        // Get edges between discovered nodes
        if !node_ids.is_empty() {
            let ids_list: Vec<String> = node_ids
                .iter()
                .map(|id| format!("'{}'", Self::escape_cypher_string(id)))
                .collect();

            let edges_cypher = format!(
                "MATCH (a:Node)-[r:EDGE]->(b:Node) \
                 WHERE a.node_id IN [{}] AND b.node_id IN [{}] \
                 RETURN r",
                ids_list.join(", "),
                ids_list.join(", ")
            );

            let edge_rows = self.cypher_query(&edges_cypher, &["r"]).await?;

            for row in &edge_rows {
                let json_value: serde_json::Value = row.get("r");
                let agtype_str = json_value.to_string();
                if let Some(edge) = Self::parse_edge(&agtype_str) {
                    kg.add_edge(edge);
                }
            }
        }

        kg.is_truncated = kg.node_count() >= max_nodes;

        Ok(kg)
    }

    /// Tenant-scoped BFS using native SQL edge batch lookups (SPEC-027 IMP-022).
    ///
    /// Requires migration 046 expression indexes for production-scale graphs.
    async fn pg_get_knowledge_graph_scoped(
=======
        // DRY: single native BFS path for scoped and unscoped (filters optional).
        self.pg_bfs_expand(start_node, max_depth, max_nodes, tenant_id, workspace_id)
            .await
    }

    /**
     * @dataop      DATA-AGE-GRAPH-GET-NEIGHBORS-042
     * @engine      apache_age (native BFS; IMP-031-04)
     * @intent      Distinct neighbors within depth 1..3 (excludes start).
     * @complexity  time: O(depth × F log E + K log N); space: O(K)
     * @limits      depth clamped to 3; max 500 neighbors
     */
    pub(in crate::adapters::postgres::graph) async fn pg_get_neighbors(
        &self,
        node_id: &str,
        depth: usize,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<GraphNode>> {
        let safe_depth = depth.clamp(1, 3);
        const MAX_NEIGHBORS: usize = 500;
        let kg = self
            .pg_bfs_expand(
                node_id,
                safe_depth,
                MAX_NEIGHBORS.saturating_add(1),
                tenant_id,
                workspace_id,
            )
            .await?;
        Ok(kg
            .nodes
            .into_iter()
            .filter(|n| n.id != node_id)
            .take(MAX_NEIGHBORS)
            .collect())
    }

    /// Native multi-hop BFS (SSOT for expand + neighbors).
    ///
    /// Per hop:
    /// 1. `pg_get_incident_edges_batch(frontier)` — O(F log E)
    /// 2. Collect neighbor IDs, `pg_get_nodes_batch` once — O(K log N) one RT
    async fn pg_bfs_expand(
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        &self,
        start_node: &str,
        max_depth: usize,
        max_nodes: usize,
<<<<<<< HEAD
        tenant_id: &str,
        workspace_id: &str,
    ) -> Result<KnowledgeGraph> {
        use std::collections::{HashSet, VecDeque};

        use crate::traits::{edge_matches_list_filter, node_matches_list_filter};

        let node_filter = NodeListFilter {
            tenant_id: Some(tenant_id.to_string()),
            workspace_id: Some(workspace_id.to_string()),
            ..Default::default()
        };
        let edge_filter = EdgeListFilter {
            tenant_id: Some(tenant_id.to_string()),
            workspace_id: Some(workspace_id.to_string()),
            relationship_type: None,
        };
=======
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<KnowledgeGraph> {
        let node_filter = NodeListFilter {
            tenant_id: tenant_id.map(str::to_string),
            workspace_id: workspace_id.map(str::to_string),
            ..Default::default()
        };
        let edge_filter = EdgeListFilter {
            tenant_id: tenant_id.map(str::to_string),
            workspace_id: workspace_id.map(str::to_string),
            relationship_type: None,
        };
        let filter_nodes = tenant_id.is_some() || workspace_id.is_some();
        let filter_edges = filter_nodes;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

        let Some(start) = self.pg_get_node(start_node).await? else {
            return Ok(KnowledgeGraph::new());
        };
<<<<<<< HEAD
        if !node_matches_list_filter(&start, &node_filter) {
=======
        if filter_nodes && !node_matches_list_filter(&start, &node_filter) {
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            return Ok(KnowledgeGraph::new());
        }

        let mut kg = KnowledgeGraph::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut frontier: VecDeque<String> = VecDeque::new();
        visited.insert(start.id.clone());
        frontier.push_back(start.id.clone());
        kg.add_node(start);

        for _ in 0..max_depth {
            if frontier.is_empty() || kg.node_count() >= max_nodes {
                break;
            }

            let current_frontier: Vec<String> = frontier.drain(..).collect();
            let frontier_set: HashSet<&str> = current_frontier.iter().map(String::as_str).collect();
            let edges = self
                .pg_get_incident_edges_batch(
                    &current_frontier,
                    edge_filter.tenant_id.as_deref(),
                    edge_filter.workspace_id.as_deref(),
                )
                .await?;

<<<<<<< HEAD
            for edge in edges {
                if !edge_matches_list_filter(&edge, &edge_filter) {
                    continue;
                }

=======
            let mut candidate_ids: Vec<String> = Vec::new();
            let mut candidate_seen: HashSet<String> = HashSet::new();
            for edge in &edges {
                if filter_edges && !edge_matches_list_filter(edge, &edge_filter) {
                    continue;
                }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                for (endpoint, other) in
                    [(&edge.source, &edge.target), (&edge.target, &edge.source)]
                {
                    if !frontier_set.contains(endpoint.as_str()) || visited.contains(other) {
                        continue;
                    }
<<<<<<< HEAD
                    if let Some(node) = self.pg_get_node(other).await? {
                        if node_matches_list_filter(&node, &node_filter)
                            && visited.insert(other.clone())
                        {
                            kg.add_node(node);
                            if kg.node_count() < max_nodes {
                                frontier.push_back(other.clone());
                            }
                        }
                    }
                }
            }
=======
                    if candidate_seen.insert(other.clone()) {
                        candidate_ids.push(other.clone());
                    }
                }
            }

            if candidate_ids.is_empty() {
                continue;
            }

            let batch = self.pg_get_nodes_batch(&candidate_ids).await?;
            for id in candidate_ids {
                let Some(node) = batch.get(&id) else {
                    continue;
                };
                if filter_nodes && !node_matches_list_filter(node, &node_filter) {
                    continue;
                }
                if !visited.insert(id.clone()) {
                    continue;
                }
                kg.add_node(node.clone());
                if kg.node_count() < max_nodes {
                    frontier.push_back(id);
                }
                if kg.node_count() >= max_nodes {
                    break;
                }
            }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        }

        let node_ids: Vec<String> = kg.nodes.iter().map(|n| n.id.clone()).collect();
        if !node_ids.is_empty() {
            let edges = self
<<<<<<< HEAD
                .pg_get_edges_for_node_set(&node_ids, Some(tenant_id), Some(workspace_id))
                .await?;
            for edge in edges {
                kg.add_edge(edge);
=======
                .pg_get_edges_for_node_set(&node_ids, tenant_id, workspace_id)
                .await?;
            for edge in edges {
                if !filter_edges || edge_matches_list_filter(&edge, &edge_filter) {
                    kg.add_edge(edge);
                }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            }
        }

        kg.is_truncated = kg.node_count() >= max_nodes;
        Ok(kg)
    }

<<<<<<< HEAD
    pub(in crate::adapters::postgres::graph) async fn pg_get_neighbors(
        &self,
        node_id: &str,
        depth: usize,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<GraphNode>> {
        let escaped_id = Self::escape_cypher_string(node_id);

        let safe_depth = depth.clamp(1, 3);
        const MAX_NEIGHBORS: usize = 500;

        let mut tenant_where = String::new();
        if let Some(tid) = tenant_id {
            tenant_where.push_str(&format!(
                " AND neighbor.tenant_id = '{}'",
                Self::escape_cypher_string(tid)
            ));
        }
        if let Some(wid) = workspace_id {
            tenant_where.push_str(&format!(
                " AND neighbor.workspace_id = '{}'",
                Self::escape_cypher_string(wid)
            ));
        }

        let cypher = format!(
            "MATCH (start:Node {{node_id: '{escaped_id}'}})-[*1..{safe_depth}]-(neighbor:Node) \
             WHERE neighbor.node_id <> '{escaped_id}'{tenant_where} \
             RETURN DISTINCT neighbor \
             LIMIT {MAX_NEIGHBORS}"
        );

        let rows = self.cypher_query(&cypher, &["neighbor"]).await?;

        let neighbors: Vec<GraphNode> = rows
            .iter()
            .filter_map(|row| {
                let json_value: serde_json::Value = row.get("neighbor");
                let agtype_str = json_value.to_string();
                Self::parse_vertex(&agtype_str)
            })
            .collect();

        Ok(neighbors)
    }

=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    /// FAST OPTIMIZED: Get edges between nodes in a specified set using native SQL.
    ///
    /// # WHY: Replace Cypher with native SQL (9s → <200ms)
    ///
    /// The previous Cypher `MATCH (a:Node)-[r:EDGE]->(b:Node) WHERE a.node_id IN [...]`
    /// required AGE to traverse the full vertex table twice (once per endpoint) even with
    /// expression indexes, because the AGE query planner does not push SQL indexes into
    /// Cypher IN-list evaluations for large node sets.
    ///
    /// The native SQL approach directly queries `_ag_label_edge` properties, which stores
    /// `source_id` and `target_id` as top-level properties. With expression indexes on
    /// these fields (migration 036), this becomes an indexed ANY($) lookup.
    pub(in crate::adapters::postgres::graph) async fn pg_get_edges_for_node_set(
        &self,
        node_ids: &[String],
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<GraphEdge>> {
        if node_ids.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

<<<<<<< HEAD
        // WHY: Build SQL IN clause using escaped string literals to avoid the AGE Cypher
        // overhead. This is the same pattern as `get_popular_nodes_with_degree` — native
        // SQL with direct table access. `escape_sql_string` uses '' (not \') for safety.
        let ids_list: Vec<String> = node_ids
            .iter()
            .map(|id| format!("'{}'", Self::escape_sql_string(id)))
            .collect();
        let ids_str = ids_list.join(", ");

=======
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        // WHY: Tenant/workspace filters — legacy NULL-as-wildcard for pre-multitenancy edges.
        let edge_filter = EdgeListFilter {
            tenant_id: tenant_id.map(str::to_string),
            workspace_id: workspace_id.map(str::to_string),
            relationship_type: None,
        };
        let extra_where = Self::edge_and_clause(
            "e",
            &edge_filter,
            EdgeTenantFilterMode::LegacyNullAsWildcard,
        );

<<<<<<< HEAD
        // Native SQL: filter on edge properties directly.
        // `source_id` and `target_id` are stored in edge properties (not vertex joins needed).
=======
        // SPEC-090 F-090-10: bind `= ANY($1::text[])` (referenced twice) so the plan
        // cache is stable and string interpolation is not the injection defense.
        // Native SQL: filter on edge properties directly.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        // Migration 036 adds expression indexes on these properties for fast lookups.
        let sql = format!(
            r#"SELECT ag_catalog.agtype_to_json(e.properties) AS edge_props
               FROM {}."_ag_label_edge" e
<<<<<<< HEAD
               WHERE ag_catalog.agtype_to_json(e.properties)->>'source_id' IN ({})
                 AND ag_catalog.agtype_to_json(e.properties)->>'target_id' IN ({})
                 {}"#,
            self.graph_name, ids_str, ids_str, extra_where
        );

        // WHY: No LOAD 'age' / search_path required for native SQL on AGE tables.
        // The ag_catalog.agtype_to_json function is callable from any search_path
        // when the schema is fully qualified.
        sqlx::query("SET search_path = ag_catalog, \"$user\", public")
            .execute(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Failed to set search_path: {}", e)))?;

        let rows = sqlx::query(&sql).fetch_all(&mut *conn).await.map_err(|e| {
            StorageError::Database(format!("get_edges_for_node_set SQL failed: {}", e))
        })?;
=======
               WHERE ag_catalog.agtype_to_json(e.properties)->>'source_id' = ANY($1::text[])
                 AND ag_catalog.agtype_to_json(e.properties)->>'target_id' = ANY($1::text[])
                 {}"#,
            self.graph_name, extra_where
        );

        // SPEC-089 Wave 3 / F-336-10: PG kill aligned with run_timed_graph_query.
        // SPEC-090 F-090-07: SET LOCAL search_path inside the timed txn (no leak).
        let timeout_ms = super::super::helpers::graph_query_statement_timeout_ms();
        let mut timed = super::super::helpers::LocalTimeoutTx::begin(&mut conn, timeout_ms).await?;
        if let Err(e) = sqlx::query("SET LOCAL search_path TO ag_catalog, \"$user\", public")
            .execute(&mut **timed.as_mut())
            .await
        {
            let _ = timed.rollback().await;
            return Err(StorageError::Database(format!(
                "Failed to set LOCAL search_path: {e}"
            )));
        }
        let rows = match sqlx::query(&sql)
            .bind(node_ids)
            .fetch_all(&mut **timed.as_mut())
            .await
        {
            Ok(r) => {
                timed.commit().await?;
                r
            }
            Err(e) => {
                let _ = timed.rollback().await;
                return Err(StorageError::Database(format!(
                    "get_edges_for_node_set SQL failed: {e}"
                )));
            }
        };
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

        let edges: Vec<GraphEdge> = rows
            .iter()
            .filter_map(|row| {
                let props_json: serde_json::Value = row.get("edge_props");
                Self::parse_edge_from_props(props_json)
            })
            .collect();

        Ok(edges)
    }
}
