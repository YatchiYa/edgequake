-- SPEC-091 Wave B6: backfill injection entries from the legacy KV family
-- (`injection:{workspace}:{injection_id}-metadata`) into public.documents
-- with metadata->>'source_type' = 'injection'. The row id is the injection id
-- (UUIDv4 at creation); status maps to the documents CHECK constraint
-- ('completed' → 'indexed'). Idempotent: ON CONFLICT refreshes the typed row
-- from the KV record (KV stays write-authoritative until the family retires).
-- Rows with non-UUID ids/workspace or a missing workspace FK are skipped.

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
        SELECT replace(split_part(kv.key, ':', 5), '-metadata', '')::uuid,
               split_part(kv.key, ':', 3)::uuid,
               COALESCE(kv.value->>'name', ''),
               COALESCE(kv.value->>'content', ''),
               CASE kv.value->>'status'
                   WHEN 'completed' THEN 'indexed'
                   WHEN 'indexed'   THEN 'indexed'
                   WHEN 'failed'    THEN 'failed'
                   WHEN 'cancelled' THEN 'failed'
                   WHEN 'pending'   THEN 'pending'
                   ELSE 'processing'
               END,
               jsonb_set(kv.value, '{source_type}', '"injection"')
        FROM %I kv
        WHERE kv.key LIKE 'injection::%%'
          AND kv.key LIKE '%%-metadata'
          AND split_part(kv.key, ':', 3) ~ $1
          AND replace(split_part(kv.key, ':', 5), '-metadata', '') ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 3)::uuid)
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
