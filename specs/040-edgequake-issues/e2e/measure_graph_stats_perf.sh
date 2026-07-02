#!/usr/bin/env bash
# SPEC-040 — Measure graph stats query performance post-M078.
# Usage: ./specs/040-edgequake-issues/e2e/measure_graph_stats_perf.sh [DATABASE_URL]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
DB_URL="${1:-${DATABASE_URL:-postgresql://edgequake:edgequake_secret@localhost:5432/edgequake}}"

psql_cmd() {
  if docker ps --format '{{.Names}}' 2>/dev/null | grep -qx 'edgequake-postgres'; then
    docker exec edgequake-postgres psql -U edgequake -d edgequake -v ON_ERROR_STOP=1 "$@"
  elif command -v psql >/dev/null 2>&1; then
    psql "$DB_URL" -v ON_ERROR_STOP=1 "$@"
  else
    echo "ERROR: no psql and edgequake-postgres container not running" >&2
    exit 1
  fi
}

echo "== SPEC-040 graph performance (M078) =="
echo "Database: ${DB_URL%%@*}@***"

echo ""
echo "== Migration 078 applied? =="
psql_cmd -c "SELECT version, description, installed_on FROM _sqlx_migrations WHERE version = 78;"

echo ""
echo "== Graph size =="
psql_cmd -c "
SELECT
  (SELECT COUNT(*) FROM eq_eq_default_graph.\"Node\") AS nodes,
  (SELECT COUNT(*) FROM eq_eq_default_graph.\"EDGE\") AS edges;
"

echo ""
echo "== M078 index inventory =="
psql_cmd -c "
SELECT indexrelname, idx_scan, pg_size_pretty(pg_relation_size(indexrelid)) AS size
FROM pg_stat_user_indexes
WHERE schemaname = 'eq_eq_default_graph'
  AND indexrelname IN ('idx_node_workspace_id','idx_node_tenant_id','idx_edge_start_id_text','idx_edge_end_id_text')
ORDER BY indexrelname;
"

WS_ID="$(psql_cmd -t -A -c "SELECT ag_catalog.agtype_to_json(properties)->>'workspace_id' FROM eq_eq_default_graph.\"Node\" WHERE ag_catalog.agtype_to_json(properties)->>'workspace_id' IS NOT NULL LIMIT 1;")"
echo ""
echo "== Workspace filter count (child Node) ws=${WS_ID} =="
psql_cmd -c "\\timing on" -c "
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT COUNT(*)::bigint
FROM eq_eq_default_graph.\"Node\" n
WHERE ag_catalog.agtype_to_json(n.properties)->>'workspace_id' = '${WS_ID}';
"

echo ""
echo "== Degree join pattern (child Node + EDGE) =="
psql_cmd -c "\\timing on" -c "
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
WITH filtered_nodes AS MATERIALIZED (
  SELECT n.id::text AS id_text
  FROM eq_eq_default_graph.\"Node\" n
  WHERE ag_catalog.agtype_to_json(n.properties)->>'workspace_id' = '${WS_ID}'
),
edge_counts AS (
  SELECT e.start_id::text AS start_id_text, COUNT(*) AS out_degree
  FROM eq_eq_default_graph.\"EDGE\" e
  INNER JOIN filtered_nodes fn ON e.start_id::text = fn.id_text
  GROUP BY e.start_id::text
)
SELECT COUNT(*)
FROM filtered_nodes fn
LEFT JOIN edge_counts ec ON fn.id_text = ec.start_id_text;
"

echo ""
echo "Pass criteria: execution time << 15000ms (Tokio graph stream timeout)"
echo "Recorded: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
