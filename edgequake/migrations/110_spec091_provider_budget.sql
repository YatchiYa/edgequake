-- SPEC-091 QW1: cluster-global provider in-flight budget (LAW-Q3, LD-11)
-- 13-queue-admission-target-spec.md. Slot leases mirror the task-claim
-- discipline: SKIP LOCKED acquisition, TTL expiry, fencing token, attribution.

CREATE TABLE IF NOT EXISTS edgequake.provider_slot (
    provider_key     text        NOT NULL,             -- 'ollama', 'openai', ...
    slot_id          integer     NOT NULL,             -- 0..budget-1, seeded per budget change
    lease_owner      text,                             -- worker instance id
    lease_token      uuid,                             -- fencing token (CAS)
    lease_expires_at timestamptz,
    task_track_id    text,                             -- attribution (observability)
    workspace_id     uuid,
    acquired_at      timestamptz,
    PRIMARY KEY (provider_key, slot_id)
);

-- Stale-slot reclaim (same discipline as idx_tasks_stale_processing_lease).
CREATE INDEX IF NOT EXISTS idx_provider_slot_stale
    ON edgequake.provider_slot (provider_key, lease_expires_at)
    WHERE lease_owner IS NOT NULL;

-- LAW-Q1 SSOT: one budget row per provider. Seeded by the admission resolver
-- (source: 'measured' | 'env' | 'profile'). budget = 0 disables the ledger.
CREATE TABLE IF NOT EXISTS edgequake.provider_budget (
    provider_key   text PRIMARY KEY,
    budget         integer NOT NULL CHECK (budget BETWEEN 0 AND 64),
    source         text NOT NULL,
    updated_at     timestamptz NOT NULL DEFAULT now()
);

-- Seed/reconcile slot rows to match a budget (never deletes leased rows;
-- surplus free slots are removed, leased surplus expires naturally).
CREATE OR REPLACE FUNCTION edgequake.provider_budget_reconcile_slots(
    p_provider_key text,
    p_budget       integer
) RETURNS void
LANGUAGE sql
AS $$
    INSERT INTO edgequake.provider_slot (provider_key, slot_id)
    SELECT p_provider_key, g
    FROM generate_series(0, GREATEST(p_budget - 1, -1)) AS g
    ON CONFLICT (provider_key, slot_id) DO NOTHING;

    DELETE FROM edgequake.provider_slot s
    WHERE s.provider_key = p_provider_key
      AND s.slot_id >= p_budget
      AND s.lease_owner IS NULL;
$$;

-- Observability projection (LAW-D4): inflight is a projection of slot rows.
CREATE OR REPLACE VIEW edgequake.provider_inflight AS
SELECT
    s.provider_key,
    COUNT(*) FILTER (WHERE s.lease_owner IS NOT NULL
                     AND (s.lease_expires_at IS NULL OR s.lease_expires_at >= now()))::bigint
        AS inflight,
    b.budget
FROM edgequake.provider_slot s
LEFT JOIN edgequake.provider_budget b ON b.provider_key = s.provider_key
GROUP BY s.provider_key, b.budget;

COMMENT ON TABLE edgequake.provider_slot IS
    'SPEC-091 QW1 (LAW-Q3): cluster-wide provider concurrency slots. Acquire via UPDATE ... FOR UPDATE SKIP LOCKED; TTL + fencing token; reaper reclaims stale leases.';
COMMENT ON TABLE edgequake.provider_budget IS
    'SPEC-091 QW1 (LAW-Q1): one provider budget row per provider — the capacity SSOT the admission resolver derives from.';
