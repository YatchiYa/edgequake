-- SPEC-091 Wave B5: backfill public.document_artifacts from legacy per-document
-- KV sidecar keys (`{uuid}-lineage`, `{uuid}-multimodal-manifest`,
-- `{uuid}-multimodal-chunks`). Idempotent: ON CONFLICT keeps the NEWER row
-- (dual-write may already have landed fresher payloads).
--
-- Deliberately NOT backfilled:
--   * pipeline checkpoints/snapshots — transient crash-resume blobs (24h/7d
--     TTL); a stale backfill would resume superseded runs. KV fallback covers
--     in-flight documents until their next save dual-writes.
--   * the multimodal LLM analysis cache — content-addressed (hash+model+prompt),
--     not document-keyed, so it cannot live under a documents FK without
--     breaking cross-document cache reuse.
--
-- Rows whose key prefix is not a UUID or whose documents parent is missing are
-- skipped — KV stays the fallback for them until the final drop wave.

DO $$
DECLARE
    kv_table RECORD;
    uuid_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
    kind_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}-(lineage|multimodal-manifest|multimodal-chunks)$';
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    EXECUTE format($f$
        INSERT INTO public.document_artifacts (document_id, kind, payload)
        SELECT left(kv.key, 36)::uuid,
               substring(kv.key FROM 38),
               kv.value
        FROM %I kv
        WHERE kv.key ~ $1
          AND left(kv.key, 36) ~ $2
          AND EXISTS (
              SELECT 1 FROM documents d
              WHERE d.id = left(kv.key, 36)::uuid)
        ON CONFLICT (document_id, kind) DO UPDATE SET
            payload = EXCLUDED.payload, updated_at = now()
            WHERE document_artifacts.updated_at <= now()
    $f$, kv_table.tablename) USING kind_re, uuid_re;
END LOOP;
END $$;
