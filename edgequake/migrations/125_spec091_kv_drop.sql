-- SPEC-091 Wave D (final, ships alone): DROP the generic KV store.
--
-- ┌──────────────────────────────────────────────────────────────────────┐
-- │ IRREVERSIBLE. Every durable KV family has been copied to typed       │
-- │ tables by earlier migrations:                                        │
-- │   • chunk text            → public.chunks        (066/068 + engine)  │
-- │   • dedup reservations    → public.ingestion_dedup        (117)      │
-- │   • wsdoc membership      → documents.workspace_id        (118)      │
-- │   • artifacts/lineage/MM  → public.document_artifacts     (119)      │
-- │   • auth shim             → purged                      (120)        │
-- │   • injection metadata    → public.documents              (121)      │
-- │   • doc/staging shells    → public.documents              (122)      │
-- │ Transient families (pipeline checkpoints, extraction snapshots, LLM  │
-- │ caches) were drained in 124; caches recompute on demand.             │
-- │ Rollback after this migration = RESTORE FROM BACKUP (spec law).      │
-- └──────────────────────────────────────────────────────────────────────┘
--
-- Safety: the drop is guarded — the DO block aborts if any KV table still
-- holds durable data that is NOT yet represented in its typed SSOT. The
-- guard verifies the *typed side* per family (it does not trust key
-- prefixes alone), so it blocks only on genuine data-loss risk and lets
-- already-backfilled (redundant) rows pass:
--   • chunk text   {uuid}-chunk-{n}     durable unless (document_id,
--                                     chunk_index) exists in public.chunks
--                                     with non-empty content — closes the
--                                     EC-34 gap where uploads that ran with
--                                     chunk authority = `kv` left text in KV
--                                     only and the empty `chunks` spine would
--                                     silently lose it on drop;
--   • doc shells   {uuid}-metadata/-content     durable unless the document
--                                     row exists in public.documents;
--   • lineage/MM   {uuid}-lineage/-multimodal-* durable unless the matching
--                                     kind row exists in public.document_artifacts;
--   • dedup / wsdoc / injection prefixes are presence-conservative in the
--     guard below, AFTER a verified purge of keys already represented in
--     typed SSOT (117/118/121). Keys the backfills skipped (bad UUID / missing
--     FK) remain and correctly abort the drop.
-- Transient keys (checkpoints, caches) are excluded: their loss is by design.

-- ── 0. Verified purge of presence-conservative families ─────────────────────
-- Backfills 117/118/121 copy these into typed tables but leave the KV keys.
-- Without this purge, every in-flight staging:hash / wsdoc row blocks 125
-- forever (soak proved: 6 staging:hash keys → Wave D ABORT after green
-- shell/chunk guards). Only delete keys whose typed SSOT row exists.
DO $$
DECLARE
    kv_table RECORD;
    uuid_re CONSTANT text :=
        '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
BEGIN
    FOR kv_table IN
        SELECT tablename FROM pg_tables
        WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv' ESCAPE '\'
    LOOP
        -- doc:hash:{ws}:{sha} → public.ingestion_dedup (v1)
        EXECUTE format($f$
            DELETE FROM public.%I k
            WHERE k.key LIKE 'doc:hash:%%'
              AND split_part(k.key, ':', 3) ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.ingestion_dedup d
                  WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                    AND d.content_hash = split_part(k.key, ':', 4)
                    AND d.pipeline_version = 'v1')
        $f$, kv_table.tablename) USING uuid_re;

        -- staging:hash:{ws}:{sha} → public.ingestion_dedup (staging)
        EXECUTE format($f$
            DELETE FROM public.%I k
            WHERE k.key LIKE 'staging:hash:%%'
              AND split_part(k.key, ':', 3) ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.ingestion_dedup d
                  WHERE d.workspace_id = split_part(k.key, ':', 3)::uuid
                    AND d.content_hash = split_part(k.key, ':', 4)
                    AND d.pipeline_version = 'staging')
        $f$, kv_table.tablename) USING uuid_re;

        -- wsdoc:{ws}:{doc} → public.documents.workspace_id
        EXECUTE format($f$
            DELETE FROM public.%I k
            WHERE k.key LIKE 'wsdoc:%%'
              AND split_part(k.key, ':', 2) ~ $1
              AND split_part(k.key, ':', 3) ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.documents d
                  WHERE d.id = split_part(k.key, ':', 3)::uuid
                    AND d.workspace_id = split_part(k.key, ':', 2)::uuid)
        $f$, kv_table.tablename) USING uuid_re;

        -- injection::{ws}::{id}-metadata → public.documents (source_type=injection)
        EXECUTE format($f$
            DELETE FROM public.%I k
            WHERE k.key LIKE 'injection::%%'
              AND k.key LIKE '%%-metadata'
              AND split_part(k.key, ':', 3) ~ $1
              AND replace(split_part(k.key, ':', 5), '-metadata', '') ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.documents d
                  WHERE d.id = replace(split_part(k.key, ':', 5), '-metadata', '')::uuid
                    AND d.metadata->>'source_type' = 'injection')
        $f$, kv_table.tablename) USING uuid_re;
    END LOOP;
END $$;

DO $$
DECLARE
    kv_rec   RECORD;
    durable  BIGINT;
    -- Matches a canonical 36-char UUID (used to extract the document id from
    -- legacy shell/lineage/MM keys, and as the chunk-key document prefix).
    uuid_re  CONSTANT TEXT :=
        '([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})';
BEGIN
    FOR kv_rec IN
        SELECT c.relname AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND c.relname LIKE 'eq\_%\_kv' ESCAPE '\'
          AND c.relname NOT LIKE '%\_kv\_stats' ESCAPE '\'
    LOOP
        EXECUTE format($q$
            SELECT count(*) FROM public.%I k WHERE
              -- chunk text: durable unless (document_id, chunk_index) in chunks
              (k.key ~ '^%s-chunk-[0-9]+$'
                 AND COALESCE(k.value->>'content', '') <> ''
                 AND NOT EXISTS (
                        SELECT 1 FROM public.chunks c
                        WHERE c.document_id = left(k.key, 36)::uuid
                          AND c.chunk_index  = substring(k.key from 44)::int))
              -- metadata/content shells: durable unless the document exists
              OR ((k.key LIKE '%%-metadata' OR k.key LIKE '%%-content')
                 AND NOT EXISTS (
                        SELECT 1 FROM public.documents d
                        WHERE d.id::text = substring(k.key from '%s')))
              -- lineage: durable unless the artifact exists
              OR (k.key LIKE '%%-lineage'
                 AND NOT EXISTS (
                        SELECT 1 FROM public.document_artifacts a
                        WHERE a.kind = 'lineage'
                          AND a.document_id::text = substring(k.key from '%s')))
              -- multimodal manifest
              OR (k.key LIKE '%%-multimodal-manifest'
                 AND NOT EXISTS (
                        SELECT 1 FROM public.document_artifacts a
                        WHERE a.kind = 'multimodal-manifest'
                          AND a.document_id::text = substring(k.key from '%s')))
              -- multimodal chunk dump
              OR (k.key LIKE '%%-multimodal-chunks'
                 AND NOT EXISTS (
                        SELECT 1 FROM public.document_artifacts a
                        WHERE a.kind = 'multimodal-chunks'
                          AND a.document_id::text = substring(k.key from '%s')))
              -- conservative presence: dedup / wsdoc / injection (post-purge)
              OR k.key LIKE 'doc:hash:%%'
              OR k.key LIKE 'staging:hash:%%'
              OR k.key LIKE 'wsdoc:%%'
              OR k.key LIKE 'injection::%%'
            $q$,
            kv_rec.tbl,
            uuid_re,   -- chunk document prefix
            uuid_re,   -- metadata/content shells
            uuid_re,   -- lineage
            uuid_re,   -- multimodal-manifest
            uuid_re    -- multimodal-chunks
        ) INTO durable;

        IF durable > 0 THEN
            RAISE EXCEPTION
                'SPEC-091 Wave D ABORT: % still holds % un-migrated durable KV rows (chunk text, shells, lineage/MM, or dedup/wsdoc/injection not yet in their typed tables). Run the family backfills (117-122) or the migration engine, then re-apply.',
                kv_rec.tbl, durable;
        END IF;
    END LOOP;
END $$;

-- Guard passed: drop the relations, their stats sidecars, and the
-- trigger machinery created by the (now deleted) runtime DDL.
DO $$
DECLARE
    obj RECORD;
BEGIN
    -- 1. Drop KV + KV-stats tables (CASCADE removes their triggers).
    FOR obj IN
        SELECT c.relname AS tbl
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND (c.relname LIKE 'eq\_%\_kv' ESCAPE '\'
               OR c.relname LIKE 'eq\_%\_kv\_stats' ESCAPE '\')
    LOOP
        EXECUTE format('DROP TABLE public.%I CASCADE', obj.tbl);
        RAISE NOTICE 'SPEC-091: dropped %', obj.tbl;
    END LOOP;

    -- 2. Drop the row-count stats functions (naming: eq_{prefix}_kv_stats_*).
    FOR obj IN
        SELECT p.proname AS fn
        FROM pg_proc p
        JOIN pg_namespace n ON n.oid = p.pronamespace
        WHERE n.nspname = 'public'
          AND p.proname LIKE 'eq\_%\_kv\_stats\_%' ESCAPE '\'
    LOOP
        EXECUTE format('DROP FUNCTION public.%I() CASCADE', obj.fn);
        RAISE NOTICE 'SPEC-091: dropped function %', obj.fn;
    END LOOP;
END $$;
