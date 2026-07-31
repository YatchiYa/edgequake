-- SPEC-091 Wave D: typed home for LLM cache keys + drain transient checkpoints.
--
-- Two parts, both reversible-by-recompute:
--   1. `public.llm_cache` — typed table for the cache families
--      (`{hash}-cache`, `{hash}-kwcache`, multimodal `{mode}-{type}:{hash}-cache`).
--      Cache rows are pure function-of-input: losing them costs one LLM
--      recompute, never data. `namespace` carries the KV namespace so
--      per-tenant isolation semantics are preserved.
--   2. Drain stale pipeline checkpoints: a crash-recoverable checkpoint older
--      than 24h means its owning task is long gone — recovery would redo work
--      the user already cancelled or superseded. Deleting them is the same
--      policy `cleanup_stale_checkpoints` applies at startup; the migration
--      applies it once, atomically, before the KV drop.
--
-- No KV data is copied here on purpose: backfilling caches is wasted work
-- (they recompute on demand) and the shell (122) / dedup (117) families that
-- DO hold durable data have their own backfills.

-- ── 1. typed cache table ────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS public.llm_cache (
    cache_key    TEXT        NOT NULL,
    namespace    TEXT        NOT NULL DEFAULT 'default',
    value        JSONB       NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ NULL,
    PRIMARY KEY (cache_key, namespace)
);

COMMENT ON TABLE public.llm_cache IS
    'SPEC-091: LLM extraction/keyword/multimodal cache entries (replaces cache KV families). Rows are recomputable — TTL-based expiry allowed.';

-- Reaper: periodic DELETE of expired rows.
CREATE INDEX IF NOT EXISTS idx_llm_cache_expiry
    ON public.llm_cache (expires_at)
    WHERE expires_at IS NOT NULL;

-- Keep updated_at honest on upsert (matches sibling typed tables' pattern).
CREATE OR REPLACE FUNCTION public.touch_llm_cache_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at := now();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_llm_cache_touch ON public.llm_cache;
CREATE TRIGGER trg_llm_cache_touch
    BEFORE UPDATE ON public.llm_cache
    FOR EACH ROW EXECUTE FUNCTION public.touch_llm_cache_updated_at();

-- ── 2. drain transient checkpoints ──────────────────────────────────────────
-- Same predicate as the startup sweep (`cleanup_stale_checkpoints`,
-- EDGEQUAKE_CHECKPOINT_STALE_SECS default 86400). Rows here are the typed
-- copies; the KV copies are dropped wholesale in the Wave-D drop migration.

DELETE FROM public.pipeline_checkpoints
WHERE kind = 'checkpoint'
  AND updated_at < now() - interval '24 hours';

-- Crash snapshots (kind = 'snapshot') for failed documents or documents that
-- no longer exist can never be resumed — crash-recovery requires a live
-- queued/processing task. Drain them before the KV drop.
DELETE FROM public.pipeline_checkpoints c
WHERE c.kind = 'snapshot'
  AND NOT EXISTS (
      SELECT 1 FROM public.documents d
      WHERE d.id = c.document_id
        AND d.status IN ('pending', 'processing', 'indexed')
  );
