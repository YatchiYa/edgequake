-- SPEC-111: provenance column for iw2 fleet embedding backfill → drop-guard parity.
--
-- `legacy_vector_id` records the source `eq_*_vectors.id` so migration 131 can
-- prove coverage without exact `entities.name` equality (display-name drift).
-- Expandable SAFE SCHEMA — no --confirm-drop.

BEGIN;

ALTER TABLE public.entity_embeddings
    ADD COLUMN IF NOT EXISTS legacy_vector_id text;

ALTER TABLE public.relationship_embeddings
    ADD COLUMN IF NOT EXISTS legacy_vector_id text;

ALTER TABLE public.report_embeddings
    ADD COLUMN IF NOT EXISTS legacy_vector_id text;

CREATE UNIQUE INDEX IF NOT EXISTS idx_entity_embeddings_legacy_vector_id
    ON public.entity_embeddings (legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_relationship_embeddings_legacy_vector_id
    ON public.relationship_embeddings (legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_report_embeddings_legacy_vector_id
    ON public.report_embeddings (legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

COMMENT ON COLUMN public.entity_embeddings.legacy_vector_id IS
    'SPEC-111: source eq_*_vectors.id from iw2 backfill (drop-guard provenance).';
COMMENT ON COLUMN public.relationship_embeddings.legacy_vector_id IS
    'SPEC-111: source eq_*_vectors.id from iw2 backfill (drop-guard provenance).';
COMMENT ON COLUMN public.report_embeddings.legacy_vector_id IS
    'SPEC-111: source eq_*_vectors.id from iw2 backfill (drop-guard provenance).';

COMMIT;
