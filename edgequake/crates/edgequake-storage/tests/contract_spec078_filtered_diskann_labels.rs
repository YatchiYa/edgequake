//! SPEC-078 — Filtered-DiskANN labels SQL + map contract (no database).

#![cfg(feature = "postgres")]

use edgequake_storage::{
    build_diskann_embedding_only_index_sql, build_diskann_labels_index_sql,
    build_filtered_diskann_label_select_sql, build_postfilter_diskann_select_sql,
    FilteredDiskannLabelPolicy, WorkspaceLabelMap, MAX_WORKSPACE_LABELS,
};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn filtered_diskann_labels_default_off() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var("EDGEQUAKE_FILTERED_DISKANN_LABELS");
    let p = FilteredDiskannLabelPolicy::from_env();
    assert!(!p.enabled, "silent flip forbidden");

    std::env::set_var("EDGEQUAKE_FILTERED_DISKANN_LABELS", "1");
    assert!(FilteredDiskannLabelPolicy::from_env().enabled);
    std::env::remove_var("EDGEQUAKE_FILTERED_DISKANN_LABELS");
}

#[test]
fn labels_index_ddl_includes_labels_column() {
    let sql = build_diskann_labels_index_sql("public.eq_fdl", "eq_fdl_labels_idx");
    assert!(sql.contains("USING diskann (embedding vector_cosine_ops, labels)"));
    assert!(sql.contains("CREATE INDEX IF NOT EXISTS"));
    let only = build_diskann_embedding_only_index_sql("public.eq_fdl", "eq_fdl_emb_idx");
    assert!(only.contains("USING diskann (embedding vector_cosine_ops)"));
    assert!(!only.contains(", labels)"));
}

#[test]
fn label_query_uses_overlap_not_text_workspace() {
    let sql = build_filtered_diskann_label_select_sql("public.eq_fdl", 2, 3);
    assert!(sql.contains("labels && ARRAY[$2]::smallint[]"), "{sql}");
    assert!(sql.contains("ORDER BY embedding <=> $1::vector"), "{sql}");
    assert!(sql.contains("LIMIT $3"), "{sql}");
    assert!(
        !sql.contains("workspace_id"),
        "label path must not use TEXT workspace post-filter: {sql}"
    );
}

#[test]
fn postfilter_baseline_uses_workspace_text() {
    let sql = build_postfilter_diskann_select_sql("public.eq_fdl", 2, 3);
    assert!(sql.contains("workspace_id = $2"), "{sql}");
    assert!(!sql.contains("labels &&"), "{sql}");
}

#[test]
fn workspace_label_map_bounds() {
    let mut m = WorkspaceLabelMap::new();
    assert_eq!(m.label_for("a").unwrap(), 1);
    assert_eq!(m.label_for("b").unwrap(), 2);
    assert_eq!(m.label_for("a").unwrap(), 1);
    assert_eq!(MAX_WORKSPACE_LABELS, 32_767);
}
