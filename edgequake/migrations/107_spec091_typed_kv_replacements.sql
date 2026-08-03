-- SPEC-091 Wave-2: typed KV replacements (ingestion dedup + compensation quarantine)

CREATE TABLE IF NOT EXISTS ingestion_dedup (
    id                uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id      uuid NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    content_hash      varchar(64) NOT NULL,
    pipeline_version  text NOT NULL,
    document_id       uuid REFERENCES documents(id) ON DELETE SET NULL,
    created_at        timestamptz NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, content_hash, pipeline_version)
);

CREATE INDEX IF NOT EXISTS idx_ingestion_dedup_workspace
    ON ingestion_dedup (workspace_id, created_at DESC);

CREATE TABLE IF NOT EXISTS compensation_quarantine (
    entry_id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id       uuid NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    workspace_id      uuid REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    status            text NOT NULL DEFAULT 'pending',
    next_attempt_at   timestamptz NOT NULL DEFAULT now(),
    attempt_count     integer NOT NULL DEFAULT 0,
    payload           jsonb NOT NULL DEFAULT '{}',
    last_error        jsonb,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    CHECK (status IN ('pending','processing','failed','dead','resolved'))
);

CREATE INDEX IF NOT EXISTS idx_compensation_quarantine_status_next
    ON compensation_quarantine (status, next_attempt_at)
    WHERE status IN ('pending', 'failed');

CREATE INDEX IF NOT EXISTS idx_compensation_quarantine_document
    ON compensation_quarantine (document_id);
