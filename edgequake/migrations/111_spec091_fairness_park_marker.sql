-- SPEC-091 post-wave hardening (LAW-Q2/Q5, risk register R-18): durable
-- fairness-park marker on tasks.
--
-- WHY: QW3's fair-share park set is process-local memory, invisible to
-- claim_next. Every idle worker re-claimed the same parked rows on each poll
-- (claim → "already parked" → release), producing a hot spin of wasted DB
-- writes. A durable marker lets claim_next exclude parked rows in SQL
-- (SSOT: guard strings live in edgequake_tasks::state_machine).
--
-- Semantics:
--   - Set atomically with claim release when a worker parks (AtCapacity).
--   - Cleared before the park waiter's queue re-wake.
--   - Volatile scheduling state, NOT a lifecycle status: swept at boot and
--     by the stale-park reaper (covers replica death mid-park).

ALTER TABLE public.tasks
    ADD COLUMN IF NOT EXISTS fairness_parked_at timestamptz;

-- Claim-path support: pending rows that are not fairness-parked.
-- (Partial index mirrors the status='pending' claim filter family.)
CREATE INDEX IF NOT EXISTS idx_tasks_pending_not_parked
    ON public.tasks (created_at)
    WHERE status = 'pending' AND fairness_parked_at IS NULL;
