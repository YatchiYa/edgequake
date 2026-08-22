-- Migration 149: Denormalized document_id for task lookups (issue #384)
--
-- WHY: In-flight documents were joined to tasks via JSONB payload or
-- documents.track_id (often a batch id, not the worker task id). There was
-- no tasks.document_id column, so the diagnostic LEFT JOIN in #384 could
-- not run. Promote document_id at enqueue (same pattern as pdf_id / mig 101).
--
-- SAFE: ADD COLUMN IF NOT EXISTS + CREATE INDEX IF NOT EXISTS.
-- Partitioned parent ALTER TABLE ADD COLUMN propagates to partitions.

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS document_id TEXT;

CREATE INDEX IF NOT EXISTS idx_tasks_workspace_document_id
    ON tasks (workspace_id, document_id)
    WHERE document_id IS NOT NULL;

COMMENT ON COLUMN tasks.document_id IS
    'Denormalized from payload task_data/metadata at enqueue (issue #384 INV-07)';

-- Backfill live rows so INV-07 is true on existing fleets without waiting
-- for the next enqueue.
UPDATE tasks SET document_id = NULLIF(TRIM(COALESCE(
    payload->'task_data'->>'existing_document_id',
    payload->'task_data'->>'document_id',
    payload->'task_data'->'metadata'->>'document_id',
    payload->'metadata'->>'document_id',
    payload->'metadata'->>'existing_document_id'
)), '')
WHERE status IN ('pending', 'processing')
  AND document_id IS NULL;
