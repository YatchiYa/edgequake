-- ============================================================================
-- Migration 091: Writable content_tsv + KV backfill (SPEC-058 Wave 3)
-- Version: 1.0.0 — 2026-07-18
--
-- PURPOSE:
--   Generated content_tsv from metadata->>'content' is empty for SPEC-024
--   content_ref-only rows. Empty tsvector is not NULL, so FTS coalesce never
--   reaches KV. Convert to a writable column and backfill from shared KV.
--
-- IDEMPOTENT: skips tables already non-generated; safe re-run.
-- ============================================================================

SET search_path = public;

DO $$
DECLARE
    tbl record;
    is_generated text;
    kv_table text;
    converted int := 0;
BEGIN
    FOR tbl IN
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename LIKE 'eq\_%\_vectors' ESCAPE '\'
    LOOP
        BEGIN
            SELECT a.attgenerated INTO is_generated
            FROM pg_attribute a
            JOIN pg_class c ON c.oid = a.attrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'public'
              AND c.relname = tbl.tablename
              AND a.attname = 'content_tsv'
              AND a.attnum > 0
              AND NOT a.attisdropped;

            IF is_generated IS NULL THEN
                -- Column missing: add writable.
                EXECUTE format(
                    'ALTER TABLE %I ADD COLUMN content_tsv TSVECTOR',
                    tbl.tablename
                );
                converted := converted + 1;
            ELSIF is_generated <> '' THEN
                -- Drop generated column and re-add writable.
                EXECUTE format('ALTER TABLE %I DROP COLUMN content_tsv', tbl.tablename);
                EXECUTE format(
                    'ALTER TABLE %I ADD COLUMN content_tsv TSVECTOR',
                    tbl.tablename
                );
                converted := converted + 1;
            END IF;

            -- Prefer shared default KV: eq_eq_default_kv (double eq_ prefix).
            -- Fallback: strip trailing _vectors → sibling _kv if present.
            kv_table := 'eq_eq_default_kv';
            IF NOT EXISTS (
                SELECT 1 FROM pg_tables
                WHERE schemaname = 'public' AND tablename = kv_table
            ) THEN
                kv_table := regexp_replace(tbl.tablename, '_vectors$', '_kv');
            END IF;

            IF EXISTS (
                SELECT 1 FROM pg_tables
                WHERE schemaname = 'public' AND tablename = kv_table
            ) THEN
                EXECUTE format(
                    $u$
                    UPDATE %I v
                    SET content_tsv = to_tsvector(
                        'english',
                        coalesce(
                            v.metadata->>'content',
                            k.value->>'content',
                            ''
                        )
                    )
                    FROM %I k
                    WHERE k.key = coalesce(v.metadata->>'content_ref', v.id)
                      AND (v.content_tsv IS NULL OR v.content_tsv = ''::tsvector)
                    $u$,
                    tbl.tablename,
                    kv_table
                );
            END IF;

            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS eq_%s_content_tsv_idx ON %I USING GIN (content_tsv)',
                tbl.tablename,
                tbl.tablename
            );
        EXCEPTION WHEN OTHERS THEN
            RAISE NOTICE 'SPEC-058 M091: % failed: %', tbl.tablename, SQLERRM;
        END;
    END LOOP;

    RAISE NOTICE 'SPEC-058 M091: converted/ensured content_tsv on % table(s)', converted;
END $$;
