-- SPEC-091 IW2 (ships alone, human-gated): retire the entire legacy vector fleet.
--
-- ┌──────────────────────────────────────────────────────────────────────┐
-- │ IRREVERSIBLE. Every legacy vector row in public.eq_%_vectors has been │
-- │ copied to typed SSOT by the migration engine:                         │
-- │   • chunks         → public.chunk_embeddings   (w3 engine job)        │
-- │   • entities       → public.entity_embeddings  (iw2 engine job)       │
-- │   • relationships  → public.relationship_embeddings (iw2)             │
-- │   • reports        → public.report_embeddings  (iw2)                  │
-- │ Rollback after this migration = RESTORE FROM BACKUP (spec law).       │
-- └──────────────────────────────────────────────────────────────────────┘
--
-- Gate: `edgequake migrate --confirm-drop` (mirrors migrations 125/126).
-- The advisor `drop vector-fleet` GuardedAction reports readiness; the
-- physical DROP runs here, never in the engine.
--
-- Safety: abort if any eq_%_vectors row is NOT represented in its typed
-- table. Coverage rules mirror the engine verify helpers:
--   chunk:         {doc}-chunk-{n} → chunks + chunk_embeddings
--   entity:        entity:{name}   → entities.name + entity_embeddings
--   relationship:  {s}->{t}:{type} → relationships + relationship_embeddings
--   report:        community_report:{n} → report_embeddings.report_id

-- ── 1. Guard: abort if any legacy row lacks typed coverage ──
DO $$
DECLARE
    vec_rec   RECORD;
    durable   BIGINT;
    uuid_re   CONSTANT TEXT := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}';
BEGIN
    FOR vec_rec IN
        SELECT c.relname AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_vectors' ESCAPE '\'
    LOOP
        -- Uncovered chunk rows
        EXECUTE format($q$
            SELECT count(*) FROM public.%I v
            WHERE v.id ~ '%s-chunk-[0-9]+$'
              AND NOT EXISTS (
                    SELECT 1
                    FROM public.chunks c
                    JOIN public.chunk_embeddings ce ON ce.chunk_id = c.id
                    WHERE c.document_id = left(v.id, 36)::uuid
                      AND c.chunk_index  = substring(v.id from 44)::int)
            $q$,
            vec_rec.tbl,
            uuid_re
        ) INTO durable;
        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % holds % legacy chunk vectors not in chunk_embeddings. Run w3-chunk-embedding-backfill, then re-apply with --confirm-drop.',
                vec_rec.tbl, durable;
        END IF;

        -- Uncovered entity rows (entity: prefix)
        EXECUTE format($q$
            SELECT count(*) FROM public.%I v
            WHERE v.id LIKE 'entity:%%'
              AND NOT EXISTS (
                    SELECT 1
                    FROM public.entities e
                    JOIN public.entity_embeddings ee ON ee.entity_id = e.id
                    WHERE e.name = substring(v.id from 8))
            $q$,
            vec_rec.tbl
        ) INTO durable;
        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % holds % legacy entity vectors not in entity_embeddings. Run iw2-fleet-embedding-backfill, then re-apply with --confirm-drop.',
                vec_rec.tbl, durable;
        END IF;

        -- Uncovered relationship rows ({src}->{tgt}:{type})
        EXECUTE format($q$
            SELECT count(*) FROM public.%I v
            WHERE v.id ~ '^.+->.+:.+$'
              AND v.id NOT LIKE 'entity:%%'
              AND v.id NOT LIKE 'community_report:%%'
              AND NOT EXISTS (
                    SELECT 1
                    FROM public.relationships r
                    JOIN public.entities es ON es.id = r.source_id
                    JOIN public.entities et ON et.id = r.target_id
                    JOIN public.relationship_embeddings re ON re.relationship_id = r.id
                    WHERE r.relation_type = split_part(v.id, ':', 2)
                      AND es.name = split_part(split_part(v.id, '->', 1), ':', 1)
                      AND et.name = split_part(split_part(v.id, '->', 2), ':', 1))
            $q$,
            vec_rec.tbl
        ) INTO durable;
        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % holds % legacy relationship vectors not in relationship_embeddings. Run iw2-fleet-embedding-backfill, then re-apply with --confirm-drop.',
                vec_rec.tbl, durable;
        END IF;

        -- Uncovered report rows
        EXECUTE format($q$
            SELECT count(*) FROM public.%I v
            WHERE v.id LIKE 'community_report:%%'
              AND NOT EXISTS (
                    SELECT 1
                    FROM public.report_embeddings re
                    WHERE re.report_id = v.id)
            $q$,
            vec_rec.tbl
        ) INTO durable;
        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % holds % legacy report vectors not in report_embeddings. Run iw2-fleet-embedding-backfill, then re-apply with --confirm-drop.',
                vec_rec.tbl, durable;
        END IF;

        -- Residual non-classifiable rows (fail closed)
        EXECUTE format($q$
            SELECT count(*) FROM public.%I v
            WHERE v.id NOT LIKE 'entity:%%'
              AND v.id NOT LIKE 'community_report:%%'
              AND v.id !~ '%s-chunk-[0-9]+$'
              AND v.id !~ '^.+->.+:.+$'
            $q$,
            vec_rec.tbl,
            uuid_re
        ) INTO durable;
        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % holds % unclassified legacy vector rows. Classify or quarantine before drop.',
                vec_rec.tbl, durable;
        END IF;
    END LOOP;
END $$;

-- ── 2. Delete all covered rows; drop empty eq_%_vectors tables ──
DO $$
DECLARE
    vec_rec   RECORD;
    remaining BIGINT;
BEGIN
    FOR vec_rec IN
        SELECT c.relname AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_vectors' ESCAPE '\'
    LOOP
        EXECUTE format('DELETE FROM public.%I', vec_rec.tbl);

        EXECUTE format('SELECT count(*) FROM public.%I', vec_rec.tbl)
        INTO remaining;

        IF remaining = 0 THEN
            EXECUTE format('DROP TABLE public.%I CASCADE', vec_rec.tbl);
            RAISE NOTICE 'SPEC-091 IW2: dropped legacy vector table %', vec_rec.tbl;
        ELSE
            RAISE EXCEPTION
                'SPEC-091 IW2 ABORT: % still holds % rows after DELETE (guard bug).',
                vec_rec.tbl, remaining;
        END IF;
    END LOOP;
END $$;

-- ── 3. Drop orphaned stats relations + hot-ANN registry ──
DO $$
DECLARE
    stats_rec RECORD;
BEGIN
    FOR stats_rec IN
        SELECT c.relname AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_vectors\_stats%' ESCAPE '\'
    LOOP
        EXECUTE format('DROP TABLE IF EXISTS public.%I CASCADE', stats_rec.tbl);
        RAISE NOTICE 'SPEC-091 IW2: dropped orphaned stats table %', stats_rec.tbl;
    END LOOP;
END $$;

DROP TABLE IF EXISTS public.eq_hot_ann_workspaces CASCADE;
