-- SPEC-006 Migration 038 — Post-apply verification (read-only)
-- Confirms expected indexes exist per AGE graph (Node/EDGE child tables).
--
-- Usage:
--   edgequake/scripts/migrations/apply_038.sh --verify

DO $$
DECLARE
    graph_name text;
    graph_schema text;
    missing int := 0;
    expected text;
    found boolean;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE NOTICE 'AGE not installed — nothing to verify';
        RETURN;
    END IF;

    RAISE NOTICE '=== Migration 038 index verification ===';

    FOR graph_name IN SELECT name FROM ag_catalog.ag_graph LOOP
        graph_schema := graph_name;

        IF to_regclass(format('%I."Node"', graph_schema)) IS NULL
           AND to_regclass(format('%I."EDGE"', graph_schema)) IS NULL THEN
            RAISE NOTICE 'Skip graph % — no Node/EDGE label tables', graph_name;
            CONTINUE;
        END IF;

        RAISE NOTICE '--- Graph: % ---', graph_name;

        FOR expected IN
            SELECT unnest(ARRAY[
                'idx_node_source_id_expr',
                'idx_node_source_ids_gin',
                'idx_edge_source_ids_gin'
            ])
        LOOP
            SELECT EXISTS (
                SELECT 1 FROM pg_indexes
                WHERE schemaname = graph_schema AND indexname = expected::name
            ) INTO found;

            IF found THEN
                RAISE NOTICE '  ✓ %', expected;
            ELSE
                IF expected LIKE '%edge_%' AND to_regclass(format('%I."EDGE"', graph_schema)) IS NULL THEN
                    RAISE NOTICE '  ~ % (skipped — no EDGE table)', expected;
                ELSIF expected LIKE '%node_%' AND to_regclass(format('%I."Node"', graph_schema)) IS NULL THEN
                    RAISE NOTICE '  ~ % (skipped — no Node table)', expected;
                ELSE
                    RAISE WARNING '  ✗ MISSING: %', expected;
                    missing := missing + 1;
                END IF;
            END IF;
        END LOOP;
    END LOOP;

    IF missing > 0 THEN
        RAISE EXCEPTION 'Migration 038 verification failed: % missing index(es)', missing;
    END IF;

    RAISE NOTICE 'Verification passed — all required indexes present';
END $$;
