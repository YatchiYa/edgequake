-- SPEC-091 Wave B6: backfill injection entries from the legacy KV family
-- (`injection:{workspace}:{injection_id}-metadata`) into public.documents
-- with metadata->>'source_type' = 'injection'. The row id is the injection id
-- (UUIDv4 at creation); status maps to the documents CHECK constraint
-- ('completed' → 'indexed'). Idempotent: ON CONFLICT refreshes the typed row
-- from the KV record (KV stays write-authoritative until the family retires).
-- Rows with non-UUID ids/workspace or a missing workspace FK are skipped.
--
-- SPEC-110: DISTINCT ON (inj_id) guards against the same injection id appearing
-- under more than one workspace-prefixed key — emitting it twice in one
-- statement would trip "ON CONFLICT DO UPDATE command cannot affect row a
-- second time". Deterministic ORDER BY keeps the winning row stable.

DO $$
DECLARE
    kv_table RECORD;
    uuid_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    EXECUTE format($f$
        INSERT INTO public.documents (id, workspace_id, title, content, status, metadata)
        SELECT DISTINCT ON (inj_id) inj_id, ws_id, name, content, status, metadata
        FROM (
            SELECT replace(split_part(kv.key, ':', 5), '-metadata', '')::uuid AS inj_id,
                   split_part(kv.key, ':', 3)::uuid AS ws_id,
                   COALESCE(kv.value->>'name', '') AS name,
                   COALESCE(kv.value->>'content', '') AS content,
                   CASE kv.value->>'status'
                       WHEN 'completed' THEN 'indexed'
                       WHEN 'indexed'   THEN 'indexed'
                       WHEN 'failed'    THEN 'failed'
                       WHEN 'cancelled' THEN 'failed'
                       WHEN 'pending'   THEN 'pending'
                       ELSE 'processing'
                   END AS status,
                   jsonb_set(kv.value, '{source_type}', '"injection"') AS metadata
            FROM %I kv
            WHERE kv.key LIKE 'injection::%%'
              AND kv.key LIKE '%%-metadata'
              AND split_part(kv.key, ':', 3) ~ $1
              AND replace(split_part(kv.key, ':', 5), '-metadata', '') ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.workspaces w
                  WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
        ) src
        ORDER BY inj_id, ws_id
        ON CONFLICT (id) DO UPDATE SET
            workspace_id = EXCLUDED.workspace_id,
            title = EXCLUDED.title,
            content = EXCLUDED.content,
            status = EXCLUDED.status,
            metadata = EXCLUDED.metadata,
            updated_at = now()
    $f$, kv_table.tablename) USING uuid_re;
END LOOP;
END $$;
