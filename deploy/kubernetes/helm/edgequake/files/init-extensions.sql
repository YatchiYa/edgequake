-- EdgeQuake PostgreSQL Extensions Initialization (from edgequake/docker/init-extensions.sql)
ALTER USER edgequake SET search_path TO public;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS btree_gin;
CREATE EXTENSION IF NOT EXISTS vector;
DO $$
BEGIN
    CREATE EXTENSION IF NOT EXISTS age CASCADE;
    LOAD 'age';
    SET search_path = ag_catalog, "$user", public;
EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Apache AGE extension not available: %', SQLERRM;
END $$;
