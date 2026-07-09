-- SPEC-006 Migration 038 — Size-aware index apply (SSOT for bootstrap + ops)
--
-- Used by:
--   migration_bootstrap.rs (startup, after sqlx marker 038)
--   edgequake/scripts/migrations/apply_038.sh --apply
--
-- Session variable (optional, set by bootstrap):
--   edgequake.migration_large_graph_threshold — default 500000 vertices
--
-- Large graphs are SKIPPED here; use concurrent.sql for zero-downtime builds.
--
-- PG16 / PG17 / PG18: indexes target AGE child label tables ("Node", "EDGE")
-- where graph data lives (SPEC-034). Names are ≤63 chars so sqlx text bind
-- audit matches pg_indexes (NAMEDATALEN-safe).

DO $$
DECLARE
    graph_name text;
    graph_schema text;
    node_tbl regclass;
    edge_tbl regclass;
    vertex_count bigint;
    threshold bigint;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE NOTICE 'AGE not installed — skipping source_ids index apply';
        RETURN;
    END IF;

    threshold := COALESCE(
        NULLIF(current_setting('edgequake.migration_large_graph_threshold', true), '')::bigint,
        500000
    );

    RAISE NOTICE 'Migration 038 apply — large graph threshold: % vertices', threshold;

    FOR graph_name IN SELECT name FROM ag_catalog.ag_graph LOOP
        graph_schema := graph_name;
        node_tbl := to_regclass(format('%I."Node"', graph_schema));
        edge_tbl := to_regclass(format('%I."EDGE"', graph_schema));

        IF node_tbl IS NULL AND edge_tbl IS NULL THEN
            RAISE NOTICE 'Skip graph % — Node/EDGE label tables not present', graph_name;
            CONTINUE;
        END IF;

        vertex_count := NULL;
        IF node_tbl IS NOT NULL THEN
            EXECUTE format('SELECT COUNT(*) FROM %I."Node"', graph_schema)
                INTO vertex_count;
        END IF;

        IF vertex_count IS NOT NULL AND vertex_count >= threshold THEN
            RAISE WARNING 'Defer graph % (%) — exceeds threshold %; use apply_038.sh --concurrent',
                graph_name, vertex_count, threshold;
            CONTINUE;
        END IF;

        RAISE NOTICE 'Applying source_ids indexes for graph: % (vertices=%)', graph_name, COALESCE(vertex_count, 0);

        IF node_tbl IS NOT NULL THEN
            BEGIN
                EXECUTE format(
                    'CREATE INDEX IF NOT EXISTS idx_node_source_id_expr ON %I."Node" '
                    '((ag_catalog.agtype_to_json(properties)->>''source_id''))',
                    graph_schema
                );
                RAISE NOTICE '  ✓ Node source_id btree';
            EXCEPTION WHEN OTHERS THEN
                RAISE WARNING '  ✗ Node source_id btree: %', SQLERRM;
            END;

            BEGIN
                EXECUTE format(
                    'CREATE INDEX IF NOT EXISTS idx_node_source_ids_gin ON %I."Node" '
                    'USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> ''source_ids'') jsonb_ops)',
                    graph_schema
                );
                RAISE NOTICE '  ✓ Node source_ids GIN';
            EXCEPTION WHEN OTHERS THEN
                RAISE WARNING '  ✗ Node source_ids GIN: %', SQLERRM;
            END;
        END IF;

        IF edge_tbl IS NOT NULL THEN
            BEGIN
                EXECUTE format(
                    'CREATE INDEX IF NOT EXISTS idx_edge_source_ids_gin ON %I."EDGE" '
                    'USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> ''source_ids'') jsonb_ops)',
                    graph_schema
                );
                RAISE NOTICE '  ✓ EDGE source_ids GIN';
            EXCEPTION WHEN OTHERS THEN
                RAISE WARNING '  ✗ EDGE source_ids GIN: %', SQLERRM;
            END;
        END IF;
    END LOOP;

    RAISE NOTICE 'Migration 038 size-aware apply complete';
END $$;
