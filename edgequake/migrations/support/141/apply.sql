-- ============================================================================
-- Migration 141 Support: SPEC-098 document lifecycle status CHECK
-- File: migrations/support/141/apply.sql
-- Invoked by: migration_bootstrap / operators
-- IDEMPOTENT: safe to re-run
-- PORTABLE: PG16 / PG17 / PG18
-- ============================================================================

SET search_path = public;

DO $$
DECLARE
  major int;
  already boolean;
BEGIN
  major := current_setting('server_version_num')::int / 10000;
  RAISE NOTICE 'Migration 141 support: postgres_major=%', major;

  -- Detect whether CHECK already allows 'deleting'.
  SELECT EXISTS (
    SELECT 1
    FROM pg_constraint c
    JOIN pg_class t ON t.oid = c.conrelid
    JOIN pg_namespace n ON n.oid = t.relnamespace
    WHERE n.nspname = 'public'
      AND t.relname = 'documents'
      AND c.conname = 'documents_valid_status'
      AND pg_get_constraintdef(c.oid) ILIKE '%deleting%'
  ) INTO already;

  IF already THEN
    RAISE NOTICE 'Migration 141 support: documents_valid_status already includes deleting — skip';
    INSERT INTO server_config (key, value)
    VALUES (
      'spec098_document_lifecycle_status',
      jsonb_build_object('completed_at', NOW()::text, 'status', 'already_ok')
    )
    ON CONFLICT (key) DO UPDATE
      SET value = EXCLUDED.value;
    RETURN;
  END IF;

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

  INSERT INTO server_config (key, value)
  VALUES (
    'spec098_document_lifecycle_status',
    jsonb_build_object('completed_at', NOW()::text, 'status', 'applied')
  )
  ON CONFLICT (key) DO UPDATE
    SET value = EXCLUDED.value;

  RAISE NOTICE 'Migration 141 support: documents_valid_status updated with deleting/delete_failed';
END $$;
