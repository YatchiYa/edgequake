//! SPEC-078 — Opt-in Filtered-DiskANN labels (A6 bake-off helpers).
//!
//! Default OFF. Pure SQL + workspace→smallint map for harness / future opt-in —
//! not boot default and not wired into product `query_filtered`.

use crate::filter_column_policy::env_flag_true;
use std::collections::HashMap;

/// Max distinct workspace labels (PostgreSQL `smallint` positive dense assign).
pub const MAX_WORKSPACE_LABELS: i16 = 32_767;

/// Runtime tip flag for Filtered-DiskANN labels study path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilteredDiskannLabelPolicy {
    pub enabled: bool,
}

impl FilteredDiskannLabelPolicy {
    /// Load from env — default OFF (no silent flip).
    pub fn from_env() -> Self {
        Self {
            enabled: env_flag_true("EDGEQUAKE_FILTERED_DISKANN_LABELS"),
        }
    }
}

/// Dense `workspace_id` string → `smallint` label assigner (fail closed at bound).
#[derive(Debug, Default, Clone)]
pub struct WorkspaceLabelMap {
    by_workspace: HashMap<String, i16>,
}

impl WorkspaceLabelMap {
    pub fn new() -> Self {
        Self {
            by_workspace: HashMap::new(),
        }
    }

    /// Assign or return existing label. Errors if capacity exhausted.
    pub fn label_for(&mut self, workspace_id: &str) -> Result<i16, String> {
        if let Some(&id) = self.by_workspace.get(workspace_id) {
            return Ok(id);
        }
        if self.by_workspace.len() >= MAX_WORKSPACE_LABELS as usize {
            return Err(format!(
                "WorkspaceLabelMap exhausted at {MAX_WORKSPACE_LABELS} distinct workspaces"
            ));
        }
        let id = (self.by_workspace.len() as i16) + 1;
        self.by_workspace.insert(workspace_id.to_string(), id);
        Ok(id)
    }

    pub fn get(&self, workspace_id: &str) -> Option<i16> {
        self.by_workspace.get(workspace_id).copied()
    }

    pub fn len(&self) -> usize {
        self.by_workspace.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_workspace.is_empty()
    }
}

/// DDL: DiskANN index that includes `labels` for Filtered-DiskANN.
///
/// Requires pgvectorscale. Does **not** drop primary Wave-2 / HNSW indexes.
pub fn build_diskann_labels_index_sql(table: &str, index_name: &str) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {index_name} ON {table} \
         USING diskann (embedding vector_cosine_ops, labels) \
         WITH (storage_layout = 'memory_optimized')"
    )
}

/// DDL: embedding-only DiskANN (post-filter honesty baseline).
pub fn build_diskann_embedding_only_index_sql(table: &str, index_name: &str) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {index_name} ON {table} \
         USING diskann (embedding vector_cosine_ops) \
         WITH (storage_layout = 'memory_optimized')"
    )
}

/// SELECT with index-native label filter (`labels && ARRAY[$label_param]::smallint[]`).
///
/// `limit_param` is the outer LIMIT bind index. Query embedding is `$1::vector`.
pub fn build_filtered_diskann_label_select_sql(
    table: &str,
    label_param: u32,
    limit_param: u32,
) -> String {
    format!(
        r#"
            SELECT id, 1 - (embedding <=> $1::vector) AS score
            FROM {table}
            WHERE labels && ARRAY[${label_param}]::smallint[]
            ORDER BY embedding <=> $1::vector
            LIMIT ${limit_param}
            "#
    )
}

/// SELECT with TEXT workspace post-filter (honesty baseline — not index-native).
pub fn build_postfilter_diskann_select_sql(
    table: &str,
    workspace_param: u32,
    limit_param: u32,
) -> String {
    format!(
        r#"
            SELECT id, 1 - (embedding <=> $1::vector) AS score
            FROM {table}
            WHERE workspace_id = ${workspace_param}
            ORDER BY embedding <=> $1::vector
            LIMIT ${limit_param}
            "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn default_off() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("EDGEQUAKE_FILTERED_DISKANN_LABELS");
        assert!(!FilteredDiskannLabelPolicy::from_env().enabled);
    }

    #[test]
    fn map_assigns_dense_and_fails_closed() {
        let mut m = WorkspaceLabelMap::new();
        assert_eq!(m.label_for("ws-a").unwrap(), 1);
        assert_eq!(m.label_for("ws-b").unwrap(), 2);
        assert_eq!(m.label_for("ws-a").unwrap(), 1);
        // Fill to capacity without 32k loop cost: pre-insert dummy keys.
        for i in 3..=MAX_WORKSPACE_LABELS {
            m.by_workspace.insert(format!("ws-{i}"), i);
        }
        assert_eq!(m.len(), MAX_WORKSPACE_LABELS as usize);
        assert!(m.label_for("ws-overflow").is_err());
    }

    #[test]
    fn index_sql_includes_labels() {
        let sql = build_diskann_labels_index_sql("eq_v", "eq_v_fdl_idx");
        assert!(sql.contains("USING diskann (embedding vector_cosine_ops, labels)"));
        assert!(sql.contains("storage_layout = 'memory_optimized'"));
    }

    #[test]
    fn query_sql_uses_overlap() {
        let sql = build_filtered_diskann_label_select_sql("eq_v", 2, 3);
        assert!(sql.contains("labels && ARRAY[$2]::smallint[]"));
        assert!(sql.contains("ORDER BY embedding <=> $1::vector"));
        assert!(sql.contains("LIMIT $3"));
    }
}
