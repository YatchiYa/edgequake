-- Move workspace_metrics_history into public (F1).
-- 016 created it unqualified while AGE's search_path (ag_catalog first) was
-- active, so on those installs it lives in ag_catalog. Runtime queries use
-- search_path=public and fail with "relation does not exist".
-- Idempotent: move if only the ag_catalog copy exists, else create in public.
DO $$
BEGIN
    IF to_regclass('public.workspace_metrics_history') IS NULL
       AND to_regclass('ag_catalog.workspace_metrics_history') IS NOT NULL THEN
        ALTER TABLE ag_catalog.workspace_metrics_history SET SCHEMA public;
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS public.workspace_metrics_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    trigger_type TEXT NOT NULL DEFAULT 'event',
    document_count BIGINT NOT NULL DEFAULT 0,
    chunk_count BIGINT NOT NULL DEFAULT 0,
    entity_count BIGINT NOT NULL DEFAULT 0,
    relationship_count BIGINT NOT NULL DEFAULT 0,
    embedding_count BIGINT NOT NULL DEFAULT 0,
    storage_bytes BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT fk_metrics_workspace
        FOREIGN KEY (workspace_id)
        REFERENCES public.workspaces(workspace_id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_metrics_workspace_time
    ON public.workspace_metrics_history(workspace_id, recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_metrics_recorded_at
    ON public.workspace_metrics_history(recorded_at);
CREATE INDEX IF NOT EXISTS idx_metrics_trigger_type
    ON public.workspace_metrics_history(trigger_type);
