//! Bounded graph scan — SPEC-006 postgres push-down.

use super::helpers::{EdgeTenantFilterMode, VertexTenantFilterMode};
use super::PostgresAGEGraphStorage;
use crate::error::{Result, StorageError};
use crate::traits::{EdgeListFilter, GraphEdge, GraphNode, NodeListFilter, PagedGraphResult};
use sqlx::Row;
use std::collections::HashMap;

impl PostgresAGEGraphStorage {
    fn build_node_where_clause(filter: &NodeListFilter) -> String {
        Self::build_vertex_property_where("v", filter)
    }

    fn build_node_where_clause_for_discovery(filter: &NodeListFilter) -> String {
        Self::build_vertex_property_where_mode(
            "v",
            filter,
            VertexTenantFilterMode::LegacyNullAsWildcard,
        )
    }

    fn build_edge_where_clause(filter: &EdgeListFilter) -> String {
        Self::build_edge_property_where("e", filter, EdgeTenantFilterMode::Strict)
    }

    fn build_edge_where_clause_for_discovery(filter: &EdgeListFilter) -> String {
        Self::build_edge_property_where("e", filter, EdgeTenantFilterMode::LegacyNullAsWildcard)
    }

    pub(super) async fn pg_list_nodes_filtered(
        &self,
        filter: &NodeListFilter,
        offset: usize,
        limit: usize,
    ) -> Result<PagedGraphResult<GraphNode>> {
        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        let where_clause = Self::build_node_where_clause(filter);

        let count_sql = format!(
            "SELECT COUNT(*)::BIGINT AS total
             FROM {graph}.\"_ag_label_vertex\" v
             WHERE {where_clause}",
            graph = self.graph_name,
            where_clause = where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Node count query failed: {}", e)))?;

        let page_sql = format!(
            "SELECT ag_catalog.agtype_to_json(v.properties) AS props
             FROM {graph}.\"_ag_label_vertex\" v
             WHERE {where_clause}
             ORDER BY ag_catalog.agtype_to_json(v.properties)->>'node_id'
             OFFSET {offset} LIMIT {limit}",
            graph = self.graph_name,
            where_clause = where_clause,
            offset = offset,
            limit = limit
        );

        let rows = sqlx::query(&page_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Node list query failed: {}", e)))?;

        let items: Vec<GraphNode> = rows
            .iter()
            .filter_map(|row| {
                let props: serde_json::Value = row.get("props");
                let node_id = props.get("node_id")?.as_str()?.to_string();
                let properties = props.as_object()?.clone().into_iter().collect();
                Some(GraphNode {
                    id: node_id,
                    properties,
                })
            })
            .collect();

        Ok(PagedGraphResult {
            items,
            total: total as usize,
            offset,
            limit,
        })
    }

    pub(super) async fn pg_list_edges_filtered(
        &self,
        filter: &EdgeListFilter,
        offset: usize,
        limit: usize,
    ) -> Result<PagedGraphResult<GraphEdge>> {
        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        let where_clause = Self::build_edge_where_clause(filter);

        let count_sql = format!(
            "SELECT COUNT(*)::BIGINT AS total
             FROM {graph}.\"_ag_label_edge\" e
             WHERE {where_clause}",
            graph = self.graph_name,
            where_clause = where_clause
        );

        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Edge count query failed: {}", e)))?;

        let page_sql = format!(
            "SELECT
                ag_catalog.agtype_to_json(e.properties) AS props,
                ag_catalog.agtype_to_json(sv.properties)->>'node_id' AS source_id,
                ag_catalog.agtype_to_json(tv.properties)->>'node_id' AS target_id
             FROM {graph}.\"_ag_label_edge\" e
             JOIN {graph}.\"_ag_label_vertex\" sv ON e.start_id::text = sv.id::text
             JOIN {graph}.\"_ag_label_vertex\" tv ON e.end_id::text = tv.id::text
             WHERE {where_clause}
             ORDER BY source_id, target_id
             OFFSET {offset} LIMIT {limit}",
            graph = self.graph_name,
            where_clause = where_clause,
            offset = offset,
            limit = limit
        );

        let rows = sqlx::query(&page_sql)
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Edge list query failed: {}", e)))?;

        let items: Vec<GraphEdge> = rows
            .iter()
            .filter_map(|row| {
                let props: serde_json::Value = row.get("props");
                let source: String = row.get("source_id");
                let target: String = row.get("target_id");
                let properties = props.as_object()?.clone().into_iter().collect();
                Some(GraphEdge {
                    source,
                    target,
                    properties,
                })
            })
            .collect();

        Ok(PagedGraphResult {
            items,
            total: total as usize,
            offset,
            limit,
        })
    }

    fn build_source_prefix_clause_modern(props_expr: &str, source_prefixes: &[String]) -> String {
        let props = format!("({props_expr})::jsonb");
        let mut conditions = Vec::new();
        for prefix in source_prefixes {
            conditions.push(super::helpers::jsonb_matches_doc_source_prefix_modern(
                &props, prefix,
            ));
        }
        if conditions.is_empty() {
            "FALSE".to_string()
        } else {
            conditions.join(" OR ")
        }
    }

    fn build_source_prefix_clause_legacy(props_expr: &str, source_prefixes: &[String]) -> String {
        let props = format!("({props_expr})::jsonb");
        let mut conditions = Vec::new();
        for prefix in source_prefixes {
            conditions.push(super::helpers::jsonb_matches_doc_source_prefix_legacy(
                &props, prefix,
            ));
        }
        if conditions.is_empty() {
            "FALSE".to_string()
        } else {
            conditions.join(" OR ")
        }
    }

    pub(super) async fn pg_find_nodes_by_source_prefixes(
        &self,
        filter: &NodeListFilter,
        source_prefixes: &[String],
    ) -> Result<Vec<GraphNode>> {
        if source_prefixes.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        // Discovery uses legacy-null workspace match; never require tenant props.
        let tenant_where = Self::build_node_where_clause_for_discovery(filter);
        let props_expr = "ag_catalog.agtype_to_json(v.properties)";
        let modern_where = Self::build_source_prefix_clause_modern(props_expr, source_prefixes);
        let legacy_where = Self::build_source_prefix_clause_legacy(props_expr, source_prefixes);

        // Two-path: indexed modern `@>` first, then bounded legacy-only fallback.
        let modern_sql = format!(
            "SELECT {props} AS props
             FROM {graph}.\"_ag_label_vertex\" v
             WHERE {tenant_where} AND ({modern_where})
             ORDER BY {props}->>'node_id'",
            props = props_expr,
            graph = self.graph_name,
            tenant_where = tenant_where,
            modern_where = modern_where
        );
        let legacy_sql = format!(
            "SELECT {props} AS props
             FROM {graph}.\"_ag_label_vertex\" v
             WHERE {tenant_where} AND ({legacy_where})
             ORDER BY {props}->>'node_id'
             LIMIT 5000",
            props = props_expr,
            graph = self.graph_name,
            tenant_where = tenant_where,
            legacy_where = legacy_where
        );

        let mut by_id: HashMap<String, GraphNode> = HashMap::new();
        for sql in [modern_sql, legacy_sql] {
            let rows = sqlx::query(&sql).fetch_all(&mut *conn).await.map_err(|e| {
                StorageError::Database(format!("Source-prefix node query failed: {}", e))
            })?;
            for row in rows {
                let props: serde_json::Value = row.get("props");
                let Some(node_id) = props.get("node_id").and_then(|v| v.as_str()) else {
                    continue;
                };
                let Some(obj) = props.as_object() else {
                    continue;
                };
                by_id.entry(node_id.to_string()).or_insert(GraphNode {
                    id: node_id.to_string(),
                    properties: obj.clone().into_iter().collect(),
                });
            }
        }

        let mut out: Vec<GraphNode> = by_id.into_values().collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub(super) async fn pg_find_edges_by_source_prefixes(
        &self,
        filter: &EdgeListFilter,
        source_prefixes: &[String],
    ) -> Result<Vec<GraphEdge>> {
        if source_prefixes.is_empty() {
            return Ok(Vec::new());
        }

        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        let tenant_where = Self::build_edge_where_clause_for_discovery(filter);
        let props_expr = "ag_catalog.agtype_to_json(e.properties)";
        let modern_where = Self::build_source_prefix_clause_modern(props_expr, source_prefixes);
        let legacy_where = Self::build_source_prefix_clause_legacy(props_expr, source_prefixes);

        let mut by_key: HashMap<(String, String), GraphEdge> = HashMap::new();
        for (source_where, limit_sql) in [
            (modern_where.as_str(), ""),
            (legacy_where.as_str(), " LIMIT 5000"),
        ] {
            let sql = format!(
                "SELECT
                    {props} AS props,
                    ag_catalog.agtype_to_json(sv.properties)->>'node_id' AS source_id,
                    ag_catalog.agtype_to_json(tv.properties)->>'node_id' AS target_id
                 FROM {graph}.\"_ag_label_edge\" e
                 JOIN {graph}.\"_ag_label_vertex\" sv ON e.start_id::text = sv.id::text
                 JOIN {graph}.\"_ag_label_vertex\" tv ON e.end_id::text = tv.id::text
                 WHERE {tenant_where} AND ({source_where})
                 ORDER BY source_id, target_id{limit_sql}",
                props = props_expr,
                graph = self.graph_name,
                tenant_where = tenant_where,
                source_where = source_where,
                limit_sql = limit_sql
            );
            let rows = sqlx::query(&sql).fetch_all(&mut *conn).await.map_err(|e| {
                StorageError::Database(format!("Source-prefix edge query failed: {}", e))
            })?;
            for row in rows {
                let props: serde_json::Value = row.get("props");
                let source: String = row.get("source_id");
                let target: String = row.get("target_id");
                let Some(obj) = props.as_object() else {
                    continue;
                };
                by_key
                    .entry((source.clone(), target.clone()))
                    .or_insert(GraphEdge {
                        source,
                        target,
                        properties: obj.clone().into_iter().collect(),
                    });
            }
        }

        let mut out: Vec<GraphEdge> = by_key.into_values().collect();
        out.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));
        Ok(out)
    }

    pub(super) async fn pg_find_edge_by_relationship_id(
        &self,
        filter: &EdgeListFilter,
        relationship_id: &str,
    ) -> Result<Option<GraphEdge>> {
        if relationship_id.is_empty() {
            return Ok(None);
        }

        let pool = self.pool.get().await?;
        let mut conn = pool.acquire().await.map_err(|e| {
            StorageError::Connection(format!("Failed to acquire connection: {}", e))
        })?;

        let tenant_where = Self::build_edge_where_clause(filter);
        let esc_id = Self::escape_sql_string(relationship_id);
        let props_expr = "ag_catalog.agtype_to_json(e.properties)";

        let sql = format!(
            "SELECT
                {props} AS props,
                ag_catalog.agtype_to_json(sv.properties)->>'node_id' AS source_id,
                ag_catalog.agtype_to_json(tv.properties)->>'node_id' AS target_id
             FROM {graph}.\"_ag_label_edge\" e
             JOIN {graph}.\"_ag_label_vertex\" sv ON e.start_id::text = sv.id::text
             JOIN {graph}.\"_ag_label_vertex\" tv ON e.end_id::text = tv.id::text
             WHERE {tenant_where}
               AND (
                 {props}->>'id' = '{esc_id}'
                 OR CONCAT(
                   ag_catalog.agtype_to_json(sv.properties)->>'node_id',
                   '_',
                   ag_catalog.agtype_to_json(tv.properties)->>'node_id'
                 ) = '{esc_id}'
               )
             LIMIT 1",
            props = props_expr,
            graph = self.graph_name,
            tenant_where = tenant_where,
            esc_id = esc_id
        );

        let row = sqlx::query(&sql)
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| StorageError::Database(format!("Relationship id lookup failed: {}", e)))?;

        Ok(row.map(|row| {
            let props: serde_json::Value = row.get("props");
            let source: String = row.get("source_id");
            let target: String = row.get("target_id");
            let properties = props
                .as_object()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();
            GraphEdge {
                source,
                target,
                properties,
            }
        }))
    }
}

#[cfg(test)]
mod source_prefix_clause_tests {
    use super::PostgresAGEGraphStorage;

    #[test]
    fn source_prefix_clause_casts_agtype_json_to_jsonb() {
        let clause = PostgresAGEGraphStorage::build_source_prefix_clause(
            "ag_catalog.agtype_to_json(v.properties)",
            &["doc-abc".to_string()],
        );
        assert!(
            clause.contains("::jsonb"),
            "jsonb_* functions require jsonb cast: {clause}"
        );
        assert!(clause.contains("jsonb_typeof"));
        assert!(clause.contains("jsonb_array_elements_text"));
    }
}
