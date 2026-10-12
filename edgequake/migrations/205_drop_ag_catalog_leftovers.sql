-- Older migrations created EdgeQuake tables and functions in ag_catalog
-- because that schema was first on search_path while Apache AGE was
-- installed. Migration 171 drops the extension and the public copies.
-- The empty tables, indexes, and helper functions in ag_catalog stay.
-- Nothing outside the schema depends on them.
--
-- This step runs only after 171. 171 raises until the cutover ledger is
-- ready and then drops the extension, so a database that reaches 205 has
-- no AGE and a ready plain graph. Refuse if that is no longer true, or if
-- any leftover table has rows. A database that never had the schema is a
-- no-op.

DO $$
DECLARE
    rel text;
    has_row boolean;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'ag_catalog') THEN
        RETURN;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age') THEN
        RAISE EXCEPTION
            '205: ag_catalog still exists and Apache AGE is installed';
    END IF;

    IF to_regclass('public.eq_graph_cutover') IS NULL
       OR NOT EXISTS (
           SELECT 1
           FROM public.eq_graph_cutover
           WHERE id = 1
             AND status = 'ready'
             AND age_absent IS TRUE
       )
    THEN
        RAISE EXCEPTION
            '205: refusing to drop ag_catalog before the plain graph cutover is ready';
    END IF;

    FOR rel IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'ag_catalog'
          AND c.relkind = 'r'
    LOOP
        EXECUTE format('SELECT EXISTS (SELECT 1 FROM ag_catalog.%I)', rel)
            INTO has_row;
        IF has_row THEN
            RAISE EXCEPTION '205: ag_catalog.% is not empty', rel;
        END IF;
    END LOOP;

    DROP SCHEMA ag_catalog CASCADE;
END $$;
