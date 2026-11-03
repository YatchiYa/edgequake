-- Median latency (ms) of EdgeQuake's request-path graph shapes on the 1M graph from bench.sql.
-- Each op runs from 20 random seeds (fixed order), inside one session (no HTTP/pool overhead).
CREATE OR REPLACE FUNCTION bench_ms(label text, sql text, seeds text[]) RETURNS TABLE(op text, median_ms numeric, max_ms numeric, rows_last bigint)
LANGUAGE plpgsql AS $$
DECLARE t0 timestamptz; ms numeric[] := '{}'; s text; n bigint;
BEGIN
  EXECUTE format(sql, seeds[1]);                       -- warm-up
  FOREACH s IN ARRAY seeds LOOP
    t0 := clock_timestamp();
    EXECUTE 'SELECT count(*) FROM (' || format(sql, s) || ') q' INTO n;
    ms := ms || round(extract(epoch FROM clock_timestamp() - t0)::numeric * 1000, 3);
  END LOOP;
  RETURN QUERY SELECT label, (SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY x)::numeric(10,3) FROM unnest(ms) x),
                      (SELECT max(x) FROM unnest(ms) x), n;
END $$;

-- k-hop neighbourhood the way the product walks it: one batched incident-edge query per hop
-- on the frontier, stop at max_nodes (graph_traversal.rs: depth <= 3, 500 nodes).
CREATE OR REPLACE FUNCTION khop(seed text, depth int, max_nodes int) RETURNS SETOF text
LANGUAGE plpgsql AS $$
DECLARE visited text[] := ARRAY[seed]; frontier text[] := ARRAY[seed]; nxt text[];
BEGIN
  FOR d IN 1..depth LOOP
    EXIT WHEN array_length(frontier, 1) IS NULL OR array_length(visited, 1) >= max_nodes;
    SELECT array_agg(x) INTO nxt FROM (
      SELECT DISTINCT x FROM (
        SELECT target_id x FROM eq_graph_edges WHERE namespace = 'bench' AND source_id = ANY(frontier)
        UNION ALL
        SELECT source_id FROM eq_graph_edges WHERE namespace = 'bench' AND target_id = ANY(frontier)) e
      WHERE x <> ALL(visited) LIMIT max_nodes - array_length(visited, 1)) y;
    EXIT WHEN nxt IS NULL;
    visited := visited || nxt; frontier := nxt;
  END LOOP;
  RETURN QUERY SELECT unnest(visited);
END $$;

-- Seeds: 20 ordinary entities (median degree) and the 20 biggest hubs.
\set ordinary '(SELECT array_agg(node_id) FROM (SELECT node_id FROM eq_graph_nodes WHERE namespace = ''bench'' AND degree BETWEEN 2 AND 8 ORDER BY md5(node_id) LIMIT 20) z)'
\set hubs '(SELECT array_agg(node_id) FROM (SELECT node_id FROM eq_graph_nodes WHERE namespace = ''bench'' ORDER BY degree DESC LIMIT 20) z)'

SELECT * FROM bench_ms('point get (entity + properties)',
  $q$SELECT * FROM eq_graph_nodes WHERE namespace = 'bench' AND node_id = %L$q$, :ordinary);
SELECT * FROM bench_ms('1-hop, ordinary entity (all links)',
  $q$SELECT * FROM eq_graph_edges WHERE namespace = 'bench' AND (source_id = %1$L OR target_id = %1$L)$q$, :ordinary);
SELECT * FROM bench_ms('1-hop, top hub, capped 500 (product)',
  $q$(SELECT * FROM eq_graph_edges WHERE namespace = 'bench' AND (source_id = %1$L OR target_id = %1$L) LIMIT 500)$q$, :hubs);
SELECT * FROM bench_ms('2-hop BFS, ordinary seed, cap 500 (product)', $q$SELECT khop(%L, 2, 500)$q$, :ordinary);
SELECT * FROM bench_ms('3-hop BFS, ordinary seed, cap 500 (product)', $q$SELECT khop(%L, 3, 500)$q$, :ordinary);
SELECT * FROM bench_ms('3-hop BFS, hub seed, cap 500 (product)', $q$SELECT khop(%L, 3, 500)$q$, :hubs);
SELECT * FROM bench_ms('top 20 most connected (graph home)',
  $q$SELECT node_id FROM eq_graph_nodes WHERE namespace = 'bench' AND %L <> '' ORDER BY degree DESC LIMIT 20$q$, :ordinary);
SELECT * FROM bench_ms('label search, substring (trigram)',
  $q$SELECT node_id FROM eq_graph_nodes WHERE namespace = 'bench' AND node_id ILIKE '%%' || substr(%L, 1, 12) || '%%' LIMIT 20$q$, :ordinary);
-- Beyond the product's caps: where a relational graph starts to cost.
SET statement_timeout = '120s';  -- a timeout here is itself the answer
SELECT * FROM bench_ms('UNCAPPED 2-hop from a top hub (count)', $q$SELECT khop(%L, 2, 10000000)$q$,
  (SELECT array_agg(node_id) FROM (SELECT node_id FROM eq_graph_nodes WHERE namespace = 'bench' ORDER BY degree DESC LIMIT 3) z));
-- Path between two ordinary entities, breadth-first up to 4 hops (recursive CTE, level-set).
SELECT * FROM bench_ms('shortest path <=4 hops, 2 ordinary entities (recursive CTE)', $q$
  WITH RECURSIVE walk(node, depth) AS (
    SELECT %L::text, 0
    UNION
    SELECT CASE WHEN e.source_id = w.node THEN e.target_id ELSE e.source_id END, w.depth + 1
    FROM walk w JOIN eq_graph_edges e ON e.namespace = 'bench' AND (e.source_id = w.node OR e.target_id = w.node)
    WHERE w.depth < 4)
  SELECT min(depth) FROM walk WHERE node = 'ENTITY_777777'$q$, (SELECT (:ordinary)[1:3]));
