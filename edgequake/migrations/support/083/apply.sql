-- ============================================================================
-- SSOT (bootstrap): Native UNIQUE index reconcile for all AGE graphs (M074/M083).
-- Invoked every boot by migration_bootstrap::reconcile_migration_083.
--
-- CHECKSUM SAFETY:
--   * Do NOT edit migrations/083_age_native_unique_index_reconcile.sql
--     (sqlx once-applied; locked in checksums.lock / _sqlx_migrations).
--   * This support/ file is NOT sqlx-scanned and NOT checksum-locked — it is the
--     every-boot reconcile SSOT and may diverge from the frozen sqlx snapshot
--     for fast-boot optimizations (skip O(N) work when UNIQUE index exists).
--
-- Idempotent. Prefer SPEC-062 denormalized eq_* UNIQUEs as the sole ON CONFLICT
-- arbiter. When eq_* UNIQUEs exist, DROP legacy expression UNIQUEs
-- (idx_node_prop_node_id_unique / idx_edge_source_target_unique) — dual unique
-- indexes cause concurrent upsert failures on the non-arbiter index.
-- Fallback: create expression UNIQUEs only when eq_* columns/indexes are absent.
-- ============================================================================

DO $$
DECLARE
  v_graph      text;
  v_dup_count  bigint;
  v_null_count bigint;
  v_node_uniq  text := 'idx_node_prop_node_id_unique';
  v_old_btree  text := 'idx_node_prop_node_id_btree';
  v_edge_uniq  text := 'idx_edge_source_target_unique';
  v_node_eq    text := 'idx_node_eq_node_id';
  v_edge_eq    text := 'idx_edge_eq_source_target';
  v_has_eq_node boolean;
  v_has_eq_edge boolean;
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
    RAISE NOTICE 'M083 reconcile: AGE not installed — skipping';
    RETURN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'ag_catalog') THEN
    RAISE NOTICE 'M083 reconcile: ag_catalog missing — skipping';
    RETURN;
  END IF;

  FOR v_graph IN
    SELECT name FROM ag_catalog.ag_graph ORDER BY name
  LOOP
    RAISE NOTICE 'M083 reconcile: Processing graph: %', v_graph;

    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'Node') THEN
      SELECT EXISTS (
        SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_node_eq
      ) INTO v_has_eq_node;

      IF v_has_eq_node THEN
        -- Single arbiter: drop legacy expression UNIQUE if present.
        IF EXISTS (
          SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_node_uniq
        ) THEN
          EXECUTE format('DROP INDEX IF EXISTS %I.%I', v_graph, v_node_uniq);
          RAISE NOTICE 'M083 reconcile: dropped legacy % on %."Node" (eq_* arbiter present)',
                       v_node_uniq, v_graph;
        ELSE
          RAISE NOTICE 'M083 reconcile: % already present on %."Node" — skip',
                       v_node_eq, v_graph;
        END IF;
      ELSIF EXISTS (
        SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_node_uniq
      ) THEN
        RAISE NOTICE 'M083 reconcile: % already exists on %."Node" — skip dedup/ANALYZE',
                     v_node_uniq, v_graph;
      ELSE
        EXECUTE format(
          'SELECT count(*) FROM %I."Node"'
          ' WHERE COALESCE(ag_catalog.agtype_to_json(properties)->>''node_id'', '''') = ''''',
          v_graph
        ) INTO v_null_count;
        IF v_null_count > 0 THEN
          RAISE WARNING 'M083 reconcile: deleting % Node rows with NULL/empty node_id in %',
                        v_null_count, v_graph;
          EXECUTE format(
            'DELETE FROM %I."Node"'
            ' WHERE COALESCE(ag_catalog.agtype_to_json(properties)->>''node_id'', '''') = ''''',
            v_graph
          );
        END IF;

        EXECUTE format(
          'SELECT count(*) FROM ('
          '  SELECT 1 FROM %I."Node"'
          '  GROUP BY ag_catalog.agtype_to_json(properties)->>''node_id'''
          '  HAVING count(*) > 1'
          ') t',
          v_graph
        ) INTO v_dup_count;

        IF v_dup_count > 0 THEN
          RAISE WARNING 'M083 reconcile: % duplicate node_id groups in %."Node" — deduplicating',
                        v_dup_count, v_graph;
          EXECUTE format(
            'DELETE FROM %I."Node"'
            ' WHERE ctid NOT IN ('
            '   SELECT max(ctid) FROM %I."Node"'
            '   GROUP BY ag_catalog.agtype_to_json(properties)->>''node_id'''
            ' )',
            v_graph, v_graph
          );
        END IF;

        IF EXISTS (
          SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_old_btree
        ) THEN
          EXECUTE format('DROP INDEX IF EXISTS %I.%I', v_graph, v_old_btree);
        END IF;
        EXECUTE format(
          'CREATE UNIQUE INDEX %I ON %I."Node"'
          ' ((ag_catalog.agtype_to_json(properties)->>''node_id''))',
          v_node_uniq, v_graph
        );
        RAISE NOTICE 'M083 reconcile: Created % on %."Node"', v_node_uniq, v_graph;
        EXECUTE format('ANALYZE %I."Node"', v_graph);
      END IF;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'EDGE') THEN
      SELECT EXISTS (
        SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_edge_eq
      ) INTO v_has_eq_edge;

      IF v_has_eq_edge THEN
        IF EXISTS (
          SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_edge_uniq
        ) THEN
          EXECUTE format('DROP INDEX IF EXISTS %I.%I', v_graph, v_edge_uniq);
          RAISE NOTICE 'M083 reconcile: dropped legacy % on %."EDGE" (eq_* arbiter present)',
                       v_edge_uniq, v_graph;
        ELSE
          RAISE NOTICE 'M083 reconcile: % already present on %."EDGE" — skip',
                       v_edge_eq, v_graph;
        END IF;
      ELSIF EXISTS (
        SELECT 1 FROM pg_indexes WHERE schemaname = v_graph AND indexname = v_edge_uniq
      ) THEN
        RAISE NOTICE 'M083 reconcile: % already exists on %."EDGE" — skip dedup/ANALYZE',
                     v_edge_uniq, v_graph;
      ELSE
        EXECUTE format(
          'SELECT count(*) FROM ('
          '  SELECT 1 FROM %I."EDGE"'
          '  GROUP BY (ag_catalog.agtype_to_json(properties)->>''source_id''),'
          '           (ag_catalog.agtype_to_json(properties)->>''target_id'')'
          '  HAVING count(*) > 1'
          ') t',
          v_graph
        ) INTO v_dup_count;

        IF v_dup_count > 0 THEN
          RAISE WARNING 'M083 reconcile: % duplicate edge groups in %."EDGE" — deduplicating',
                        v_dup_count, v_graph;
          EXECUTE format(
            'DELETE FROM %I."EDGE"'
            ' WHERE ctid NOT IN ('
            '   SELECT max(ctid) FROM %I."EDGE"'
            '   GROUP BY (ag_catalog.agtype_to_json(properties)->>''source_id''),'
            '            (ag_catalog.agtype_to_json(properties)->>''target_id'')'
            ' )',
            v_graph, v_graph
          );
        END IF;

        EXECUTE format(
          'CREATE UNIQUE INDEX %I ON %I."EDGE" ('
          '  (ag_catalog.agtype_to_json(properties)->>''source_id''),'
          '  (ag_catalog.agtype_to_json(properties)->>''target_id'')'
          ')',
          v_edge_uniq, v_graph
        );
        RAISE NOTICE 'M083 reconcile: Created % on %."EDGE"', v_edge_uniq, v_graph;
        EXECUTE format('ANALYZE %I."EDGE"', v_graph);
      END IF;
    END IF;
  END LOOP;

  RAISE NOTICE 'M083 reconcile: COMPLETE';
END $$;
