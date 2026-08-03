-- SPEC-091 Wave-1: automatic migration engine ledger (07-migration-engine.md)
-- Uses gen_random_uuid(); upgrade to uuidv7() when PG18+ capability is confirmed.

CREATE TABLE IF NOT EXISTS edgequake.edgequake_migration_job (
    job_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    step_id           text NOT NULL,
    step_sha384       text NOT NULL,
    schema_generation integer NOT NULL,
    state             text NOT NULL,
    reversibility     text NOT NULL,
    cursor_position   jsonb,
    estimated_total   bigint,
    processed_count   bigint NOT NULL DEFAULT 0,
    failed_count      bigint NOT NULL DEFAULT 0,
    batch_size        integer NOT NULL,
    lease_owner       text,
    lease_expires_at  timestamptz,
    heartbeat_at      timestamptz,
    throttle_reason   text,
    started_at        timestamptz,
    completed_at      timestamptz,
    last_error        jsonb,
    UNIQUE (step_id, schema_generation),
    CHECK (state IN ('pending','preflight','running','paused','verifying','completed','failed','rolled_back')),
    CHECK (reversibility IN ('reversible','reversible_until_drop','irreversible'))
);

CREATE TABLE IF NOT EXISTS edgequake.edgequake_migration_batch (
    job_id        uuid NOT NULL REFERENCES edgequake.edgequake_migration_job(job_id) ON DELETE CASCADE,
    batch_seq     bigint NOT NULL,
    cursor_from   jsonb NOT NULL,
    cursor_to     jsonb NOT NULL,
    row_count     integer NOT NULL,
    duration_ms   integer NOT NULL,
    wal_bytes     bigint,
    committed_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (job_id, batch_seq)
);

CREATE INDEX IF NOT EXISTS idx_edgequake_migration_batch_job_committed
    ON edgequake.edgequake_migration_batch (job_id, committed_at DESC);

CREATE OR REPLACE VIEW edgequake.migration_progress AS
SELECT
    j.job_id,
    j.step_id,
    j.schema_generation,
    j.state,
    j.processed_count,
    j.estimated_total,
    CASE
        WHEN j.estimated_total IS NULL OR j.estimated_total = 0 THEN NULL
        ELSE ROUND(100.0 * j.processed_count / j.estimated_total, 2)
    END AS completion_pct,
    j.throttle_reason,
    j.started_at,
    j.completed_at,
    j.last_error,
    (
        SELECT AVG(recent.duration_ms)::integer
        FROM (
            SELECT b.duration_ms
            FROM edgequake.edgequake_migration_batch b
            WHERE b.job_id = j.job_id
            ORDER BY b.committed_at DESC
            LIMIT 20
        ) recent
    ) AS recent_batch_duration_ms_avg
FROM edgequake.edgequake_migration_job j;
