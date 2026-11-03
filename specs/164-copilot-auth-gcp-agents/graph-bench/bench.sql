-- EdgeQuake plain graph at 1M entities, hub-heavy (power-law) degrees.
-- Schema = edgequake/migrations/170_spec164_plain_graph.sql (tables + request-path indexes).
-- Run on a throwaway database: psql -v ON_ERROR_STOP=1 -f bench.sql
\timing off
\set ns 'bench'
CREATE EXTENSION IF NOT EXISTS pg_trgm;

DROP TABLE IF EXISTS eq_graph_edges, eq_graph_nodes;
CREATE TABLE eq_graph_nodes (
    namespace text NOT NULL, node_id text NOT NULL,
    tenant_id text NOT NULL DEFAULT '', workspace_id text NOT NULL DEFAULT '',
    entity_type text, properties jsonb NOT NULL DEFAULT '{}'::jsonb,
    degree bigint NOT NULL DEFAULT 0, PRIMARY KEY (namespace, node_id));
CREATE TABLE eq_graph_edges (
    namespace text NOT NULL, source_id text NOT NULL, target_id text NOT NULL,
    rel_type text NOT NULL DEFAULT '', tenant_id text NOT NULL DEFAULT '', workspace_id text NOT NULL DEFAULT '',
    weight double precision, properties jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (namespace, source_id, target_id, rel_type));

-- 1,000,000 entities in 1 workspace. Names like the extractor's (UPPER_SNAKE).
\echo load nodes
\timing on
INSERT INTO eq_graph_nodes (namespace, node_id, tenant_id, workspace_id, entity_type, properties)
SELECT :'ns', 'ENTITY_' || i, 't1', 'w1',
       (ARRAY['PERSON','ORGANIZATION','PRODUCT','LOCATION','CONCEPT','EVENT'])[1 + i % 6],
       jsonb_build_object('description', 'Entity ' || i || ' description text about topic ' || (i % 5000))
FROM generate_series(1, 1000000) i;

-- ~3,000,000 relationships. Target chosen by a power law: node rank r gets ~1/r of the links,
-- so a few entities have tens of thousands of links (like ALTAROC in a real corpus).
\echo load edges
INSERT INTO eq_graph_edges (namespace, source_id, target_id, rel_type, tenant_id, workspace_id, weight)
SELECT DISTINCT ON (s, t) :'ns', 'ENTITY_' || s, 'ENTITY_' || t, 'RELATED_TO', 't1', 'w1', 1.0
FROM (SELECT 1 + (random() * 999999)::int AS s,
             greatest(1, least(1000000, floor(exp(random() * ln(1000000)))::int)) AS t
      FROM generate_series(1, 3000000)) x
WHERE s <> t;

\echo indexes (as migration 170)
CREATE INDEX idx_eq_graph_edges_source ON eq_graph_edges (namespace, source_id);
CREATE INDEX idx_eq_graph_edges_target ON eq_graph_edges (namespace, target_id);
CREATE INDEX idx_eq_graph_nodes_tenant_ws ON eq_graph_nodes (namespace, tenant_id, workspace_id);
CREATE INDEX idx_eq_graph_nodes_id_trgm ON eq_graph_nodes USING gin (node_id gin_trgm_ops);

\echo degree (maintained by triggers in the product)
UPDATE eq_graph_nodes n SET degree = d.deg
FROM (SELECT id, count(*) deg FROM (SELECT source_id id FROM eq_graph_edges UNION ALL SELECT target_id FROM eq_graph_edges) u GROUP BY id) d
WHERE n.namespace = 'bench' AND n.node_id = d.id;
CREATE INDEX idx_eq_graph_nodes_degree ON eq_graph_nodes (namespace, degree DESC);
\timing off
VACUUM ANALYZE eq_graph_nodes; VACUUM ANALYZE eq_graph_edges;

SELECT count(*) AS nodes FROM eq_graph_nodes;
SELECT count(*) AS edges FROM eq_graph_edges;
SELECT node_id, degree FROM eq_graph_nodes ORDER BY degree DESC LIMIT 3;
SELECT percentile_disc(ARRAY[0.5, 0.99]) WITHIN GROUP (ORDER BY degree) AS degree_p50_p99 FROM eq_graph_nodes;
SELECT pg_size_pretty(pg_total_relation_size('eq_graph_nodes') + pg_total_relation_size('eq_graph_edges')) AS graph_size;
