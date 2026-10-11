//! Search, labels, and popular nodes (Q-GRAPH-008..010).

use sqlx::Row;

use super::super::PostgresAGEGraphStorage;
use super::rows::{lit, node_from_row, tenant_and, TenantMode};
use crate::error::{Result, StorageError};
use crate::traits::GraphNode;

#[allow(clippy::module_inception)]
pub(in crate::adapters::postgres::graph) mod search {
    use super::*;

    /// Incident degree for every node in the bound namespace (`$1`).
    /// Index-only scans on `(namespace, source_id)` and `(namespace, target_id)`.
    fn incident_degrees_cte() -> &'static str {
        "degrees AS MATERIALIZED (
            SELECT node_id, COUNT(*)::bigint AS degree
            FROM (
                SELECT e.source_id AS node_id
                FROM public.eq_graph_edges e
                WHERE e.namespace = $1
                UNION ALL
                SELECT e.target_id
                FROM public.eq_graph_edges e
                WHERE e.namespace = $1
            ) ends
            GROUP BY node_id
        )"
    }

    pub(in crate::adapters::postgres::graph) async fn search_nodes(
        storage: &PostgresAGEGraphStorage,
        query: &str,
        limit: usize,
        entity_type: Option<&str>,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<(GraphNode, usize)>> {
        let q = lit(&query.to_lowercase());
        let scope = tenant_and("n", tenant_id, workspace_id, TenantMode::Strict);
        let etype = entity_type
            .map(|t| {
                format!(
                    " AND UPPER(COALESCE(n.entity_type, '')) = UPPER('{}')",
                    lit(t)
                )
            })
            .unwrap_or_default();
        let pool = storage.pool.get().await?;
        // One hash aggregate of the namespace edges, then top-k. A correlated
        // COUNT per matching node was 1.2s at 200k nodes (LIKE '%1%').
        let sql = format!(
            "WITH {degrees},
             ranked AS (
                 SELECT n.node_id, COALESCE(d.degree, 0) AS degree
                 FROM public.eq_graph_nodes n
                 LEFT JOIN degrees d ON d.node_id = n.node_id
                 WHERE n.namespace = $1
                   AND (LOWER(n.node_id) LIKE '%{q}%'
                        OR LOWER(COALESCE(n.properties->>'description', '')) LIKE '%{q}%')
                   {scope} {etype}
                 ORDER BY degree DESC, n.node_id
                 LIMIT {limit}
             )
             SELECT n.node_id, n.properties, ranked.degree
             FROM ranked
             JOIN public.eq_graph_nodes n
               ON n.namespace = $1 AND n.node_id = ranked.node_id
             ORDER BY ranked.degree DESC, ranked.node_id",
            degrees = incident_degrees_cte(),
        );
        let rows = sqlx::query(&sql)
            .bind(&storage.namespace)
            .fetch_all(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("plain search_nodes failed: {e}")))?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let node = node_from_row(row)?;
                let degree: i64 = row.try_get("degree").unwrap_or(0);
                Some((node, degree.max(0) as usize))
            })
            .collect())
    }

    pub(in crate::adapters::postgres::graph) async fn popular_nodes(
        storage: &PostgresAGEGraphStorage,
        limit: usize,
        min_degree: Option<usize>,
        entity_type: Option<&str>,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<(GraphNode, usize)>> {
        let scope = tenant_and("n", tenant_id, workspace_id, TenantMode::Strict);
        let etype = entity_type
            .map(|t| {
                format!(
                    " AND UPPER(COALESCE(n.entity_type, '')) = UPPER('{}')",
                    lit(t)
                )
            })
            .unwrap_or_default();
        let min_sql = min_degree
            .map(|d| format!(" AND COALESCE(d.degree, 0) >= {d}"))
            .unwrap_or_default();
        let pool = storage.pool.get().await?;
        // Hash-aggregate incident edges once. The correlated COUNT was 5.3s
        // at 200k nodes / 500k edges; the AGE product plan is a hash join (~0.8s).
        let sql = format!(
            "WITH {degrees},
             ranked AS (
                 SELECT n.node_id, COALESCE(d.degree, 0) AS degree
                 FROM public.eq_graph_nodes n
                 LEFT JOIN degrees d ON d.node_id = n.node_id
                 WHERE n.namespace = $1 {scope} {etype} {min_sql}
                 ORDER BY degree DESC, n.node_id
                 LIMIT {limit}
             )
             SELECT n.node_id, n.properties, ranked.degree
             FROM ranked
             JOIN public.eq_graph_nodes n
               ON n.namespace = $1 AND n.node_id = ranked.node_id
             ORDER BY ranked.degree DESC, ranked.node_id",
            degrees = incident_degrees_cte(),
        );
        let rows = sqlx::query(&sql)
            .bind(&storage.namespace)
            .fetch_all(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("plain popular_nodes failed: {e}")))?;
        Ok(rows
            .iter()
            .filter_map(|row| {
                let node = node_from_row(row)?;
                let degree: i64 = row.try_get("degree").unwrap_or(0);
                Some((node, degree.max(0) as usize))
            })
            .collect())
    }

    pub(in crate::adapters::postgres::graph) async fn search_labels(
        storage: &PostgresAGEGraphStorage,
        query: &str,
        limit: usize,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<String>> {
        let q = lit(&query.to_lowercase());
        let scope = tenant_and("n", tenant_id, workspace_id, TenantMode::Strict);
        let pool = storage.pool.get().await?;
        let sql = format!(
            "SELECT DISTINCT label FROM (
                SELECT n.node_id AS label FROM public.eq_graph_nodes n
                 WHERE n.namespace = $1 {scope} AND LOWER(n.node_id) LIKE '%{q}%'
                UNION
                SELECT n.entity_type FROM public.eq_graph_nodes n
                 WHERE n.namespace = $1 {scope}
                   AND n.entity_type IS NOT NULL
                   AND LOWER(n.entity_type) LIKE '%{q}%'
             ) s
             WHERE label IS NOT NULL AND label <> ''
             ORDER BY label
             LIMIT {limit}"
        );
        let rows = sqlx::query_scalar::<_, String>(&sql)
            .bind(&storage.namespace)
            .fetch_all(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("plain search_labels failed: {e}")))?;
        Ok(rows)
    }

    pub(in crate::adapters::postgres::graph) async fn popular_labels(
        storage: &PostgresAGEGraphStorage,
        limit: usize,
        tenant_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<String>> {
        let scope = tenant_and("n", tenant_id, workspace_id, TenantMode::Strict);
        let pool = storage.pool.get().await?;
        let sql = format!(
            "SELECT n.entity_type
             FROM public.eq_graph_nodes n
             WHERE n.namespace = $1 {scope} AND n.entity_type IS NOT NULL AND n.entity_type <> ''
             GROUP BY n.entity_type
             ORDER BY COUNT(*) DESC, n.entity_type
             LIMIT {limit}"
        );
        let rows = sqlx::query_scalar::<_, String>(&sql)
            .bind(&storage.namespace)
            .fetch_all(&pool)
            .await
            .map_err(|e| StorageError::Database(format!("plain popular_labels failed: {e}")))?;
        Ok(rows)
    }
}
