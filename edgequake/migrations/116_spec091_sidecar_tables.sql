-- SPEC-091 Wave B4/B5: typed sidecar tables replacing per-document KV blobs.
--
-- `pipeline_checkpoints`: crash-resume checkpoints + durable extraction
-- snapshots (`{doc}-pipeline-checkpoint`, `{doc}-extraction-snapshot`).
-- `document_artifacts`: lineage + multimodal sidecars (`{doc}-lineage`,
-- `{doc}-multimodal-manifest`, `{doc}-multimodal-chunks`, MM cache).
-- One row per (document, kind) — upsert semantics mirror the KV overwrite.
-- Both FK into documents(id) with CASCADE so document deletion keeps parity
-- with the legacy KV family delete.

CREATE TABLE IF NOT EXISTS public.pipeline_checkpoints (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('checkpoint', 'snapshot')),
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, kind)
);

CREATE TABLE IF NOT EXISTS public.document_artifacts (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN (
        'lineage', 'multimodal-manifest', 'multimodal-chunks', 'multimodal-cache'
    )),
    payload     JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, kind)
);
