-- SPEC-091 Wave B7: purge legacy `auth:%` KV keys. Identity has been
-- PostgreSQL-native since SPEC-027; the boot-time import shim that consumed
-- these keys is removed in the same release. Upgrades from the KV-identity
-- era MUST pass through an intermediate release (importer present) before
-- this one — this delete is intentionally final for the KV store.
-- Idempotent: DELETE is naturally re-runnable.

DO $$
DECLARE
    kv_table RECORD;
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    EXECUTE format('DELETE FROM %I WHERE key LIKE ''auth:%%''', kv_table.tablename);
END LOOP;
END $$;
