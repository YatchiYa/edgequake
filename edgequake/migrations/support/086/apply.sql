-- ============================================================================
-- SSOT (bootstrap): SPEC-053 / SPEC-070 EDGE BFS index reconcile for all AGE graphs.
-- Invoked every boot by migration_bootstrap::reconcile_migration_086.
--
-- CHECKSUM SAFETY:
--   * Do NOT edit migrations/086_edge_bfs_index_reconcile.sql
--     (sqlx once-applied; locked in checksums.lock / _sqlx_migrations).
--   * This support/ file is NOT sqlx-scanned — every-boot reconcile SSOT.
--
-- Ensures idx_edge_source_id / idx_edge_target_id (expression btree on
-- properties) for incident-edge batch + node_degrees_batch. Also ensures
-- text-cast start_id/end_id helpers when missing (parity with ensure_indexes).
-- DDL GUCs: statement_timeout=0, lock_timeout=5s.
-- ============================================================================

SET statement_timeout = 0;
SET lock_timeout = '5s';

DO $$
DECLARE
  v_graph text;
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
    RAISE NOTICE 'M086 reconcile: AGE not installed — skipping';
    RETURN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'ag_catalog') THEN
    RAISE NOTICE 'M086 reconcile: ag_catalog missing — skipping';
    RETURN;
  END IF;

  FOR v_graph IN
    SELECT name FROM ag_catalog.ag_graph ORDER BY name
  LOOP
    IF NOT EXISTS (
      SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'EDGE'
    ) THEN
      RAISE NOTICE 'M086 reconcile: No EDGE table in % — skip', v_graph;
      CONTINUE;
    END IF;

    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_source_id'
    ) THEN
      EXECUTE format(
        'CREATE INDEX IF NOT EXISTS idx_edge_source_id ON %I."EDGE" '
        '((ag_catalog.agtype_to_json(properties)->>''source_id''))',
        v_graph
      );
      RAISE NOTICE 'M086 reconcile: created idx_edge_source_id on %', v_graph;
    END IF;

    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_target_id'
    ) THEN
      EXECUTE format(
        'CREATE INDEX IF NOT EXISTS idx_edge_target_id ON %I."EDGE" '
        '((ag_catalog.agtype_to_json(properties)->>''target_id''))',
        v_graph
      );
      RAISE NOTICE 'M086 reconcile: created idx_edge_target_id on %', v_graph;
    END IF;

    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_start_id_text'
    ) THEN
      EXECUTE format(
        'CREATE INDEX IF NOT EXISTS idx_edge_start_id_text ON %I."EDGE" ((start_id::text))',
        v_graph
      );
    END IF;

    IF NOT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_end_id_text'
    ) THEN
      EXECUTE format(
        'CREATE INDEX IF NOT EXISTS idx_edge_end_id_text ON %I."EDGE" ((end_id::text))',
        v_graph
      );
    END IF;

    RAISE NOTICE 'M086 reconcile: BFS indexes ready on %', v_graph;
  END LOOP;
END
$$;
