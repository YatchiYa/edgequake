-- ============================================================================
-- Migration 112: SPEC-120 P0 — durable cancel intent + document fence epoch
-- Version: 1.0.0 — 2026-07-27
--
-- PURPOSE:
--   A2: cancel_requested_at is the intent SSOT (registry is a cache).
--   A3: documents.fence_epoch rejects post-terminal / post-delete writers.
--   G3: superseded_by keeps cancelled rows as guards (mark-and-supersede).
--
-- IDEMPOTENT: ADD COLUMN IF NOT EXISTS + CREATE INDEX IF NOT EXISTS +
--             CREATE OR REPLACE VIEW.
-- ============================================================================

SET search_path = public;

-- Durable cancel intent on the task row (SPEC-120 P0 / G1).
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS cancel_requested_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS superseded_by TEXT;

CREATE INDEX IF NOT EXISTS idx_tasks_cancel_requested_processing
    ON tasks (cancel_requested_at)
    WHERE cancel_requested_at IS NOT NULL AND status = 'processing';

COMMENT ON COLUMN tasks.cancel_requested_at IS
    'SPEC-120 P0: durable cancel intent; NULL = not cancelling; set before registry';
COMMENT ON COLUMN tasks.superseded_by IS
    'SPEC-120 P0: track_id of replacement task (reprocess/delete); NULL = none';

-- Monotonic fence for side-effect writes (SPEC-120 P0 / G2 / A3).
ALTER TABLE documents ADD COLUMN IF NOT EXISTS fence_epoch BIGINT NOT NULL DEFAULT 0;

COMMENT ON COLUMN documents.fence_epoch IS
    'SPEC-120 P0: bumped on delete/wipe/reprocess-supersede; writers must present matching epoch';

-- Keep edgequake.tasks column list current (view columns baked at CREATE time).
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
    superseded_by
  FROM public.tasks;

COMMENT ON VIEW edgequake.tasks IS
    'Alias of public.tasks including cancel_requested_at / superseded_by (SPEC-120 P0; refresh after 111)';
