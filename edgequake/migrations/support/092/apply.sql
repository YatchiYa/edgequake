-- ============================================================================
-- SSOT (bootstrap): SPEC-062 / SPEC-069 / D-30 eq_* denorm reconcile for all AGE graphs.
-- Invoked every boot by migration_bootstrap::reconcile_migration_092.
--
-- CHECKSUM SAFETY:
--   * Do NOT edit migrations/092_eq_id_denorm_marker.sql
--     (sqlx once-applied marker; locked in checksums.lock / _sqlx_migrations).
--   * Do NOT edit migrations/097_edge_multigraph_rel_type.sql (marker only).
--   * This support/ file is NOT sqlx-scanned — every-boot reconcile SSOT.
--
-- Idempotent. ADD COLUMN IF NOT EXISTS (nullable, no rewrite). Create UNIQUE
-- indexes + sync triggers only when missing. Never unconditional DROP TRIGGER.
-- D-30: eq_rel_type + idx_edge_eq_source_target_rel (3-col arbiter); drop legacy
-- 2-col idx_edge_eq_source_target after _rel exists.
--
-- DDL GUCs:
--   * Default: statement_timeout=0, lock_timeout=5s (fail-fast under query load).
--   * Maintenance (SPEC-083 / P0): when session GUC edgequake.eq_maintenance='1'
--     (set by bootstrap when EDGEQUAKE_EQ_MAINTENANCE=1), lock_timeout=120s so
--     ADD COLUMN / CREATE INDEX can win during a planned window.
--
-- Backfill:
--   * Always NULL-only (WHERE eq_* IS NULL) — never rewrite populated rows.
--   * Maintenance mode uses ctid-batched UPDATEs (~10k rows) to avoid long
--     AccessExclusive-adjacent heap rewrites under concurrent readers.
-- ============================================================================

SET statement_timeout = 0;

DO $$
DECLARE
  v_maint text := coalesce(current_setting('edgequake.eq_maintenance', true), '');
BEGIN
  IF v_maint IN ('1', 'true', 'TRUE', 'yes', 'YES') THEN
    EXECUTE 'SET lock_timeout = ''120s''';
    RAISE NOTICE 'M092 reconcile: maintenance mode — lock_timeout=120s (batched NULL-only backfill)';
  ELSE
    EXECUTE 'SET lock_timeout = ''5s''';
  END IF;
END
$$;

DO $$
DECLARE
  v_graph text;
  v_has_col boolean;
  v_has_idx boolean;
  v_has_trg boolean;
  v_maint boolean := coalesce(current_setting('edgequake.eq_maintenance', true), '')
                     IN ('1', 'true', 'TRUE', 'yes', 'YES');
  v_batch int := 10000;
  v_updated bigint;
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
    RAISE NOTICE 'M092 reconcile: AGE not installed — skipping';
    RETURN;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.schemata WHERE schema_name = 'ag_catalog') THEN
    RAISE NOTICE 'M092 reconcile: ag_catalog missing — skipping';
    RETURN;
  END IF;

  FOR v_graph IN
    SELECT name FROM ag_catalog.ag_graph ORDER BY name
  LOOP
    RAISE NOTICE 'M092 reconcile: Processing graph: %', v_graph;

    IF NOT EXISTS (
      SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'Node'
    ) OR NOT EXISTS (
      SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'EDGE'
    ) THEN
      RAISE NOTICE 'M092 reconcile: Node/EDGE pending on % — skip', v_graph;
      CONTINUE;
    END IF;

    -- Columns (nullable ADD — no table rewrite).
    SELECT EXISTS (
      SELECT 1 FROM information_schema.columns
      WHERE table_schema = v_graph AND table_name = 'Node' AND column_name = 'eq_node_id'
    ) INTO v_has_col;
    IF NOT v_has_col THEN
      EXECUTE format('ALTER TABLE %I."Node" ADD COLUMN IF NOT EXISTS eq_node_id text', v_graph);
    END IF;

    SELECT EXISTS (
      SELECT 1 FROM information_schema.columns
      WHERE table_schema = v_graph AND table_name = 'EDGE' AND column_name = 'eq_source_id'
    ) INTO v_has_col;
    IF NOT v_has_col THEN
      EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_source_id text', v_graph);
    END IF;

    SELECT EXISTS (
      SELECT 1 FROM information_schema.columns
      WHERE table_schema = v_graph AND table_name = 'EDGE' AND column_name = 'eq_target_id'
    ) INTO v_has_col;
    IF NOT v_has_col THEN
      EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_target_id text', v_graph);
    END IF;

    -- D-30: multigraph arbiter column.
    SELECT EXISTS (
      SELECT 1 FROM information_schema.columns
      WHERE table_schema = v_graph AND table_name = 'EDGE' AND column_name = 'eq_rel_type'
    ) INTO v_has_col;
    IF NOT v_has_col THEN
      EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_rel_type text', v_graph);
    END IF;

    -- NULL-only backfill (batched in maintenance mode for large AGE graphs).
    IF v_maint THEN
      LOOP
        EXECUTE format(
          'WITH batch AS (
             SELECT ctid FROM %I."Node"
             WHERE eq_node_id IS NULL
             LIMIT %s
           )
           UPDATE %I."Node" n
           SET eq_node_id = ag_catalog.agtype_to_json(n.properties)->>''node_id''
           FROM batch b WHERE n.ctid = b.ctid',
          v_graph, v_batch, v_graph
        );
        GET DIAGNOSTICS v_updated = ROW_COUNT;
        EXIT WHEN v_updated = 0;
        RAISE NOTICE 'M092 reconcile: batched Node backfill % rows on %', v_updated, v_graph;
      END LOOP;

      LOOP
        EXECUTE format(
          'WITH batch AS (
             SELECT ctid FROM %I."EDGE"
             WHERE eq_source_id IS NULL OR eq_target_id IS NULL OR eq_rel_type IS NULL
             LIMIT %s
           )
           UPDATE %I."EDGE" e
           SET
             eq_source_id = COALESCE(
               e.eq_source_id,
               ag_catalog.agtype_to_json(e.properties)->>''source_id''
             ),
             eq_target_id = COALESCE(
               e.eq_target_id,
               ag_catalog.agtype_to_json(e.properties)->>''target_id''
             ),
             eq_rel_type = COALESCE(
               e.eq_rel_type,
               UPPER(COALESCE(
                 NULLIF(TRIM(ag_catalog.agtype_to_json(e.properties)->>''relation_type''), ''''),
                 ''RELATED_TO''
               ))
             )
           FROM batch b WHERE e.ctid = b.ctid',
          v_graph, v_batch, v_graph
        );
        GET DIAGNOSTICS v_updated = ROW_COUNT;
        EXIT WHEN v_updated = 0;
        RAISE NOTICE 'M092 reconcile: batched EDGE backfill % rows on %', v_updated, v_graph;
      END LOOP;
    ELSE
      -- Boot path: single NULL-only UPDATE (small/medium graphs; fail-fast locks).
      EXECUTE format(
        'UPDATE %I."Node" SET eq_node_id = ag_catalog.agtype_to_json(properties)->>''node_id''
         WHERE eq_node_id IS NULL',
        v_graph
      );
      EXECUTE format(
        'UPDATE %I."EDGE" SET
           eq_source_id = COALESCE(
             eq_source_id,
             ag_catalog.agtype_to_json(properties)->>''source_id''
           ),
           eq_target_id = COALESCE(
             eq_target_id,
             ag_catalog.agtype_to_json(properties)->>''target_id''
           ),
           eq_rel_type = COALESCE(
             eq_rel_type,
             UPPER(COALESCE(
               NULLIF(TRIM(ag_catalog.agtype_to_json(properties)->>''relation_type''), ''''),
               ''RELATED_TO''
             ))
           )
         WHERE eq_source_id IS NULL OR eq_target_id IS NULL OR eq_rel_type IS NULL',
        v_graph
      );
    END IF;

    -- Indexes
    SELECT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_node_eq_node_id'
    ) INTO v_has_idx;
    IF NOT v_has_idx THEN
      EXECUTE format(
        'CREATE UNIQUE INDEX IF NOT EXISTS idx_node_eq_node_id
         ON %I."Node" (eq_node_id) WHERE eq_node_id IS NOT NULL',
        v_graph
      );
    END IF;

    -- D-30: 3-col multigraph unique; drop legacy 2-col after _rel exists.
    SELECT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_eq_source_target_rel'
    ) INTO v_has_idx;
    IF NOT v_has_idx THEN
      EXECUTE format(
        'CREATE UNIQUE INDEX IF NOT EXISTS idx_edge_eq_source_target_rel
         ON %I."EDGE" (eq_source_id, eq_target_id, eq_rel_type)
         WHERE eq_source_id IS NOT NULL AND eq_target_id IS NOT NULL
           AND eq_rel_type IS NOT NULL',
        v_graph
      );
    END IF;
    EXECUTE format('DROP INDEX IF EXISTS %I.idx_edge_eq_source_target', v_graph);

    EXECUTE format(
      'CREATE INDEX IF NOT EXISTS idx_edge_eq_source_id
       ON %I."EDGE" (eq_source_id) WHERE eq_source_id IS NOT NULL',
      v_graph
    );
    EXECUTE format(
      'CREATE INDEX IF NOT EXISTS idx_edge_eq_target_id
       ON %I."EDGE" (eq_target_id) WHERE eq_target_id IS NOT NULL',
      v_graph
    );

    -- Triggers only when missing (never DROP+CREATE).
    SELECT EXISTS (
      SELECT 1 FROM pg_trigger t
      JOIN pg_class c ON c.oid = t.tgrelid
      JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = v_graph AND c.relname = 'Node'
        AND t.tgname = 'trg_eq_sync_node_id' AND NOT t.tgisinternal
    ) INTO v_has_trg;
    IF NOT v_has_trg THEN
      EXECUTE format(
        'CREATE OR REPLACE FUNCTION %I() RETURNS trigger AS $fn$
         BEGIN
           NEW.eq_node_id := ag_catalog.agtype_to_json(NEW.properties)->>''node_id'';
           RETURN NEW;
         END;
         $fn$ LANGUAGE plpgsql',
        v_graph || '_eq_sync_node_id'
      );
      EXECUTE format(
        'CREATE TRIGGER trg_eq_sync_node_id
         BEFORE INSERT OR UPDATE OF properties ON %I."Node"
         FOR EACH ROW EXECUTE PROCEDURE %I()',
        v_graph, v_graph || '_eq_sync_node_id'
      );
    END IF;

<<<<<<< HEAD
    -- Always refresh EDGE sync fn so eq_rel_type stays aligned (D-30).
    EXECUTE format(
      'CREATE OR REPLACE FUNCTION %I() RETURNS trigger AS $fn$
       BEGIN
         NEW.eq_source_id := ag_catalog.agtype_to_json(NEW.properties)->>''source_id'';
         NEW.eq_target_id := ag_catalog.agtype_to_json(NEW.properties)->>''target_id'';
         NEW.eq_rel_type := UPPER(COALESCE(
=======
    -- Always refresh EDGE sync fn so eq_rel_type stays aligned (D-30 / SPEC-098).
    -- Prefer column values (native INSERT), then properties.
    EXECUTE format(
      'CREATE OR REPLACE FUNCTION %I() RETURNS trigger AS $fn$
       BEGIN
         NEW.eq_source_id := COALESCE(
           NULLIF(TRIM(NEW.eq_source_id), ''''),
           ag_catalog.agtype_to_json(NEW.properties)->>''source_id''
         );
         NEW.eq_target_id := COALESCE(
           NULLIF(TRIM(NEW.eq_target_id), ''''),
           ag_catalog.agtype_to_json(NEW.properties)->>''target_id''
         );
         NEW.eq_rel_type := UPPER(COALESCE(
           NULLIF(TRIM(NEW.eq_rel_type), ''''),
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
           NULLIF(TRIM(ag_catalog.agtype_to_json(NEW.properties)->>''relation_type''), ''''),
           ''RELATED_TO''
         ));
         RETURN NEW;
       END;
       $fn$ LANGUAGE plpgsql',
      v_graph || '_eq_sync_edge_ids'
    );

    SELECT EXISTS (
      SELECT 1 FROM pg_trigger t
      JOIN pg_class c ON c.oid = t.tgrelid
      JOIN pg_namespace n ON n.oid = c.relnamespace
      WHERE n.nspname = v_graph AND c.relname = 'EDGE'
        AND t.tgname = 'trg_eq_sync_edge_ids' AND NOT t.tgisinternal
    ) INTO v_has_trg;
    IF NOT v_has_trg THEN
      EXECUTE format(
        'CREATE TRIGGER trg_eq_sync_edge_ids
         BEFORE INSERT OR UPDATE OF properties ON %I."EDGE"
         FOR EACH ROW EXECUTE PROCEDURE %I()',
        v_graph, v_graph || '_eq_sync_edge_ids'
      );
    END IF;

    -- Single arbiter: drop legacy expression UNIQUEs when eq_* exist.
    IF EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_node_eq_node_id'
    ) THEN
      EXECUTE format('DROP INDEX IF EXISTS %I.idx_node_prop_node_id_unique', v_graph);
    END IF;
    IF EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph
        AND indexname IN ('idx_edge_eq_source_target_rel', 'idx_edge_eq_source_target')
    ) THEN
      EXECUTE format('DROP INDEX IF EXISTS %I.idx_edge_source_target_unique', v_graph);
    END IF;

    RAISE NOTICE 'M092 reconcile: eq_* ready on %', v_graph;
  END LOOP;
END
$$;
