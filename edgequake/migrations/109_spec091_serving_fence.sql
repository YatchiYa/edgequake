-- SPEC-091 Wave-4: serving lifecycle fence + transactional outbox

CREATE TABLE IF NOT EXISTS chunk_serving_state (
    chunk_id      uuid PRIMARY KEY REFERENCES chunks(id) ON DELETE CASCADE,
    state         text NOT NULL,
    attempt_count integer NOT NULL DEFAULT 0,
    last_error    jsonb,
    updated_at    timestamptz NOT NULL DEFAULT now(),
    CHECK (state IN ('declared','embedded','graphed','ready','quarantined','deleting'))
);

CREATE INDEX IF NOT EXISTS idx_chunk_serving_state_state
    ON chunk_serving_state (state, updated_at DESC);

CREATE TABLE IF NOT EXISTS outbox_events (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type  text NOT NULL,
    aggregate_id    uuid NOT NULL,
    event_type      text NOT NULL,
    payload         jsonb NOT NULL DEFAULT '{}',
    created_at      timestamptz NOT NULL DEFAULT now(),
    processed_at    timestamptz
);

CREATE INDEX IF NOT EXISTS idx_outbox_events_unprocessed
    ON outbox_events (created_at)
    WHERE processed_at IS NULL;

COMMENT ON TABLE chunk_serving_state IS
    'SPEC-091 W4: query-visible chunks require state=ready when EDGEQUAKE_SERVING_FENCE=on.';
