-- SPEC-091 IW1 (GAP-091-23, GAP-091-25 / LD-06):
--
-- 1. Converge HNSW `ef_construction` for NEW typed indexes to **128**
--    (runtime SSOT in `hnsw_ef_construction_from_env`). Migration 071 is
--    checksum-locked at 32 and is NOT rewritten — it only touches the legacy
--    `eq_*_vectors` fleet. `docker/init.sql` is updated in the same change
--    set to 128 so new installs never introduce a third value.
--
-- 2. Create the first ANN index on `chunk_embeddings` (model-scoped HNSW
--    over halfvec). Exact `ORDER BY embedding <=>` was the interim path
--    (GAP-091-23); at ≥10k rows the HNSW path is the production default.
--    Partial-per-workspace indexes remain measurement-gated (LD-10) and are
--    NOT created here — the model-scoped index covers the hot path
--    `WHERE model_id = $2 [AND workspace_id = $3] ORDER BY <=> LIMIT`.

BEGIN;

-- Model-scoped HNSW: every search already binds model_id. Workspace filter
-- is optional and selective enough that a second partial index is deferred.
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_hnsw
    ON public.chunk_embeddings
    USING hnsw (embedding halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128);

-- Ledger: record the converged build policy so the console / capability
-- surface can cite a single generation rather than tribal knowledge.
INSERT INTO public.edgequake_schema_generation (relation_name, generation, notes)
VALUES (
    'chunk_embeddings.hnsw',
    1,
    'SPEC-091 IW1 LD-06: ef_construction=128, m=16, halfvec_cosine_ops (GAP-091-23/25)'
)
ON CONFLICT (relation_name) DO UPDATE
SET generation = EXCLUDED.generation,
    notes = EXCLUDED.notes,
    updated_at = now();

COMMIT;
