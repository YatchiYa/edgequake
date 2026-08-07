-- SPEC-111 follow-up: scope legacy_vector_id uniqueness by workspace.
--
-- Migration 143 created UNIQUE (legacy_vector_id) globally. Hot-path ids are
-- `entity:NAME` / `{src}->{tgt}:{rel}` — not globally unique across workspaces.
-- Acc / multi-tenant ingest then fails with:
--   duplicate key value violates unique constraint
--     "idx_entity_embeddings_legacy_vector_id"
-- Expandable SAFE SCHEMA — no --confirm-drop.

BEGIN;

DROP INDEX IF EXISTS public.idx_entity_embeddings_legacy_vector_id;
DROP INDEX IF EXISTS public.idx_relationship_embeddings_legacy_vector_id;
DROP INDEX IF EXISTS public.idx_report_embeddings_legacy_vector_id;

-- Deduplicate any residue from the global unique era: keep one row per
-- (workspace_id, legacy_vector_id).
DELETE FROM public.entity_embeddings ee
WHERE ee.legacy_vector_id IS NOT NULL
  AND ee.ctid IN (
    SELECT ctid FROM (
      SELECT ctid,
             row_number() OVER (
               PARTITION BY workspace_id, legacy_vector_id
               ORDER BY ctid DESC
             ) AS rn
      FROM public.entity_embeddings
      WHERE legacy_vector_id IS NOT NULL
    ) d
    WHERE rn > 1
  );

DELETE FROM public.relationship_embeddings re
WHERE re.legacy_vector_id IS NOT NULL
  AND re.ctid IN (
    SELECT ctid FROM (
      SELECT ctid,
             row_number() OVER (
               PARTITION BY workspace_id, legacy_vector_id
               ORDER BY ctid DESC
             ) AS rn
      FROM public.relationship_embeddings
      WHERE legacy_vector_id IS NOT NULL
    ) d
    WHERE rn > 1
  );

DELETE FROM public.report_embeddings re
WHERE re.legacy_vector_id IS NOT NULL
  AND re.ctid IN (
    SELECT ctid FROM (
      SELECT ctid,
             row_number() OVER (
               PARTITION BY workspace_id, legacy_vector_id
               ORDER BY ctid DESC
             ) AS rn
      FROM public.report_embeddings
      WHERE legacy_vector_id IS NOT NULL
    ) d
    WHERE rn > 1
  );

CREATE UNIQUE INDEX IF NOT EXISTS idx_entity_embeddings_legacy_vector_id
    ON public.entity_embeddings (workspace_id, legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_relationship_embeddings_legacy_vector_id
    ON public.relationship_embeddings (workspace_id, legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_report_embeddings_legacy_vector_id
    ON public.report_embeddings (workspace_id, legacy_vector_id)
    WHERE legacy_vector_id IS NOT NULL;

COMMIT;
