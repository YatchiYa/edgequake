-- SPEC-091 IW2: typed entity/relationship/report embeddings (fleet cutover).
--
-- Mirrors migration 108 `chunk_embeddings` shape: shared `embedding_models`
-- registry, halfvec(1536) interim column, model-scoped HNSW (LD-06: m=16,
-- ef_construction=128, halfvec_cosine_ops). Schema generation ledger rows
-- record the converged build policy (F-091-16).

BEGIN;

-- Entity vectors: legacy key `entity:{NORMALIZED_NAME}` → relational `entities.id`.
CREATE TABLE IF NOT EXISTS public.entity_embeddings (
    model_id      uuid NOT NULL REFERENCES public.embedding_models(id) ON DELETE CASCADE,
    entity_id     uuid NOT NULL REFERENCES public.entities(id) ON DELETE CASCADE,
    workspace_id  uuid NOT NULL REFERENCES public.workspaces(workspace_id) ON DELETE CASCADE,
    embedding     halfvec(1536) NOT NULL,
    dimensions    integer NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id, entity_id),
    CHECK (dimensions > 0 AND dimensions <= 4000)
);

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_workspace
    ON public.entity_embeddings (workspace_id, model_id);

CREATE INDEX IF NOT EXISTS idx_entity_embeddings_hnsw
    ON public.entity_embeddings
    USING hnsw (embedding halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128);

-- Relationship vectors: legacy key `{src}->{tgt}:{type}` → `relationships.id`.
CREATE TABLE IF NOT EXISTS public.relationship_embeddings (
    model_id          uuid NOT NULL REFERENCES public.embedding_models(id) ON DELETE CASCADE,
    relationship_id   uuid NOT NULL REFERENCES public.relationships(id) ON DELETE CASCADE,
    workspace_id      uuid NOT NULL REFERENCES public.workspaces(workspace_id) ON DELETE CASCADE,
    embedding         halfvec(1536) NOT NULL,
    dimensions        integer NOT NULL,
    created_at        timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id, relationship_id),
    CHECK (dimensions > 0 AND dimensions <= 4000)
);

CREATE INDEX IF NOT EXISTS idx_relationship_embeddings_workspace
    ON public.relationship_embeddings (workspace_id, model_id);

CREATE INDEX IF NOT EXISTS idx_relationship_embeddings_hnsw
    ON public.relationship_embeddings
    USING hnsw (embedding halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128);

-- Community report vectors: no relational `reports` table — TEXT legacy key
-- (`community_report:{id}`) is the typed PK component (mirrors eq_*_vectors id).
CREATE TABLE IF NOT EXISTS public.report_embeddings (
    model_id      uuid NOT NULL REFERENCES public.embedding_models(id) ON DELETE CASCADE,
    report_id     text NOT NULL,
    workspace_id  uuid NOT NULL REFERENCES public.workspaces(workspace_id) ON DELETE CASCADE,
    embedding     halfvec(1536) NOT NULL,
    dimensions    integer NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id, report_id),
    CHECK (dimensions > 0 AND dimensions <= 4000),
    CHECK (report_id <> '')
);

CREATE INDEX IF NOT EXISTS idx_report_embeddings_workspace
    ON public.report_embeddings (workspace_id, model_id);

CREATE INDEX IF NOT EXISTS idx_report_embeddings_hnsw
    ON public.report_embeddings
    USING hnsw (embedding halfvec_cosine_ops)
    WITH (m = 16, ef_construction = 128);

COMMENT ON TABLE public.entity_embeddings IS
    'SPEC-091 IW2: typed entity vector authority; halfvec(1536) interim.';
COMMENT ON TABLE public.relationship_embeddings IS
    'SPEC-091 IW2: typed relationship vector authority; halfvec(1536) interim.';
COMMENT ON TABLE public.report_embeddings IS
    'SPEC-091 IW2: typed community-report vector authority; report_id is legacy TEXT key.';

INSERT INTO public.edgequake_schema_generation (relation_name, generation, notes)
VALUES
    (
        'entity_embeddings.hnsw',
        1,
        'SPEC-091 IW2 LD-06: ef_construction=128, m=16, halfvec_cosine_ops'
    ),
    (
        'relationship_embeddings.hnsw',
        1,
        'SPEC-091 IW2 LD-06: ef_construction=128, m=16, halfvec_cosine_ops'
    ),
    (
        'report_embeddings.hnsw',
        1,
        'SPEC-091 IW2 LD-06: ef_construction=128, m=16, halfvec_cosine_ops'
    )
ON CONFLICT (relation_name) DO UPDATE
SET generation = EXCLUDED.generation,
    notes = EXCLUDED.notes,
    updated_at = now();

COMMIT;
