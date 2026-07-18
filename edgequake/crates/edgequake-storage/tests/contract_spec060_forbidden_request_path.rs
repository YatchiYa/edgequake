//! SPEC-060 Wave 0 — FORBIDDEN APIs must not appear on request-path crates.

/// Strip line/block comments for crude call-site scanning.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while let Some(n) = chars.next() {
                if n == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            while let Some(n) = chars.next() {
                if n == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn assert_no_forbidden(path: &str, src: &str) {
    let clean = strip_comments(src);
    assert!(
        !clean.contains(".get_all_nodes("),
        "{path}: FORBIDDEN get_all_nodes on request path (SPEC-060)"
    );
    assert!(
        !clean.contains(".get_all_edges("),
        "{path}: FORBIDDEN get_all_edges on request path (SPEC-060)"
    );
}

#[test]
fn contract_query_modes_forbid_get_all() {
    for (path, src) in [
        (
            "local.rs",
            include_str!("../../edgequake-query/src/engine_impl/modes/local.rs"),
        ),
        (
            "global.rs",
            include_str!("../../edgequake-query/src/engine_impl/modes/global.rs"),
        ),
        (
            "naive.rs",
            include_str!("../../edgequake-query/src/engine_impl/modes/naive.rs"),
        ),
        (
            "mix.rs",
            include_str!("../../edgequake-query/src/engine_impl/modes/mix.rs"),
        ),
        (
            "hybrid.rs",
            include_str!("../../edgequake-query/src/engine_impl/modes/hybrid.rs"),
        ),
        (
            "graph_hops.rs",
            include_str!("../../edgequake-query/src/graph_hops.rs"),
        ),
        (
            "graph_expand.rs",
            include_str!("../../edgequake-query/src/graph_expand.rs"),
        ),
    ] {
        assert_no_forbidden(path, src);
    }
}

#[test]
fn contract_pipeline_merger_forbids_get_all() {
    let merger = include_str!("../../edgequake-pipeline/src/merger/mod.rs");
    assert_no_forbidden("merger/mod.rs", merger);
    let entity = include_str!("../../edgequake-pipeline/src/merger/entity.rs");
    assert_no_forbidden("merger/entity.rs", entity);
    let rel = include_str!("../../edgequake-pipeline/src/merger/relationship.rs");
    assert_no_forbidden("merger/relationship.rs", rel);
}

#[test]
fn contract_document_read_model_forbids_get_all_and_n1() {
    let src = include_str!("../../edgequake-api/src/document_read_model.rs");
    assert_no_forbidden("document_read_model.rs", src);
    assert!(
        !strip_comments(src).contains(".node_count_by_source_prefix(&"),
        "document_read_model must not N+1 prefix counts"
    );
}

#[test]
fn contract_spec060_stage_matrix_exists() {
    let matrix = include_str!("../../../../specs/060-data-layer-perf-proof/002-stage-matrix.md");
    assert!(matrix.contains("FORBIDDEN"));
    assert!(matrix.contains("e2e_spec060_fts_perf_explain"));
    assert!(matrix.contains("get_all_nodes"));
}

#[test]
fn contract_query_uses_query_filtered_not_unscoped_query() {
    // Defense: Local/Global must not call bare .query( without filter path.
    let local = include_str!("../../edgequake-query/src/engine_impl/modes/local.rs");
    let global = include_str!("../../edgequake-query/src/engine_impl/modes/global.rs");
    assert!(local.contains("query_filtered"));
    assert!(global.contains("query_filtered"));
}
