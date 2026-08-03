-- SPEC-091 RM3: AGE citation / edge property index contract (marker).
-- Child-label DDL is applied by graph_lifecycle::ensure_indexes (single-flight).
-- This migration records the schema-generation intent in _sqlx_migrations.

DO $$
BEGIN
    -- No public-schema objects; AGE labels live in the graph namespace.
    -- ensure_indexes adds:
    --   idx_node_source_chunk_ids_gin, idx_edge_source_chunk_ids_gin,
    --   idx_edge_props_gin, idx_edge_tenant_id, idx_edge_workspace_id
    RAISE NOTICE 'SPEC-091 RM3: AGE citation indexes applied at runtime via ensure_indexes';
END $$;

-- Intent recorded by version row in _sqlx_migrations only (no public DDL).
