-- SPEC-040 E2E: Workspace-scoped graph EXPLAIN template
-- Usage: psql "$DATABASE_URL" -v workspace_id="'YOUR-UUID-HERE'" -f explain_workspace_graph.sql

\set graph_schema 'eq_eq_default_graph'

\echo '=== Node / EDGE index inventory ==='
SELECT indexrelname, idx_scan, pg_size_pretty(pg_relation_size(indexrelid)) AS size
FROM pg_stat_user_indexes
WHERE schemaname = :'graph_schema'
  AND (indexrelname LIKE '%workspace%' OR indexrelname LIKE '%start_id%text%')
ORDER BY idx_scan DESC;

\echo '=== Filtered node count plan ==='
EXPLAIN (ANALYZE, BUFFERS, VERBOSE)
SELECT COUNT(*)::bigint
FROM eq_eq_default_graph."_ag_label_vertex" v
WHERE ag_catalog.agtype_to_json(v.properties)->>'workspace_id' = :workspace_id;

\echo '=== Popular nodes pattern (degree join) ==='
EXPLAIN (ANALYZE, BUFFERS)
WITH filtered_nodes AS MATERIALIZED (
  SELECT v.id::text AS id_text, v.properties
  FROM eq_eq_default_graph."_ag_label_vertex" v
  WHERE ag_catalog.agtype_to_json(v.properties)->>'workspace_id' = :workspace_id
),
edge_counts AS (
  SELECT e.start_id::text AS start_id_text, COUNT(*) AS out_degree
  FROM eq_eq_default_graph."_ag_label_edge" e
  INNER JOIN filtered_nodes fn ON e.start_id::text = fn.id_text
  GROUP BY e.start_id::text
)
SELECT COUNT(*)
FROM filtered_nodes fn
LEFT JOIN edge_counts ec ON fn.id_text = ec.start_id_text;

\echo '=== Pass: Hash Join + Index Scan on idx_node_workspace_id ==='
