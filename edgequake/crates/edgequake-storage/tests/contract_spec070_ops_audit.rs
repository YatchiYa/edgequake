//! SPEC-070 — op-family audit: request-path anti-patterns locked by source contract.

#[test]
fn contract_kv_prefix_and_suffix_use_limit() {
    let kv = include_str!("../src/adapters/postgres/kv.rs");
    assert!(
        kv.contains("WHERE key LIKE $1 LIMIT $2"),
        "KV prefix scan must LIMIT (O(limit), not unbounded)"
    );
    assert!(
        kv.contains("WHERE reverse(key) LIKE $1 LIMIT $2"),
        "KV suffix scan must use reverse-key + LIMIT (SPEC-011)"
    );
    // SPEC-070: legacy unbounded APIs must delegate to limited / carry LIMIT.
    assert!(
        kv.contains("keys_with_prefix_limited(prefix, SAFETY_CAP)"),
        "keys_with_prefix must delegate to limited path"
    );
    assert!(
        kv.contains("keys_with_suffix_limited(suffix, SAFETY_CAP)"),
        "keys_with_suffix must delegate to limited path"
    );
    // No remaining unbounded LIKE without LIMIT in executable SQL strings.
    assert!(
        !kv.contains("SELECT key FROM {} WHERE key LIKE $1\""),
        "unbounded key LIKE without LIMIT must be removed"
    );
    assert!(
        !kv.contains("SELECT key FROM {} WHERE reverse(key) LIKE $1\""),
        "unbounded reverse(key) LIKE without LIMIT must be removed"
    );
}

#[test]
fn contract_task_claim_uses_skip_locked() {
    let tasks = include_str!("../../edgequake-tasks/src/postgres.rs");
    assert!(
        tasks.contains("FOR UPDATE SKIP LOCKED"),
        "task claim must use SKIP LOCKED (O(1) claim, no lock pile-up)"
    );
}

#[test]
fn contract_pdf_list_is_paginated() {
    let pdf = include_str!("../src/adapters/postgres/pdf_list_query.rs");
    assert!(
        pdf.contains("LIMIT") && pdf.contains("OFFSET"),
        "PDF list query must paginate with LIMIT/OFFSET"
    );
}

#[test]
fn contract_conversations_list_is_paginated() {
    let conv = include_str!("../src/adapters/postgres/conversation.rs");
    assert!(
        conv.contains("LIMIT ${}") || conv.contains("LIMIT $2"),
        "conversation/message lists must use LIMIT"
    );
}

#[test]
fn contract_graph_scan_forbids_unbounded_get_all_on_trait_hot_path() {
    let read = include_str!("../src/traits/graph_read_ops.rs");
    // get_all_* may exist for admin/debug but must be documented as non-hot-path.
    assert!(
        read.contains("SPEC-006")
            || read.contains("hot path")
            || read.contains("must not")
            || read.contains("forbidden"),
        "graph read ops must document get_all_* hot-path ban (SPEC-006)"
    );
}

#[test]
fn contract_mm_assets_scoped_by_document() {
    let mm = include_str!("../src/adapters/postgres/mm_asset_storage_impl.rs");
    assert!(
        mm.contains("FROM document_mm_assets"),
        "mm asset storage must query document_mm_assets"
    );
    // Every read/delete path includes document_id (and usually workspace_id) predicates.
    let from_count = mm.matches("FROM document_mm_assets").count();
    let where_doc = mm.matches("WHERE document_id").count()
        + mm.matches("WHERE workspace_id = $1 AND document_id")
            .count()
        + mm.matches("document_id = $").count();
    assert!(
        from_count > 0 && where_doc >= from_count,
        "each document_mm_assets FROM must be scoped by document_id (from={from_count} scoped≈{where_doc})"
    );
}
