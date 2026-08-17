-- SPEC-091 P1: operator job control — add 'cancelled' to the job state CHECK
-- so POST /admin/migration-jobs/{id}/cancel can record a terminal cancel.
-- Pause/resume already exist ('paused'/'running'). Terminal states are never
-- left (transition table in migration_engine/lease.rs is the SSOT).

ALTER TABLE edgequake.edgequake_migration_job
    DROP CONSTRAINT IF EXISTS edgequake_migration_job_state_check;

ALTER TABLE edgequake.edgequake_migration_job
    ADD CONSTRAINT edgequake_migration_job_state_check
    CHECK (state IN ('pending','preflight','running','paused','verifying',
                     'completed','failed','rolled_back','cancelled'));
