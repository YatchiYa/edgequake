-- SPEC-083 D-30: multigraph edge arbiter includes relation type.
--
-- Runtime SSOT also lives in PostgresAGEGraphStorage::ensure_eq_id_columns
-- (adds eq_rel_type + replaces idx_edge_eq_source_target). This migration
-- records the excellence wave in sqlx version history.
SELECT 1;
