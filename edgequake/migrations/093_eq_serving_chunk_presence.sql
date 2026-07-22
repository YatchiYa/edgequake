-- ============================================================================
-- Migration 093: SPEC-081 C5 — admin/debug serving presence (dual-SSOT honesty)
--
-- PURPOSE:
--   Read-only surface that lists relational chunks for a workspace and whether
--   they carry an embedding_id link. Optional join against a vectors table
--   (namespace-specific) when provided.
--
--   This is NOT the RAG ANN query path and does NOT rewrite KV/AGE/vectors.
-- ============================================================================

SET search_path = public;

-- Relational spine presence (chunks + embedding_id link flag).
CREATE OR REPLACE FUNCTION eq_serving_chunk_presence(p_workspace_id uuid)
RETURNS TABLE (
    chunk_id uuid,
    document_id uuid,
    embedding_id text,
    has_embedding_link boolean
)
LANGUAGE sql
STABLE
AS $$
    SELECT
        c.id AS chunk_id,
        c.document_id,
        c.embedding_id,
        (c.embedding_id IS NOT NULL) AS has_embedding_link
    FROM public.chunks c
    WHERE c.workspace_id = p_workspace_id;
$$;

COMMENT ON FUNCTION eq_serving_chunk_presence(uuid) IS
  'SPEC-081 C5 admin/debug: relational chunks for workspace with embedding_id link. Not RAG ANN SSOT.';

-- Optional dual join: chunks ↔ vectors table (regclass) by embedding_id / document_id.
-- Vectors tables use TEXT workspace_id; pass p_workspace_id::text for denorm match.
CREATE OR REPLACE FUNCTION eq_serving_vector_presence(
    p_workspace_id uuid,
    p_vectors_table regclass
)
RETURNS TABLE (
    chunk_id uuid,
    document_id uuid,
    embedding_id text,
    vector_row_id text,
    has_vector_row boolean
)
LANGUAGE plpgsql
STABLE
AS $$
BEGIN
    RETURN QUERY EXECUTE format(
        $q$
        SELECT
            c.id AS chunk_id,
            c.document_id,
            c.embedding_id,
            v.id AS vector_row_id,
            (v.id IS NOT NULL) AS has_vector_row
        FROM public.chunks c
        LEFT JOIN %s v
          ON (
                (c.embedding_id IS NOT NULL AND v.id = c.embedding_id)
             OR (v.document_id = c.document_id::text AND v.workspace_id = $1::text)
          )
        WHERE c.workspace_id = $1
        $q$,
        p_vectors_table
    )
    USING p_workspace_id;
END;
$$;

COMMENT ON FUNCTION eq_serving_vector_presence(uuid, regclass) IS
  'SPEC-081 C5 admin/debug: join chunks to a vectors table. Not RAG ANN SSOT; not a silent store unify.';
