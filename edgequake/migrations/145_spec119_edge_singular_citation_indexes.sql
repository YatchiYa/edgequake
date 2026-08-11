-- SPEC-119: AGE singular edge citation indexes (marker).
-- Child-label DDL is applied by graph_lifecycle::ensure_indexes (single-flight).
-- This migration records the schema-generation intent in _sqlx_migrations.

DO $$
BEGIN
    -- No public-schema objects; AGE labels live in the graph namespace.
    -- ensure_indexes adds:
    --   idx_edge_source_chunk_id, idx_edge_source_document_id
    -- (btree on agtype_to_json(properties)->>'source_chunk_id'|document_id)
    RAISE NOTICE 'SPEC-119: singular edge citation indexes applied at runtime via ensure_indexes';
END $$;

-- Intent recorded by version row in _sqlx_migrations only (no public DDL).
