//! Denormalized filter column policy (SPEC-064/065) — no postgres feature required.

/// Default max workspace rows for SPEC-080 tiny-slice exact (skip Wave-2 planner bias).
pub const DEFAULT_ANN_EXACT_MAX_ROWS: u64 = 2_000;

/// True when env name is a truthy flag (`1` / `true` / `yes` / `on`).
pub fn env_flag_true(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Prefer denormalized `tenant_id` / `workspace_id` equality (no JSONB OR).
///
/// Required for workspace partial HNSW index implication.
pub fn prefer_denorm_filter_columns() -> bool {
    env_flag_true("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE")
        || env_flag_true("EDGEQUAKE_METADATA_FILTER_COLUMNS_ONLY")
}

/// SPEC-080 B3: workspace row count at or below this skips Wave-2 `enable_seqscan=off` bias
/// so Postgres/pgvector can prefer exact (btree/seq) on tiny slices.
pub fn ann_exact_max_rows() -> u64 {
    std::env::var("EDGEQUAKE_ANN_EXACT_MAX_ROWS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_ANN_EXACT_MAX_ROWS)
        .clamp(0, 10_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ann_exact_max_rows_default_and_env() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("EDGEQUAKE_ANN_EXACT_MAX_ROWS");
        assert_eq!(ann_exact_max_rows(), 2_000);
        std::env::set_var("EDGEQUAKE_ANN_EXACT_MAX_ROWS", "500");
        assert_eq!(ann_exact_max_rows(), 500);
        std::env::remove_var("EDGEQUAKE_ANN_EXACT_MAX_ROWS");
    }
}
