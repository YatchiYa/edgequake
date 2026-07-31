-- SPEC-091 IW1 (GAP-091-24): hot-path index gaps on `public.documents`.
--
-- Three targeted indexes, all regular CREATE INDEX (repo convention: sqlx
-- migrations run inside a transaction, so CONCURRENTLY is not possible here;
-- operators with very large `documents` tables may rebuild these CONCURRENTLY
-- out-of-band following the migration-015 precedent):
--
-- 1. `idx_documents_workspace_created` — the interactive listing query
--    (`document_read_model.rs`: `WHERE workspace_id = $1 … ORDER BY
--    created_at DESC`) previously seq-scanned + sorted every workspace page.
--    The composite serves equality on `workspace_id` AND the descending sort
--    from one index scan.
--
-- 2. `idx_documents_metadata_workspace_expr` — the legacy-JSONB workspace
--    fallback branch of the workspace delete (`metadata->>'workspace_id' =
--    $3`, rewritten to UNION form in the same change set) previously had no
--    index at all.
--
-- 3. `idx_documents_staging_shell` — the staging-shell scan
--    (`shell_staging_keys`: `WHERE metadata->>'_shell' = 'staging'` with
--    keyset pagination on `id`) — a partial index covering ONLY staging rows,
--    so it stays tiny even on a million-document table.

BEGIN;

CREATE INDEX IF NOT EXISTS idx_documents_workspace_created
    ON public.documents (workspace_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_documents_metadata_workspace_expr
    ON public.documents ((metadata->>'workspace_id'));

CREATE INDEX IF NOT EXISTS idx_documents_staging_shell
    ON public.documents (id)
    WHERE metadata->>'_shell' = 'staging';

COMMIT;
