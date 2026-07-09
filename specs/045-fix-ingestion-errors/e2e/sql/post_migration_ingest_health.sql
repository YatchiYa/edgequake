-- SPEC-045: Post-migration ingestion health gates
-- Usage: psql "$DATABASE_URL" -f specs/045-fix-ingestion-errors/e2e/sql/post_migration_ingest_health.sql

\echo '=== SPEC-045 Post-Migration Ingestion Health ==='

-- 1. sqlx migrations applied
\echo '--- sqlx migrations (latest 5) ---'
SELECT version, description, installed_on
FROM _sqlx_migrations
ORDER BY version DESC
LIMIT 5;

-- 2. pgvector version (M042 gate)
\echo '--- pgvector extension ---'
SELECT extname, extversion,
       CASE WHEN extversion >= '0.8.0' THEN 'OK' ELSE 'DEGRADED (M042)' END AS m042_status
FROM pg_extension
WHERE extname = 'vector';

-- 3. AGE extension (M043)
\echo '--- AGE extension ---'
SELECT extname, extversion
FROM pg_extension
WHERE extname = 'age';

-- 4. M038 source_ids GIN indexes (sample check on default graph)
\echo '--- source_ids GIN indexes (M038) ---'
SELECT indexname, tablename
FROM pg_indexes
WHERE indexname LIKE '%source_ids%'
ORDER BY indexname
LIMIT 10;

-- 5. wsdoc index backfill sample (M047)
\echo '--- wsdoc index keys (M047 sample) ---'
SELECT COUNT(*) AS wsdoc_key_count
FROM eq_eq_default_kv
WHERE key LIKE 'wsdoc:%';

\echo '--- metadata keys without wsdoc (gap indicator) ---'
SELECT COUNT(*) AS metadata_without_wsdoc_guess
FROM eq_eq_default_kv m
WHERE m.key LIKE '%-metadata'
  AND NOT EXISTS (
    SELECT 1 FROM eq_eq_default_kv w
    WHERE w.key LIKE 'wsdoc:%'
      AND w.value::text LIKE '%' || replace(m.key, '-metadata', '') || '%'
  )
LIMIT 1;

-- 6. Failed documents in KV metadata
\echo '--- failed document count (KV metadata) ---'
SELECT COUNT(*) AS failed_docs
FROM eq_eq_default_kv
WHERE key LIKE '%-metadata'
  AND value::text ILIKE '%"status"%failed%';

-- 7. Stuck processing documents
\echo '--- processing document count (KV metadata) ---'
SELECT COUNT(*) AS processing_docs
FROM eq_eq_default_kv
WHERE key LIKE '%-metadata'
  AND value::text ILIKE '%"status"%processing%';

-- 8. Vector table dimension sample
\echo '--- vector tables ---'
SELECT tablename
FROM pg_tables
WHERE schemaname = 'public'
  AND tablename LIKE 'eq\_%\_vectors' ESCAPE '\'
ORDER BY tablename
LIMIT 5;

-- 9. halfvec check (M080)
\echo '--- halfvec column check (if any vector table exists) ---'
SELECT table_name, column_name, udt_name
FROM information_schema.columns
WHERE table_schema = 'public'
  AND table_name LIKE 'eq\_%\_vectors' ESCAPE '\'
  AND column_name = 'embedding'
LIMIT 3;

\echo '=== End SPEC-045 Health Gates ==='
\echo 'Interpretation:'
\echo '  m042_status OK       → pgvector ready for ingest'
\echo '  source_ids indexes   → empty on large graph = run apply_038.sh'
\echo '  wsdoc_key_count = 0  → M047 may not have run; restart API'
\echo '  failed_docs > 0      → run reprocess per 005-quick-fix-runbook.md'
