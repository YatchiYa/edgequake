-- SPEC-091 Wave B3: backfill workspace membership into public.documents from
-- the legacy KV index family (`wsdoc:{workspace_id}:{document_id}`).
-- Idempotent: ON CONFLICT repairs only a NULL workspace_id (never overwrites
-- an existing scope) and never touches pipeline-managed status. Rows whose
-- workspace/document id is not a UUID, or whose workspace FK target is
-- missing, are skipped — the KV index stays authoritative for them until the
-- final KV drop wave.
--
-- SPEC-110: DISTINCT ON (doc_id) collapses the membership index to one row per
-- document. The same document_id legitimately appears under several
-- workspace_ids (wsdoc is a workspace→document index), so a plain SELECT
-- DISTINCT over the whole tuple would emit the same conflict key twice and
-- trip PostgreSQL's "ON CONFLICT DO UPDATE command cannot affect row a second
-- time". Deterministic ORDER BY makes the winning workspace stable; COALESCE
-- only fills a NULL scope, so the pick is harmless for rows that already carry
-- a workspace_id.

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
        INSERT INTO public.documents (id, workspace_id, content, status)
        SELECT DISTINCT ON (doc_id) doc_id, ws_id, '', 'indexed'
        FROM (
            SELECT split_part(kv.key, ':', 3)::uuid AS doc_id,
                   split_part(kv.key, ':', 2)::uuid AS ws_id
            FROM %I kv
            WHERE kv.key LIKE 'wsdoc:%%'
              AND split_part(kv.key, ':', 2) ~ $1
              AND split_part(kv.key, ':', 3) ~ $1
              AND EXISTS (
                  SELECT 1 FROM public.workspaces w
                  WHERE w.workspace_id = split_part(kv.key, ':', 2)::uuid)
        ) src
        ORDER BY doc_id, ws_id
        ON CONFLICT (id) DO UPDATE SET
            workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)
    $f$, kv_table.tablename) USING uuid_re;
END LOOP;
END $$;
