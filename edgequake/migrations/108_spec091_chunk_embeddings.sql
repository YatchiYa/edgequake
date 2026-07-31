-- SPEC-091 Wave-3: typed embeddings + schema generation ledger

CREATE TABLE IF NOT EXISTS embedding_models (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name        text NOT NULL,
    dimensions  integer NOT NULL,
    metric      text NOT NULL DEFAULT 'cosine' CHECK (metric = 'cosine'),
    created_at  timestamptz NOT NULL DEFAULT now(),
    UNIQUE (name, dimensions)
);

-- Wave-3 expand-and-contract: halfvec(1536) until model-scoped partial indexes land.
-- Unconstrained halfvec requires pgvector version confirmation; fixed dim is safe interim.
CREATE TABLE IF NOT EXISTS chunk_embeddings (
    model_id      uuid NOT NULL REFERENCES embedding_models(id) ON DELETE CASCADE,
    chunk_id      uuid NOT NULL REFERENCES chunks(id) ON DELETE CASCADE,
    workspace_id  uuid NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    embedding     halfvec(1536) NOT NULL,
    dimensions    integer NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (model_id, chunk_id),
    CHECK (dimensions > 0 AND dimensions <= 4000)
);

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_workspace
    ON chunk_embeddings (workspace_id, model_id);

CREATE TABLE IF NOT EXISTS edgequake_schema_generation (
    relation_name   text PRIMARY KEY,
    generation      integer NOT NULL,
    retired_at      timestamptz,
    notes           text,
    updated_at      timestamptz NOT NULL DEFAULT now()
);

COMMENT ON TABLE chunk_embeddings IS
    'SPEC-091 W3: typed vector authority; halfvec(1536) interim — expand-and-contract per model generation.';
