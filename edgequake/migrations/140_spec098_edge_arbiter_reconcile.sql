-- ============================================================================
-- Migration 140: SPEC-098 single EDGE arbiter + relationship spine (marker)
-- Version: 1.0.0 — 2026-08-01
--
-- PURPOSE:
--   Marker for SPEC-098 W6 reconcile that:
--     1. Enforces a single EDGE ON CONFLICT arbiter
--        `idx_edge_eq_source_target_rel` (eq_source_id, eq_target_id, eq_rel_type)
--     2. Drops legacy endpoint UNIQUEs that cause multigraph / cardinality failures
--     3. Refreshes `_eq_sync_edge_ids` (prefer column, then properties)
--     4. Ensures relational `relationships` spine from AGE edges (typed RelVectors)
--
-- ACTUAL WORK:
--   `migrations/support/140/apply.sql` — idempotent, portable PG16/17/18.
--   Invoked by migration_bootstrap after this marker is recorded.
--
-- IDEMPOTENT: NOTICE-only marker; support script uses IF EXISTS / ON CONFLICT.
-- ============================================================================

SET search_path = public;

DO $$
DECLARE
    major int;
BEGIN
    major := current_setting('server_version_num')::int / 10000;
    RAISE NOTICE 'Migration 140 (SPEC-098): edge arbiter reconcile marker recorded (postgres_major=%)', major;
    IF major < 16 OR major > 18 THEN
        RAISE NOTICE 'Migration 140: unexpected postgres_major=% (supported: 16, 17, 18) — continuing with portable SQL', major;
    END IF;
END $$;
