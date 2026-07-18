-- SPEC-062 Wave 1: denormalized eq_node_id / eq_source_id / eq_target_id on AGE label tables.
--
-- Runtime SSOT: PostgresAGEGraphStorage::ensure_eq_id_columns (per-graph DDL + triggers).
-- This migration is a marker so sqlx version history records the excellence wave.
SELECT 1;
