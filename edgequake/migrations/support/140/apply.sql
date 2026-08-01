-- ============================================================================
-- Migration 140 Support: SPEC-098 single EDGE arbiter + relationship spine
-- File: migrations/support/140/apply.sql
-- Invoked by: migration_bootstrap after migration 140 marker is recorded
-- IDEMPOTENT: safe to restart
-- PORTABLE: PG16 / PG17 / PG18 — no major-branched DDL
-- ============================================================================
--
-- WHAT THIS DOES:
--   1. Per AGE graph: ensure 3-col EDGE unique; drop legacy 2-col/expression UNIQUEs
--   2. Refresh EDGE sync trigger (prefer NEW.eq_rel_type, then properties)
--   3. Paginated AGE EDGE → public.relationships spine ensure (LAW-098-1)
--
-- MONITORING:
--   SELECT value FROM server_config WHERE key = 'spec098_edge_arbiter_progress';
-- ============================================================================

SET search_path = public, ag_catalog;

-- ── Part A: single-arbiter reconcile on every AGE graph ─────────────────────
DO $$
DECLARE
  v_graph text;
  v_has_rel boolean;
  v_has_trg boolean;
  major int;
BEGIN
  major := current_setting('server_version_num')::int / 10000;
  RAISE NOTICE 'Migration 140 arbiter: postgres_major=% (supported 16/17/18)', major;

  IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
    RAISE NOTICE 'Migration 140: AGE not installed — skipping arbiter reconcile';
    RETURN;
  END IF;

  FOR v_graph IN
    SELECT name FROM ag_catalog.ag_graph ORDER BY name
  LOOP
    IF NOT EXISTS (
      SELECT 1 FROM pg_tables WHERE schemaname = v_graph AND tablename = 'EDGE'
    ) THEN
      CONTINUE;
    END IF;

    -- Columns (idempotent)
    EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_source_id text', v_graph);
    EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_target_id text', v_graph);
    EXECUTE format('ALTER TABLE %I."EDGE" ADD COLUMN IF NOT EXISTS eq_rel_type text', v_graph);

    -- NULL-only backfill
    EXECUTE format(
      'UPDATE %I."EDGE" SET
         eq_source_id = COALESCE(eq_source_id, ag_catalog.agtype_to_json(properties)->>''source_id''),
         eq_target_id = COALESCE(eq_target_id, ag_catalog.agtype_to_json(properties)->>''target_id''),
         eq_rel_type = COALESCE(
           NULLIF(TRIM(eq_rel_type), ''''),
           UPPER(COALESCE(
             NULLIF(TRIM(ag_catalog.agtype_to_json(properties)->>''relation_type''), ''''),
             ''RELATED_TO''
           ))
         )
       WHERE eq_source_id IS NULL OR eq_target_id IS NULL
          OR eq_rel_type IS NULL OR TRIM(eq_rel_type) = ''''',
      v_graph
    );

    -- 3-col multigraph arbiter
    SELECT EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_edge_eq_source_target_rel'
    ) INTO v_has_rel;
    IF NOT v_has_rel THEN
      EXECUTE format(
        'CREATE UNIQUE INDEX IF NOT EXISTS idx_edge_eq_source_target_rel
         ON %I."EDGE" (eq_source_id, eq_target_id, eq_rel_type)
         WHERE eq_source_id IS NOT NULL AND eq_target_id IS NOT NULL
           AND eq_rel_type IS NOT NULL',
        v_graph
      );
    END IF;

    -- LAW-098-7: drop legacy arbiters when 3-col exists
    EXECUTE format('DROP INDEX IF EXISTS %I.idx_edge_eq_source_target', v_graph);
    EXECUTE format('DROP INDEX IF EXISTS %I.idx_edge_source_target_unique', v_graph);

    IF EXISTS (
      SELECT 1 FROM pg_indexes
      WHERE schemaname = v_graph AND indexname = 'idx_node_eq_node_id'
    ) THEN
      EXECUTE format('DROP INDEX IF EXISTS %I.idx_node_prop_node_id_unique', v_graph);
    END IF;

    -- Prefer column, then properties (prevents trigger collapse of distinct keys)
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

    RAISE NOTICE 'Migration 140: arbiter ready on graph %', v_graph;
  END LOOP;
END $$;

-- ── Part B: AGE EDGE → relationships spine (paginated) ──────────────────────
DO $$
DECLARE
    graph_name    TEXT;
    batch_size    INT := 500;
    offset_val    INT := 0;
    batch_count   INT := 0;
    total_synced  INT := 0;
    age_total     INT := 0;
    inserted      INT;
    graph_cfg     TEXT;
    major         INT;
BEGIN
    major := current_setting('server_version_num')::int / 10000;

    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE NOTICE 'Migration 140 spine: AGE not available — skipping relationship ensure.';
        RETURN;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'relationships'
    ) THEN
        RAISE NOTICE 'Migration 140 spine: relationships table missing — skip.';
        RETURN;
    END IF;

    SELECT value::text INTO graph_cfg
    FROM server_config WHERE key = 'age_graph_name';
    graph_name := COALESCE(NULLIF(TRIM(BOTH '"' FROM COALESCE(graph_cfg, '')), ''), 'edgequake');

    IF NOT EXISTS (
        SELECT 1 FROM ag_catalog.ag_graph WHERE name = graph_name
    ) THEN
        RAISE NOTICE 'Migration 140 spine: AGE graph "%" not found — skipping.', graph_name;
        RETURN;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_tables WHERE schemaname = graph_name AND tablename = 'EDGE'
    ) THEN
        RAISE NOTICE 'Migration 140 spine: %.EDGE missing — skip.', graph_name;
        RETURN;
    END IF;

    EXECUTE format(
        'SELECT COUNT(*)::int FROM %I."EDGE"',
        graph_name
    ) INTO age_total;

    RAISE NOTICE 'Migration 140 relationship spine: starting (graph=%, age_edges=%)',
                 graph_name, age_total;

    LOOP
        EXECUTE format(
            $batch$
            WITH age_batch AS (
                SELECT
                    COALESCE(
                      NULLIF(TRIM(eq_source_id), ''),
                      ag_catalog.agtype_to_json(properties)->>'source_id'
                    ) AS raw_src,
                    COALESCE(
                      NULLIF(TRIM(eq_target_id), ''),
                      ag_catalog.agtype_to_json(properties)->>'target_id'
                    ) AS raw_tgt,
                    UPPER(COALESCE(
                      NULLIF(TRIM(eq_rel_type), ''),
                      NULLIF(TRIM(ag_catalog.agtype_to_json(properties)->>'relation_type'), ''),
                      'RELATED_TO'
                    )) AS rel_type,
                    ag_catalog.agtype_to_json(properties)->>'description' AS description,
                    COALESCE(
                      (ag_catalog.agtype_to_json(properties)->>'weight')::real,
                      1.0
                    ) AS weight,
                    ag_catalog.agtype_to_json(properties)->>'tenant_id' AS tenant_id_str,
                    ag_catalog.agtype_to_json(properties)->>'workspace_id' AS workspace_id_str
                FROM %I."EDGE"
                ORDER BY id
                LIMIT %s OFFSET %s
            ),
            normalized AS (
                SELECT
                    CASE
                        WHEN raw_src ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}::.+'
                        THEN substring(raw_src from position('::' in raw_src) + 2)
                        ELSE raw_src
                    END AS src_name,
                    CASE
                        WHEN raw_tgt ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}::.+'
                        THEN substring(raw_tgt from position('::' in raw_tgt) + 2)
                        ELSE raw_tgt
                    END AS tgt_name,
                    rel_type,
                    description,
                    weight,
                    CASE WHEN tenant_id_str ~ '^[0-9a-fA-F\-]{36}$'
                         THEN tenant_id_str::uuid ELSE NULL END AS tenant_id,
                    CASE WHEN workspace_id_str ~ '^[0-9a-fA-F\-]{36}$'
                         THEN workspace_id_str::uuid ELSE NULL END AS workspace_id
                FROM age_batch
                WHERE raw_src IS NOT NULL AND length(trim(raw_src)) > 0
                  AND raw_tgt IS NOT NULL AND length(trim(raw_tgt)) > 0
            )
            INSERT INTO public.relationships
                (source_id, target_id, tenant_id, workspace_id, relation_type,
                 description, weight, created_at, updated_at)
            SELECT
                es.id,
                et.id,
                n.tenant_id,
                n.workspace_id,
                n.rel_type,
                n.description,
                n.weight,
                NOW(),
                NOW()
            FROM normalized n
            JOIN public.entities es
              ON es.name = n.src_name
             AND es.workspace_id IS NOT DISTINCT FROM n.workspace_id
            JOIN public.entities et
              ON et.name = n.tgt_name
             AND et.workspace_id IS NOT DISTINCT FROM n.workspace_id
            ON CONFLICT (tenant_id, workspace_id, source_id, target_id, relation_type)
                DO UPDATE SET
                    updated_at = NOW()
            $batch$,
            graph_name, batch_size, offset_val
        );

        GET DIAGNOSTICS inserted = ROW_COUNT;
        total_synced := total_synced + inserted;
        batch_count  := batch_count + 1;
        offset_val   := offset_val + batch_size;

        IF batch_count % 10 = 0 THEN
            INSERT INTO server_config (key, value)
            VALUES (
                'spec098_edge_arbiter_progress',
                format(
                    '{"synced": %s, "total": %s, "batch": %s, "postgres_major": %s}',
                    total_synced, age_total, batch_count, major
                )::jsonb
            )
            ON CONFLICT (key) DO UPDATE SET
                value = format(
                    '{"synced": %s, "total": %s, "batch": %s, "postgres_major": %s}',
                    total_synced, age_total, batch_count, major
                )::jsonb;
        END IF;

        EXIT WHEN inserted = 0 AND offset_val >= age_total;
        EXIT WHEN offset_val >= age_total;

        PERFORM pg_sleep(0.05);
    END LOOP;

    INSERT INTO server_config (key, value)
    VALUES (
        'spec098_edge_arbiter_progress',
        format(
            '{"synced": %s, "total": %s, "completed_at": "%s", "postgres_major": %s}',
            total_synced, age_total, NOW()::text, major
        )::jsonb
    )
    ON CONFLICT (key) DO UPDATE SET
        value = format(
            '{"synced": %s, "total": %s, "completed_at": "%s", "postgres_major": %s}',
            total_synced, age_total, NOW()::text, major
        )::jsonb;

    RAISE NOTICE 'Migration 140 relationship spine COMPLETE: % rows touched (age_total=%).',
                 total_synced, age_total;
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Migration 140 relationship spine failed: % — will retry on next apply', SQLERRM;
        INSERT INTO server_config (key, value)
        VALUES (
            'spec098_edge_arbiter_progress',
            format('{"failed_at": "%s", "error": %s}', NOW()::text, to_json(SQLERRM))::jsonb
        )
        ON CONFLICT (key) DO UPDATE SET
            value = format('{"failed_at": "%s", "error": %s}', NOW()::text, to_json(SQLERRM))::jsonb;
END $$;
