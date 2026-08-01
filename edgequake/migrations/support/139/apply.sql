-- ============================================================================
-- Migration 139 Support: SPEC-098 entity spine ensure
-- File: migrations/support/139/apply.sql
-- Invoked by: migration_bootstrap after migration 139 marker is recorded
-- IDEMPOTENT: safe to restart (ON CONFLICT DO UPDATE)
-- PORTABLE: PG16 / PG17 / PG18 — no major-branched DDL
-- ============================================================================
--
-- WHAT THIS DOES:
--   Ensures bare `entities.name` rows exist for AGE graph vertices so typed
--   fleet mirror can resolve entity:NAME → entities.id (LAW-098-1).
--   Strips `{uuid}::` workspace scope from AGE node_id when present.
--
-- MONITORING:
--   SELECT value FROM server_config WHERE key = 'spec098_spine_ensure_progress';
-- ============================================================================

SET search_path = public, ag_catalog;

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
    RAISE NOTICE 'Migration 139 spine ensure: postgres_major=% (supported 16/17/18)', major;

    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE NOTICE 'Migration 139: AGE not available — skipping spine ensure.';
        RETURN;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'entities'
          AND column_name = 'sync_status'
    ) THEN
        RAISE NOTICE 'Migration 139: entities.sync_status missing — run migration 039 first.';
        RETURN;
    END IF;

    SELECT value::text INTO graph_cfg
    FROM server_config WHERE key = 'age_graph_name';
    graph_name := COALESCE(NULLIF(TRIM(BOTH '"' FROM COALESCE(graph_cfg, '')), ''), 'edgequake');

    IF NOT EXISTS (
        SELECT 1 FROM ag_catalog.ag_graph WHERE name = graph_name
    ) THEN
        RAISE NOTICE 'Migration 139: AGE graph "%" not found — skipping.', graph_name;
        RETURN;
    END IF;

    EXECUTE format(
        'SELECT COUNT(*)::int FROM %I."_ag_label_vertex"',
        graph_name
    ) INTO age_total;

    RAISE NOTICE 'Migration 139 spine ensure: starting (graph=%, age_nodes=%)', graph_name, age_total;

    LOOP
        EXECUTE format(
            $batch$
            WITH age_batch AS (
                SELECT
                    ag_catalog.agtype_to_json(properties)->>'node_id'        AS raw_name,
                    ag_catalog.agtype_to_json(properties)->>'entity_type'    AS entity_type,
                    ag_catalog.agtype_to_json(properties)->>'description'    AS description,
                    ag_catalog.agtype_to_json(properties)->>'tenant_id'      AS tenant_id_str,
                    ag_catalog.agtype_to_json(properties)->>'workspace_id'   AS workspace_id_str
                FROM %I."_ag_label_vertex"
                ORDER BY id
                LIMIT %s OFFSET %s
            ),
            normalized AS (
                SELECT
                    CASE
                        WHEN raw_name ~ '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}::.+'
                        THEN substring(raw_name from position('::' in raw_name) + 2)
                        ELSE raw_name
                    END AS name,
                    COALESCE(NULLIF(entity_type, ''), 'UNKNOWN') AS entity_type,
                    description,
                    CASE WHEN tenant_id_str ~ '^[0-9a-fA-F\-]{36}$'
                         THEN tenant_id_str::uuid ELSE NULL END AS tenant_id,
                    CASE WHEN workspace_id_str ~ '^[0-9a-fA-F\-]{36}$'
                         THEN workspace_id_str::uuid ELSE NULL END AS workspace_id
                FROM age_batch
                WHERE raw_name IS NOT NULL AND length(trim(raw_name)) > 0
            )
            INSERT INTO public.entities
                (name, entity_type, description, tenant_id, workspace_id,
                 sync_status, created_at, updated_at)
            SELECT
                n.name,
                n.entity_type,
                n.description,
                n.tenant_id,
                n.workspace_id,
                'synced',
                NOW(),
                NOW()
            FROM normalized n
            ON CONFLICT (tenant_id, workspace_id, name)
                DO UPDATE SET
                    sync_status = 'synced',
                    updated_at  = NOW()
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
                'spec098_spine_ensure_progress',
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
        'spec098_spine_ensure_progress',
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

    RAISE NOTICE 'Migration 139 spine ensure COMPLETE: % rows touched (age_total=%).',
        total_synced, age_total;

EXCEPTION WHEN OTHERS THEN
    RAISE WARNING 'Migration 139 spine ensure FAILED at batch % (offset %): %',
        batch_count, offset_val, SQLERRM;
END $$;
