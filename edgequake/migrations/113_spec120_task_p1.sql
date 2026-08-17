-- ============================================================================
-- Migration 113: SPEC-120 P1 — task lifecycle, backoff, and document lookup
-- Version: 1.0.0 — 2026-07-27
--
-- NOTE: `tasks` is RANGE-partitioned (M104). Status/type CHECKs may exist only
-- on child partitions (e.g. tasks_history). Drop by name across all relations
-- before ADD on the parent, or ADD CONSTRAINT races with the child name.
-- ============================================================================

SET search_path = public;

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS document_id TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS available_at TIMESTAMPTZ;

-- Backfill from every task payload shape used by PDF, text, and lifecycle tasks.
UPDATE tasks
SET document_id = COALESCE(
    NULLIF(payload #>> '{task_data,existing_document_id}', ''),
    NULLIF(payload #>> '{task_data,document_id}', ''),
    NULLIF(payload #>> '{task_data,metadata,document_id}', ''),
    NULLIF(payload #>> '{existing_document_id}', ''),
    NULLIF(payload #>> '{document_id}', ''),
    NULLIF(payload #>> '{metadata,document_id}', '')
)
WHERE document_id IS NULL;

CREATE INDEX IF NOT EXISTS idx_tasks_document_active
    ON tasks (document_id)
    WHERE document_id IS NOT NULL AND status NOT IN ('cancelled', 'indexed', 'dead_letter');

CREATE INDEX IF NOT EXISTS idx_tasks_available_pending
    ON tasks (available_at, created_at)
    WHERE status = 'pending' AND available_at IS NOT NULL;

-- Drop legacy CHECKs from parent + every partition (name is not unique across
-- the partition tree when only the child retained the pre-partition constraint).
DO $$
DECLARE
  r RECORD;
BEGIN
  FOR r IN
    SELECT c.oid::regclass AS tbl, con.conname
    FROM pg_constraint con
    JOIN pg_class c ON c.oid = con.conrelid
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = 'public'
      AND con.conname IN (
        'tasks_valid_status',
        'valid_status',
        'valid_task_type',
        'tasks_valid_type'
      )
  LOOP
    EXECUTE format('ALTER TABLE %s DROP CONSTRAINT IF EXISTS %I', r.tbl, r.conname);
  END LOOP;
END $$;

ALTER TABLE tasks ADD CONSTRAINT tasks_valid_status CHECK (
    status IN (
        'pending', 'held', 'processing', 'cancelling',
        'indexed', 'failed', 'dead_letter', 'cancelled'
    )
);

ALTER TABLE tasks ADD CONSTRAINT valid_task_type CHECK (
    task_type IN (
        'upload', 'insert', 'reprocess', 'scan', 'reindex',
        'pdf_processing', 'knowledge_injection', 'deletion',
        'batch_deletion', 'workspace_wipe'
    )
);

COMMENT ON COLUMN tasks.document_id IS
    'SPEC-120 P1: denormalized document id for indexed lifecycle task lookup';
COMMENT ON COLUMN tasks.available_at IS
    'SPEC-120 P1: earliest claim time for durable retry backoff; NULL = immediately available';

CREATE OR REPLACE VIEW edgequake.tasks AS
  SELECT
    id,
    tenant_id,
    workspace_id,
    track_id,
    task_type,
    status,
    priority,
    payload,
    result,
    error_message,
    retry_count,
    max_retries,
    scheduled_at,
    started_at,
    completed_at,
    created_at,
    updated_at,
    consecutive_timeout_failures,
    circuit_breaker_tripped,
    error,
    lease_owner,
    lease_token,
    lease_expires_at,
    progress,
    pdf_id,
    fairness_parked_at,
    cancel_requested_at,
    superseded_by,
    document_id,
    available_at
  FROM public.tasks;

COMMENT ON VIEW edgequake.tasks IS
    'Alias of public.tasks including SPEC-120 P1 document_id and available_at';
