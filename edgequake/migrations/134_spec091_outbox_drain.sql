-- SPEC-091 RM0: outbox drain claim columns + indexes (LAW-RM5: signal only).
-- Writers remain best-effort; drain marks processed_at / TTL deletes on maint path.

ALTER TABLE public.outbox_events
    ADD COLUMN IF NOT EXISTS available_at timestamptz NOT NULL DEFAULT now();

ALTER TABLE public.outbox_events
    ADD COLUMN IF NOT EXISTS attempt_count integer NOT NULL DEFAULT 0;

-- Prefer claim order by availability then creation (SKIP LOCKED drain).
DROP INDEX IF EXISTS idx_outbox_events_unprocessed;
CREATE INDEX IF NOT EXISTS idx_outbox_events_unprocessed
    ON public.outbox_events (available_at, created_at)
    WHERE processed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_outbox_events_processed_at
    ON public.outbox_events (processed_at)
    WHERE processed_at IS NOT NULL;

COMMENT ON TABLE public.outbox_events IS
    'SPEC-091 RM0: transactional outbox. Drain (EDGEQUAKE_OUTBOX_DRAIN) claims '
    'unprocessed rows with FOR UPDATE SKIP LOCKED; never mutates document status.';

COMMENT ON COLUMN public.outbox_events.available_at IS
    'SPEC-091 RM0: next claim eligibility (backoff after failed apply).';

COMMENT ON COLUMN public.outbox_events.attempt_count IS
    'SPEC-091 RM0: apply attempts; dead after configured max (default 6).';
