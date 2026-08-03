-- SPEC-091 W4 (ships alone, human-gated): retire legacy chunk vectors.
--
-- ┌──────────────────────────────────────────────────────────────────────┐
-- │ IRREVERSIBLE. Every legacy chunk vector ({doc}-chunk-{n}) has been    │
-- │ copied to the typed SSOT by the migration engine:                     │
-- │   • chunk embeddings  → public.chunk_embeddings   (w3 engine job)     │
-- │ Scope is CHUNKS ONLY: entity/relationship/community-report vectors    │
-- │ (different namespaces / key shapes) are untouched and remain on       │
-- │ legacy eq_*_vectors until a later wave gives them a typed home.       │
-- │ Rollback after this migration = RESTORE FROM BACKUP (spec law).       │
-- └──────────────────────────────────────────────────────────────────────┘
--
-- Gate: `edgequake migrate --confirm-drop` (mirrors migration 125). The
-- advisor `drop vector-legacy` GuardedAction and `VectorPosture.retirable()`
-- report readiness; the physical DROP runs here, never in the engine.
--
-- Safety: the drop is guarded — the DO block aborts if any eq_%_vectors
-- table still holds a chunk row NOT yet represented in chunk_embeddings.
-- The guard verifies the *typed side* per row (it does not trust key
-- prefixes), mirroring verify_chunk_embedding_backfill's coverage rule:
--   a chunk row {doc}-chunk-{n} is covered iff (document_id, chunk_index)
--   resolves to a public.chunks row that has a chunk_embeddings row.
--
-- Then it (1) DELETEs the covered chunk rows and (2) DROPs any eq_%_vectors
-- table that becomes empty (chunk-dedicated). Tables that still hold
-- entity/relationship/report vectors are left fully intact.

-- ── 1. Guard: abort if any legacy chunk row is not covered in typed SSOT ──
DO $$
DECLARE
    vec_rec   RECORD;
    durable   BIGINT;
    -- Canonical 36-char UUID document prefix of a chunk key.
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
                'SPEC-091 W4 ABORT: % still holds % legacy chunk vectors not yet in chunk_embeddings. Run the w3-chunk-embedding-backfill engine job (EDGEQUAKE_MIGRATION_MODE=automatic), flip EDGEQUAKE_VECTOR_BACKEND=chunk_embeddings, then re-apply with --confirm-drop.',
                vec_rec.tbl, durable;
        END IF;
    END LOOP;
END $$;

-- ── 2. Delete covered chunk rows; drop now-empty (chunk-dedicated) tables ──
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
        -- Remove only the chunk rows (entity/rel/report key shapes untouched).
        EXECUTE format(
            'DELETE FROM public.%I WHERE id LIKE ''%%-chunk-%%''',
            vec_rec.tbl
        );

        EXECUTE format('SELECT count(*) FROM public.%I', vec_rec.tbl)
        INTO remaining;

        IF remaining = 0 THEN
            -- Chunk-dedicated relation: nothing else lives here → drop it.
            EXECUTE format('DROP TABLE public.%I CASCADE', vec_rec.tbl);
            RAISE NOTICE 'SPEC-091 W4: dropped chunk-dedicated %', vec_rec.tbl;
        ELSE
            RAISE NOTICE 'SPEC-091 W4: % still holds % non-chunk vectors — kept (out of W4 scope)',
                vec_rec.tbl, remaining;
        END IF;
    END LOOP;
END $$;
