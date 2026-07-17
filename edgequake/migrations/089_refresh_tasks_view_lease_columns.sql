-- ============================================================================
-- Migration 089: Refresh edgequake.tasks view for lease columns (SPEC-057 P1)
-- Version: 1.0.0 — 2026-07-17
--
-- PURPOSE:
--   Migration 088 added lease_owner / lease_token / lease_expires_at to
--   public.tasks. PostgreSQL bakes view column lists at CREATE time, so
--   edgequake.tasks stayed stale. With search_path ("$user", public) and
--   DB role edgequake, workers hit the VIEW → "column lease_expires_at
--   does not exist". Same class of fix as migration 031.
--
-- IDEMPOTENT: CREATE OR REPLACE VIEW (append-only columns at end).
-- ============================================================================

SET search_path = public;

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
    lease_expires_at
  FROM public.tasks;

COMMENT ON VIEW edgequake.tasks IS
    'Alias of public.tasks including lease columns (SPEC-057 P1; refresh after 088)';
