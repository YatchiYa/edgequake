-- SPEC-091: unconstrained typed halfvec + dimension-scoped HNSW
--
-- Root cause: migrations 108/130 froze embedding as halfvec(1536) (interim).
-- Runtime models (mistral-embed / mxbai-embed-large → 1024, gemma/nomic → 768)
-- produce other lengths; pgvector then raises
--   "expected 1536 dimensions, not 1024"
-- during Knowledge graph / chunk typed persist.
--
-- Target (specs/091-simplify-data-layer/05-target-specification.md):
--   embedding halfvec (unconstrained)
--   CHECK (vector_dims(embedding) = dimensions)
--   partial expression HNSW per supported dimension (pgvector mixed-dim pattern).
--
-- Expandable SAFE SCHEMA — no --confirm-drop.

BEGIN;

-- ── Drop typmod-bound HNSW (created by 129 / 130) ──────────────────────────
DROP INDEX IF EXISTS public.idx_chunk_embeddings_hnsw;
DROP INDEX IF EXISTS public.idx_entity_embeddings_hnsw;
DROP INDEX IF EXISTS public.idx_relationship_embeddings_hnsw;
DROP INDEX IF EXISTS public.idx_report_embeddings_hnsw;

-- ── Unconstrain columns (keep existing 1536 rows intact) ───────────────────
ALTER TABLE public.chunk_embeddings
    ALTER COLUMN embedding TYPE halfvec USING embedding::halfvec;
ALTER TABLE public.entity_embeddings
    ALTER COLUMN embedding TYPE halfvec USING embedding::halfvec;
ALTER TABLE public.relationship_embeddings
    ALTER COLUMN embedding TYPE halfvec USING embedding::halfvec;
ALTER TABLE public.report_embeddings
    ALTER COLUMN embedding TYPE halfvec USING embedding::halfvec;

-- Row length must match the routing `dimensions` column (SSOT for ANN filters).
ALTER TABLE public.chunk_embeddings
    DROP CONSTRAINT IF EXISTS chunk_embeddings_dims_match;
ALTER TABLE public.chunk_embeddings
    ADD CONSTRAINT chunk_embeddings_dims_match
    CHECK (vector_dims(embedding) = dimensions);

ALTER TABLE public.entity_embeddings
    DROP CONSTRAINT IF EXISTS entity_embeddings_dims_match;
ALTER TABLE public.entity_embeddings
    ADD CONSTRAINT entity_embeddings_dims_match
    CHECK (vector_dims(embedding) = dimensions);

ALTER TABLE public.relationship_embeddings
    DROP CONSTRAINT IF EXISTS relationship_embeddings_dims_match;
ALTER TABLE public.relationship_embeddings
    ADD CONSTRAINT relationship_embeddings_dims_match
    CHECK (vector_dims(embedding) = dimensions);

ALTER TABLE public.report_embeddings
    DROP CONSTRAINT IF EXISTS report_embeddings_dims_match;
ALTER TABLE public.report_embeddings
    ADD CONSTRAINT report_embeddings_dims_match
    CHECK (vector_dims(embedding) = dimensions);

-- ── Dimension-scoped expression HNSW (768 / 1024 / 1536) ───────────────────
-- Covers known EdgeQuake embedding families (OpenAI 1536, Mistral/mxbai 1024,
-- gemma/nomic 768). Queries MUST filter `dimensions = N` (and preferably cast
-- the probe to halfvec(N)) so the planner can pick the matching partial index.

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_hnsw_d768
    ON public.chunk_embeddings
    USING hnsw ((embedding::halfvec(768)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 768;

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_hnsw_d1024
    ON public.chunk_embeddings
    USING hnsw ((embedding::halfvec(1024)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1024;

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_hnsw_d1536
    ON public.chunk_embeddings
    USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1536;

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_hnsw_d768
    ON public.entity_embeddings
    USING hnsw ((embedding::halfvec(768)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 768;

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_hnsw_d1024
    ON public.entity_embeddings
    USING hnsw ((embedding::halfvec(1024)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1024;

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_hnsw_d1536
    ON public.entity_embeddings
    USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1536;

CREATE INDEX IF NOT EXISTS idx_relationship_embeddings_hnsw_d768
    ON public.relationship_embeddings
    USING hnsw ((embedding::halfvec(768)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 768;

CREATE INDEX IF NOT EXISTS idx_relationship_embeddings_hnsw_d1024
    ON public.relationship_embeddings
    USING hnsw ((embedding::halfvec(1024)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1024;

CREATE INDEX IF NOT EXISTS idx_relationship_embeddings_hnsw_d1536
    ON public.relationship_embeddings
    USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1536;

CREATE INDEX IF NOT EXISTS idx_report_embeddings_hnsw_d768
    ON public.report_embeddings
    USING hnsw ((embedding::halfvec(768)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 768;

CREATE INDEX IF NOT EXISTS idx_report_embeddings_hnsw_d1024
    ON public.report_embeddings
    USING hnsw ((embedding::halfvec(1024)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1024;

CREATE INDEX IF NOT EXISTS idx_report_embeddings_hnsw_d1536
    ON public.report_embeddings
    USING hnsw ((embedding::halfvec(1536)) halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128)
    WHERE dimensions = 1536;

COMMENT ON COLUMN public.chunk_embeddings.embedding IS
    'SPEC-091: unconstrained halfvec; length must equal dimensions (CHECK). ANN via dim-scoped expression HNSW (768/1024/1536).';
COMMENT ON COLUMN public.entity_embeddings.embedding IS
    'SPEC-091: unconstrained halfvec; dim-scoped HNSW (768/1024/1536).';
COMMENT ON COLUMN public.relationship_embeddings.embedding IS
    'SPEC-091: unconstrained halfvec; dim-scoped HNSW (768/1024/1536).';
COMMENT ON COLUMN public.report_embeddings.embedding IS
    'SPEC-091: unconstrained halfvec; dim-scoped HNSW (768/1024/1536).';

INSERT INTO public.edgequake_schema_generation (relation_name, generation, notes)
VALUES
    (
        'chunk_embeddings.hnsw',
        2,
        'SPEC-091 mig 132: unconstrained halfvec + dim-scoped HNSW 768/1024/1536 (ef_construction=128)'
    ),
    (
        'entity_embeddings.hnsw',
        2,
        'SPEC-091 mig 132: unconstrained halfvec + dim-scoped HNSW 768/1024/1536'
    ),
    (
        'relationship_embeddings.hnsw',
        2,
        'SPEC-091 mig 132: unconstrained halfvec + dim-scoped HNSW 768/1024/1536'
    ),
    (
        'report_embeddings.hnsw',
        2,
        'SPEC-091 mig 132: unconstrained halfvec + dim-scoped HNSW 768/1024/1536'
    )
ON CONFLICT (relation_name) DO UPDATE
SET generation = EXCLUDED.generation,
    notes = EXCLUDED.notes,
    updated_at = now();

COMMIT;
