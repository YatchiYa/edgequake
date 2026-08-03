-- ============================================================================
-- Migration 114: SPEC-120 P2 — jobs, task lineage, attempts, events, fairness
-- Version: 1.0.0 — 2026-07-27
-- ============================================================================

SET search_path = public;

CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    operation TEXT NOT NULL,
    subject_kind TEXT,
    subject_id TEXT,
    idempotency_key TEXT,
    state TEXT NOT NULL DEFAULT 'requested',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, idempotency_key)
);

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS job_id UUID;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS parent_task_id TEXT;

CREATE INDEX IF NOT EXISTS idx_tasks_parent
    ON tasks(parent_task_id) WHERE parent_task_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_job
    ON tasks(job_id) WHERE job_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS task_events (
    id BIGSERIAL PRIMARY KEY,
    task_id TEXT NOT NULL,
    job_id UUID,
    seq BIGINT NOT NULL,
    kind TEXT NOT NULL,
    payload JSONB,
    at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (task_id, seq)
);

CREATE INDEX IF NOT EXISTS idx_task_events_job
    ON task_events(job_id, at) WHERE job_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_track_id TEXT NOT NULL,
    attempt_no INT NOT NULL,
    worker_id TEXT,
    lease_token UUID,
    lease_expires_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    outcome TEXT,
    fence_epoch BIGINT,
    UNIQUE (task_track_id, attempt_no)
);

CREATE INDEX IF NOT EXISTS idx_attempts_task_started
    ON attempts(task_track_id, started_at DESC);

CREATE TABLE IF NOT EXISTS tenant_lane_quota (
    tenant_id UUID NOT NULL,
    fairness_class TEXT NOT NULL,
    weight DOUBLE PRECISION NOT NULL DEFAULT 1.0 CHECK (weight > 0),
    max_concurrent INT NOT NULL DEFAULT 2 CHECK (max_concurrent > 0),
    PRIMARY KEY (tenant_id, fairness_class)
);

CREATE TABLE IF NOT EXISTS tenant_vruntime (
    tenant_id UUID NOT NULL,
    fairness_class TEXT NOT NULL,
    vruntime DOUBLE PRECISION NOT NULL DEFAULT 0,
    PRIMARY KEY (tenant_id, fairness_class)
);

COMMENT ON COLUMN tasks.parent_task_id IS
    'SPEC-120 P2: direct task lineage; PDF Insert follow-on points to Convert track_id';
COMMENT ON COLUMN tasks.job_id IS
    'SPEC-120 P2: optional owning job; additive while legacy task APIs remain active';
COMMENT ON TABLE task_events IS
    'SPEC-120 P2 append-only task event stream; writers allocate seq per task';
COMMENT ON TABLE attempts IS
    'SPEC-120 P2 lease/attempt audit trail; dual-write is optional during rollout';

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
    available_at,
    job_id,
    parent_task_id
  FROM public.tasks;

COMMENT ON VIEW edgequake.tasks IS
    'Alias of public.tasks including SPEC-120 P2 job and parent task lineage';
