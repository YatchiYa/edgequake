-- ============================================================================
-- Migration 086: Edge BFS property index reconcile (SPEC-053 hardening)
-- Version: 2.0.0 — 2026-07-14
--
-- PURPOSE:
--   Ensure idx_edge_source_id and idx_edge_target_id exist on every "EDGE"
--   child table across all AGE graph schemas.
--
-- BACKGROUND (SPEC-053 root-cause fix):
--   pg_get_incident_edges_batch was rewritten to query "EDGE" (child table)
--   directly via source_id / target_id properties, eliminating the O(V+E)
--   sequential scan on _ag_label_vertex (parent, no indexes after M070).
--   The new query shape requires both btree expression indexes to be present.
--
--   ensure_indexes() already creates these, but databases that were created
--   before the "EDGE" label was first written (or where ensure_indexes was
--   not called) may be missing them.
--
-- IDEMPOTENT: IF NOT EXISTS on every CREATE INDEX.
-- TRANSACTION SAFETY: Regular CREATE INDEX (no CONCURRENTLY) — safe in txn.
-- AGE not installed: graceful no-op.
-- ============================================================================

DO $$
DECLARE
  v_graph   text;
  v_idx_src text;
  v_idx_tgt text;
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
    RAISE NOTICE 'SPEC-053 M086: AGE not installed — skipping BFS index reconcile';
    RETURN;
  END IF;

  IF NOT EXISTS (
    SELECT 1 FROM information_schema.schemata WHERE schema_name = 'ag_catalog'
  ) THEN
    RAISE NOTICE 'SPEC-053 M086: ag_catalog missing — skipping';
    RETURN;
  END IF;

  FOR v_graph IN
    SELECT name FROM ag_catalog.ag_graph ORDER BY name
  LOOP
    -- Only reconcile graphs that have an "EDGE" table (AGE creates it lazily).
    IF NOT EXISTS (
      SELECT 1 FROM pg_tables
      WHERE schemaname = v_graph AND tablename = 'EDGE'
    ) THEN
      RAISE NOTICE 'SPEC-053 M086: No EDGE table in graph % — skipping', v_graph;
      CONTINUE;
    END IF;

    v_idx_src := 'idx_edge_source_id';
    v_idx_tgt := 'idx_edge_target_id';

    -- -----------------------------------------------------------------------
    -- idx_edge_source_id — used by incident-edge UNION (source branch)
    --   and node_degrees_batch (out-degree COUNT).
    -- -----------------------------------------------------------------------
    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = v_idx_src
    ) THEN
      EXECUTE format(
        'CREATE INDEX %I ON %I."EDGE" '
        '((ag_catalog.agtype_to_json(properties)->>''source_id''))',
        v_idx_src, v_graph
      );
      RAISE NOTICE 'SPEC-053 M086: Created % on graph %', v_idx_src, v_graph;
    ELSE
      RAISE NOTICE 'SPEC-053 M086: % already exists on graph %', v_idx_src, v_graph;
    END IF;

    -- -----------------------------------------------------------------------
    -- idx_edge_target_id — used by incident-edge UNION (target branch)
    --   and node_degrees_batch (in-degree COUNT).
    -- -----------------------------------------------------------------------
    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = v_idx_tgt
    ) THEN
      EXECUTE format(
        'CREATE INDEX %I ON %I."EDGE" '
        '((ag_catalog.agtype_to_json(properties)->>''target_id''))',
        v_idx_tgt, v_graph
      );
      RAISE NOTICE 'SPEC-053 M086: Created % on graph %', v_idx_tgt, v_graph;
    ELSE
      RAISE NOTICE 'SPEC-053 M086: % already exists on graph %', v_idx_tgt, v_graph;
    END IF;

  END LOOP;

  RAISE NOTICE 'SPEC-053 M086: BFS index reconcile complete';
END $$;
