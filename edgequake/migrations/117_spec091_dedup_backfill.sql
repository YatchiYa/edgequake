-- SPEC-091 W2: backfill public.ingestion_dedup from the legacy KV hash
-- families (`doc:hash:{ws}:{sha}` → durable 'v1', `staging:hash:{ws}:{sha}`
-- → 'staging'). Idempotent: ON CONFLICT DO NOTHING on every insert; safe to
-- re-run. Rows whose workspace/document id is not a UUID or whose FK target
-- is missing are skipped — KV stays authoritative for them until retirement.
--
-- Minimal `documents` parents are ensured for referenced ids so the
-- reservation FK holds: 'indexed' for durable rows (completed ingests),
-- 'processing' for staging rows (in-flight at upgrade time).

DO $$
DECLARE
    kv_table RECORD;
    uuid_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    -- Durable: doc:hash:{ws}:{sha} → documents parents ('indexed') + 'v1' rows.
    EXECUTE format($f$
        INSERT INTO public.documents (id, workspace_id, content, status)
        SELECT DISTINCT (kv.value #>> '{}')::uuid,
                        split_part(kv.key, ':', 3)::uuid, '', 'indexed'
        FROM %I kv
        WHERE kv.key LIKE 'doc:hash:%%'
          AND split_part(kv.key, ':', 3) ~ $1
          AND (kv.value #>> '{}') ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
        ON CONFLICT (id) DO NOTHING
    $f$, kv_table.tablename) USING uuid_re;

    EXECUTE format($f$
        INSERT INTO public.ingestion_dedup
            (workspace_id, content_hash, pipeline_version, document_id)
        SELECT split_part(kv.key, ':', 3)::uuid,
               split_part(kv.key, ':', 4), 'v1',
               (kv.value #>> '{}')::uuid
        FROM %I kv
        WHERE kv.key LIKE 'doc:hash:%%'
          AND split_part(kv.key, ':', 3) ~ $1
          AND (kv.value #>> '{}') ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
          AND EXISTS (
              SELECT 1 FROM public.documents d
              WHERE d.id = (kv.value #>> '{}')::uuid)
        ON CONFLICT (workspace_id, content_hash, pipeline_version) DO NOTHING
    $f$, kv_table.tablename) USING uuid_re;

    -- Staging: staging:hash:{ws}:{sha} → parents ('processing') + 'staging'.
    EXECUTE format($f$
        INSERT INTO public.documents (id, workspace_id, content, status)
        SELECT DISTINCT (kv.value #>> '{}')::uuid,
                        split_part(kv.key, ':', 3)::uuid, '', 'processing'
        FROM %I kv
        WHERE kv.key LIKE 'staging:hash:%%'
          AND split_part(kv.key, ':', 3) ~ $1
          AND (kv.value #>> '{}') ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
        ON CONFLICT (id) DO NOTHING
    $f$, kv_table.tablename) USING uuid_re;

    EXECUTE format($f$
        INSERT INTO public.ingestion_dedup
            (workspace_id, content_hash, pipeline_version, document_id)
        SELECT split_part(kv.key, ':', 3)::uuid,
               split_part(kv.key, ':', 4), 'staging',
               (kv.value #>> '{}')::uuid
        FROM %I kv
        WHERE kv.key LIKE 'staging:hash:%%'
          AND split_part(kv.key, ':', 3) ~ $1
          AND (kv.value #>> '{}') ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
          AND EXISTS (
              SELECT 1 FROM public.documents d
              WHERE d.id = (kv.value #>> '{}')::uuid)
        ON CONFLICT (workspace_id, content_hash, pipeline_version) DO NOTHING
    $f$, kv_table.tablename) USING uuid_re;
END LOOP;
END $$;
