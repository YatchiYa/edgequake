-- ============================================================================
-- Migration 139: SPEC-098 entity spine ensure (marker)
-- Version: 1.0.0 — 2026-08-01
--
-- PURPOSE:
--   Marker for the SPEC-098 reconcile that ensures relational `entities`
--   spine rows exist for AGE vertices (and bare-name identity) so typed
--   fleet mirror (`entity_embeddings` FK) can resolve after saturated KEEP
--   re-ingests.
--
-- ACTUAL WORK:
--   `migrations/support/139/apply.sql` — paginated, idempotent
--   `INSERT … ON CONFLICT DO UPDATE` (portable on PG16 / PG17 / PG18).
--   Invoked by migration_bootstrap after this marker is recorded.
--
-- WHY A MARKER?
--   Same rationale as migration 040: large AGE corpora must not hold a
--   single sqlx migration transaction for minutes.
--
-- IDEMPOTENT: NOTICE-only marker; support script uses ON CONFLICT.
-- ============================================================================

SET search_path = public;

DO $$
DECLARE
    major int;
BEGIN
    major := current_setting('server_version_num')::int / 10000;
    RAISE NOTICE 'Migration 139 (SPEC-098): entity spine ensure marker recorded (postgres_major=%)', major;
    IF major < 16 OR major > 18 THEN
        RAISE NOTICE 'Migration 139: unexpected postgres_major=% (supported: 16, 17, 18) — continuing with portable SQL', major;
    END IF;
END $$;
