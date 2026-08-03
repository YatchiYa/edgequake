-- SPEC-091 Wave C: backfill document shells into public.documents from the
-- legacy KV families (`{uuid}-metadata`, `{uuid}-content`,
-- `staging:{uuid}-metadata`, `staging:{uuid}-content`).
--
-- Ordering is deliberate: staging shells first (marked `_shell: "staging"`,
-- status 'processing'), then final metadata/content OVERWRITE them — mirroring
-- the runtime promote semantics where the final write clears the marker.
-- Idempotent; KV stays write-authoritative until the family flag flips, so a
-- re-run simply re-syncs from KV. Rows whose key prefix is not a UUID are
-- skipped (KV fallback covers them until the final drop wave).

DO $$
DECLARE
    kv_table RECORD;
    uuid_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    -- 1. Staging shells (marker + 'processing'); never clobber a richer row.
    EXECUTE format($f$
        INSERT INTO public.documents (id, content, status, metadata)
        SELECT substring(kv.key FROM 9 FOR 36)::uuid, '', 'processing',
               jsonb_set(kv.value, '{_shell}', '"staging"')
        FROM %I kv
        WHERE kv.key LIKE 'staging:%%'
          AND kv.key LIKE '%%-metadata'
          AND substring(kv.key FROM 9 FOR 36) ~ $1
        ON CONFLICT (id) DO UPDATE SET
            metadata = EXCLUDED.metadata,
            status = 'processing',
            updated_at = now()
            WHERE public.documents.metadata IS NULL
               OR public.documents.metadata = '{}'::jsonb
    $f$, kv_table.tablename) USING uuid_re;

    -- 2. Staging content → content column (only when no content yet).
    EXECUTE format($f$
        UPDATE public.documents d
        SET content = kv.value->>'content', updated_at = now()
        FROM %I kv
        WHERE kv.key = 'staging:' || d.id::text || '-content'
          AND d.content = ''
          AND kv.value->>'content' IS NOT NULL
    $f$, kv_table.tablename);

    -- 3. Final metadata OVERWRITES (clears the staging marker — promote parity),
    --    promotes title, and marks the row indexed when it was only a shell.
    EXECUTE format($f$
        INSERT INTO public.documents (id, title, content, status, metadata)
        SELECT left(kv.key, 36)::uuid,
               COALESCE(kv.value->>'title', ''), '',
               'indexed', kv.value
        FROM %I kv
        WHERE kv.key LIKE '%%-metadata'
          AND kv.key NOT LIKE 'staging:%%'
          AND left(kv.key, 36) ~ $1
        ON CONFLICT (id) DO UPDATE SET
            metadata = EXCLUDED.metadata,
            title = CASE WHEN EXCLUDED.title = '' THEN public.documents.title
                         ELSE EXCLUDED.title END,
            status = CASE WHEN public.documents.metadata->>'_shell' = 'staging'
                          THEN 'indexed' ELSE public.documents.status END,
            updated_at = now()
    $f$, kv_table.tablename) USING uuid_re;

    -- 4. Final content → content column (authoritative over staging content).
    EXECUTE format($f$
        UPDATE public.documents d
        SET content = kv.value->>'content', updated_at = now()
        FROM %I kv
        WHERE kv.key = d.id::text || '-content'
          AND kv.value->>'content' IS NOT NULL
    $f$, kv_table.tablename);
END LOOP;
END $$;
