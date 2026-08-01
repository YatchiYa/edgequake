-- ============================================================================
-- Migration 141: SPEC-098 document lifecycle statuses (deleting / delete_failed)
-- Version: 1.0.0 — 2026-08-01
--
-- PURPOSE:
--   Extend `documents_valid_status` so delete admit can dual-write
--   `public.documents.status` alongside KV metadata (LAW-098-9).
--
-- PORTABLE: PG16 / PG17 / PG18 — drop/re-add CHECK; NOT VALID + VALIDATE.
-- IDEMPOTENT: DROP IF EXISTS + ADD IF NOT PRESENT via DO block.
--
-- Operator re-run: `migrations/support/141/apply.sql`
-- ============================================================================

SET search_path = public;

DO $$
DECLARE
    major int;
BEGIN
    major := current_setting('server_version_num')::int / 10000;
    RAISE NOTICE 'Migration 141 (SPEC-098): document lifecycle status CHECK (postgres_major=%)', major;
    IF major < 16 OR major > 18 THEN
        RAISE NOTICE 'Migration 141: unexpected postgres_major=% (supported: 16, 17, 18) — continuing with portable SQL', major;
    END IF;
END $$;

ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_valid_status;

ALTER TABLE documents ADD CONSTRAINT documents_valid_status CHECK (
    status IN (
        'pending',
        'processing',
        'chunking',
        'extracting',
        'embedding',
        'indexing',
        'completed',
        'indexed',
        'failed',
        'partial_failure',
        'cancelled',
        'deleting',
        'delete_failed'
    )
) NOT VALID;

ALTER TABLE documents VALIDATE CONSTRAINT documents_valid_status;

DO $$ BEGIN
    RAISE NOTICE 'Migration 141: documents_valid_status now includes deleting, delete_failed';
END $$;
