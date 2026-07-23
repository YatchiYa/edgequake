-- Migration 096: SPEC-083 Sprint 1 — real RLS (S-03…S-06)
--
-- WHY:
--   1. set_config(..., is_local=true) is transaction-local. Autocommit
--      SELECT set_tenant_context() clears GUCs before the next statement.
--      Application code MUST call set_tenant_context inside BEGIN…COMMIT
--      (see edgequake-storage rls::with_rls_transaction).
--   2. ENABLE without FORCE lets the table owner bypass policies.
--   3. Fail-open `tenant_id IS NULL OR …` leaked rows with NULL tenant.
--   4. document_originals had no RLS at all (binary original leak).
--
-- GUC contract (SSOT): app.current_tenant_id / app.current_workspace_id /
-- app.current_user_id via set_tenant_context() — always inside a transaction.

SET search_path = public;

-- ---------------------------------------------------------------------------
-- document_originals: ENABLE + FORCE + workspace fail-closed policy
-- ---------------------------------------------------------------------------
ALTER TABLE IF EXISTS document_originals ENABLE ROW LEVEL SECURITY;
ALTER TABLE IF EXISTS document_originals FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS document_originals_workspace_isolation ON document_originals;
CREATE POLICY document_originals_workspace_isolation ON document_originals
    FOR ALL
    USING (
        current_workspace_id() IS NOT NULL
        AND workspace_id = current_workspace_id()
    )
    WITH CHECK (
        current_workspace_id() IS NOT NULL
        AND workspace_id = current_workspace_id()
    );

-- ---------------------------------------------------------------------------
-- FORCE RLS on core tenant tables (owner must not bypass)
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    t text;
BEGIN
    FOREACH t IN ARRAY ARRAY[
        'documents',
        'entities',
        'relationships',
        'chunks',
        'conversation_history',
        'tasks',
        'conversations',
        'messages',
        'folders',
        'pdf_documents',
        'audit_logs'
    ]
    LOOP
        IF to_regclass(format('public.%I', t)) IS NOT NULL THEN
            EXECUTE format('ALTER TABLE public.%I ENABLE ROW LEVEL SECURITY', t);
            EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', t);
        END IF;
    END LOOP;
END
$$;

-- ---------------------------------------------------------------------------
-- Fail-closed policies: drop legacy NULL-tenant OR branches
-- ---------------------------------------------------------------------------

-- documents
DROP POLICY IF EXISTS documents_tenant_isolation ON documents;
CREATE POLICY documents_tenant_isolation ON documents
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

-- entities
DROP POLICY IF EXISTS entities_tenant_isolation ON entities;
CREATE POLICY entities_tenant_isolation ON entities
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

-- relationships
DROP POLICY IF EXISTS relationships_tenant_isolation ON relationships;
CREATE POLICY relationships_tenant_isolation ON relationships
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

-- chunks
DROP POLICY IF EXISTS chunks_tenant_isolation ON chunks;
CREATE POLICY chunks_tenant_isolation ON chunks
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

-- conversation_history
DROP POLICY IF EXISTS conversation_history_tenant_isolation ON conversation_history;
CREATE POLICY conversation_history_tenant_isolation ON conversation_history
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

-- tasks
DROP POLICY IF EXISTS tasks_tenant_isolation ON tasks;
CREATE POLICY tasks_tenant_isolation ON tasks
    FOR ALL
    USING (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
        AND (
            current_workspace_id() IS NULL
            OR workspace_id = current_workspace_id()
        )
    )
    WITH CHECK (
        current_tenant_id() IS NOT NULL
        AND tenant_id = current_tenant_id()
    );

COMMENT ON FUNCTION set_tenant_context(UUID, UUID, UUID) IS
    'SPEC-083: Sets app.current_* GUCs with is_local=true. MUST be called inside an open transaction (with_rls_transaction); autocommit clears GUCs immediately.';
