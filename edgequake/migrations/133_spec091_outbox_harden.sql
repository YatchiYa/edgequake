-- SPEC-091 IP2: harden transactional outbox (migration 109 base).
-- Portable SQL only (LAW-IP6) — no PG18-only triggers.

ALTER TABLE outbox_events
    ADD COLUMN IF NOT EXISTS workspace_id uuid;

-- Constrain event types used by the ingest persister (idempotent).
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'outbox_events_event_type_check'
          AND conrelid = 'public.outbox_events'::regclass
    ) THEN
        ALTER TABLE outbox_events
            ADD CONSTRAINT outbox_events_event_type_check
            CHECK (event_type IN (
                'chunk_declared',
                'chunk_ready',
                'merge_done',
                'compensate'
            ));
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_outbox_events_aggregate
    ON outbox_events (aggregate_type, aggregate_id, created_at);

CREATE INDEX IF NOT EXISTS idx_outbox_events_workspace_created
    ON outbox_events (workspace_id, created_at DESC)
    WHERE workspace_id IS NOT NULL;

COMMENT ON COLUMN outbox_events.workspace_id IS
    'SPEC-091 IP2: optional workspace scope for future isolation/drain filters.';
