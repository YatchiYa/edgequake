-- ============================================================================
-- Migration 088: Task lease columns for SKIP LOCKED claim (SPEC-057 P1)
-- Version: 1.0.0 — 2026-07-17
--
-- PURPOSE:
--   Postgres becomes delivery SSOT. Workers claim with FOR UPDATE SKIP LOCKED
--   and hold a lease (owner + token + expiry). Channel/NOTIFY is wake-only.
--
-- IDEMPOTENT: ADD COLUMN IF NOT EXISTS + CREATE INDEX IF NOT EXISTS.
-- ============================================================================

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_owner TEXT;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_token UUID;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_expires_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_tasks_claimable_pending
    ON tasks (created_at ASC)
    WHERE status = 'pending';

CREATE INDEX IF NOT EXISTS idx_tasks_stale_processing_lease
    ON tasks (lease_expires_at ASC)
    WHERE status = 'processing' AND lease_expires_at IS NOT NULL;

COMMENT ON COLUMN tasks.lease_owner IS
    'Worker id holding the processing lease (SPEC-057 P1)';
COMMENT ON COLUMN tasks.lease_token IS
    'CAS token for refresh_lease / release_claim (SPEC-057 P1)';
COMMENT ON COLUMN tasks.lease_expires_at IS
    'Lease expiry; claimable when status=processing and expired (SPEC-057 P1)';
