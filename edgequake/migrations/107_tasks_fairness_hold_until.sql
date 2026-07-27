-- ============================================================================
-- Migration 107: Durable fairness hold for claim-invisible park (SPEC-057 INV-06)
-- Version: 1.0.0 — 2026-07-26
--
-- PURPOSE:
--   When a tenant lane is at capacity, workers mark fairness_hold_until so
--   claim_next excludes the row (FP-1 / FP-5). Hold TTL prevents stranded
--   Pending if the park waiter process dies. Refresh edgequake.tasks view.
--
-- IDEMPOTENT: ADD COLUMN IF NOT EXISTS + CREATE INDEX IF NOT EXISTS +
--             CREATE OR REPLACE VIEW.
-- ============================================================================

SET search_path = public;

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS fairness_hold_until TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_tasks_fairness_hold_until
    ON tasks (fairness_hold_until)
    WHERE fairness_hold_until IS NOT NULL;

COMMENT ON COLUMN tasks.fairness_hold_until IS
    'SPEC-057 INV-06: claim_next skips rows while NOW() < fairness_hold_until; NULL = runnable';

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
    fairness_hold_until
  FROM public.tasks;

COMMENT ON VIEW edgequake.tasks IS
    'Alias of public.tasks including fairness_hold_until (SPEC-057 INV-06; refresh after 107)';
