//! SPEC-074 — DiskANN / pgvectorscale query-time GUCs (ops + harness; not boot default).
//!
//! Official pgvectorscale tip: tune `query_rescore` for accuracy alongside
//! `query_search_list_size`. EdgeQuake does **not** silently enable DiskANN;
//! callers (ops session or claim-gate harness) apply these `SET LOCAL` statements.

/// Recommended opt-in recipe for dedicated DiskANN @150k full-gate (SPEC-072/074).
pub const DISKANN_OPTIN_SEARCH_LIST: u32 = 400;
/// Tip: rescore ≈ list/2 (pgvectorscale suggests adjusting rescore for accuracy).
pub const DISKANN_OPTIN_RESCORE: u32 = 200;

/// Clamp helpers shared by harness and future product DiskANN session path.
pub fn diskann_rescore_for_list(search_list: u32) -> u32 {
    (search_list / 2).max(50)
}

/// Build transaction-local DiskANN tuning statements.
///
/// Does **not** include planner bias (`enable_seqscan`); callers add that when needed.
pub fn diskann_query_tuning_statements(search_list: u32, rescore: u32) -> Vec<String> {
    let list = search_list.clamp(1, 10_000);
    let rescore = rescore.clamp(0, 10_000);
    vec![
        format!("SET LOCAL diskann.query_search_list_size = {list}"),
        format!("SET LOCAL diskann.query_rescore = {rescore}"),
    ]
}

/// Opt-in recipe statements: list=400, rescore=200.
pub fn diskann_optin_recipe_statements() -> Vec<String> {
    diskann_query_tuning_statements(DISKANN_OPTIN_SEARCH_LIST, DISKANN_OPTIN_RESCORE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optin_recipe_sets_list_and_rescore() {
        let stmts = diskann_optin_recipe_statements();
        assert_eq!(stmts.len(), 2);
        assert!(stmts[0].contains("query_search_list_size = 400"));
        assert!(stmts[1].contains("query_rescore = 200"));
    }

    #[test]
    fn rescore_for_list_matches_spec072_rule() {
        assert_eq!(diskann_rescore_for_list(100), 50);
        assert_eq!(diskann_rescore_for_list(400), 200);
        assert_eq!(diskann_rescore_for_list(800), 400);
    }
}
