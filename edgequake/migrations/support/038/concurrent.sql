-- SPEC-006 Migration 038 — CONCURRENT index build (production / large graphs)
--
-- IMPORTANT:
--   - Run OUTSIDE a transaction (psql default autocommit OK)
--   - Run during low-traffic window; each index builds without blocking writes
--   - Idempotent: IF NOT EXISTS on each index
--   - Run support/038/preflight.sql first (or apply_038.sh --dry-run)
--
-- PG16 / PG17 / PG18: child label tables ("Node", "EDGE") — NAMEDATALEN-safe names.
--
-- Usage:
--   edgequake/scripts/migrations/apply_038.sh --dry-run
--   edgequake/scripts/migrations/apply_038.sh --apply --concurrent --yes

\set ON_ERROR_STOP on
SET lock_timeout = '5s';
SET statement_timeout = '0';

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE NOTICE 'AGE not installed — skipping concurrent index build';
        RETURN;
    END IF;
    RAISE NOTICE 'Starting concurrent index build (SPEC-006 038)...';
END $$;

SELECT format(
    'CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_node_source_id_expr ON %I."Node" '
    '((ag_catalog.agtype_to_json(properties)->>''source_id''));',
    g.name
)
FROM ag_catalog.ag_graph g
WHERE to_regclass(format('%I."Node"', g.name)) IS NOT NULL
\gexec

SELECT format(
    'CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_node_source_ids_gin ON %I."Node" '
    'USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> ''source_ids'') jsonb_ops);',
    g.name
)
FROM ag_catalog.ag_graph g
WHERE to_regclass(format('%I."Node"', g.name)) IS NOT NULL
\gexec

SELECT format(
    'CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_edge_source_ids_gin ON %I."EDGE" '
    'USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> ''source_ids'') jsonb_ops);',
    g.name
)
FROM ag_catalog.ag_graph g
WHERE to_regclass(format('%I."EDGE"', g.name)) IS NOT NULL
\gexec

DO $$ BEGIN RAISE NOTICE 'Concurrent index build complete (038)'; END $$;
