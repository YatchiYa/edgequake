//! SPEC-047 P7b/P7d contract: merge concurrency + SOURCE_IDS KEEP (pure SSOT).

use edgequake_pipeline::{
    apply_local_merge_async_clamp, apply_source_ids_limit, merge_source_ids, parse_merge_max_async,
    should_skip_description_update_keep, SourceIdsLimitMethod, DEFAULT_MAX_SOURCE_IDS,
    DEFAULT_MERGE_MAX_ASYNC, LOCAL_MERGE_MAX_ASYNC,
};

#[test]
fn contract_defaults_match_lightrag() {
    assert_eq!(DEFAULT_MAX_SOURCE_IDS, 200);
    assert_eq!(DEFAULT_MERGE_MAX_ASYNC, 8);
    assert_eq!(SourceIdsLimitMethod::default(), SourceIdsLimitMethod::Keep);
}

#[test]
fn contract_keep_skips_saturated_new_chunks_only() {
    let existing: Vec<_> = (0..200).map(|i| format!("c{i}")).collect();
    assert!(should_skip_description_update_keep(
        &existing,
        &["c200".into()],
        200,
        SourceIdsLimitMethod::Keep,
    ));
    assert!(!should_skip_description_update_keep(
        &existing,
        &["c0".into()],
        200,
        SourceIdsLimitMethod::Keep,
    ));
}

#[test]
fn contract_apply_limit_keep_vs_fifo() {
    let ids: Vec<_> = (0..5).map(|i| format!("id{i}")).collect();
    assert_eq!(
        apply_source_ids_limit(&ids, 2, SourceIdsLimitMethod::Keep),
        vec!["id0", "id1"]
    );
    assert_eq!(
        apply_source_ids_limit(&ids, 2, SourceIdsLimitMethod::Fifo),
        vec!["id3", "id4"]
    );
}

#[test]
fn contract_merge_then_keep_preserves_oldest() {
    // Already at cap: KEEP head drops brand-new ids appended at the tail.
    let existing: Vec<_> = (0..200).map(|i| format!("e{i}")).collect();
    let incoming = vec!["e0".into(), "new1".into(), "new2".into()];
    let merged = merge_source_ids(&existing, &incoming);
    assert_eq!(merged.len(), 202);
    let capped = apply_source_ids_limit(&merged, 200, SourceIdsLimitMethod::Keep);
    assert_eq!(capped.len(), 200);
    assert_eq!(capped[0], "e0");
    assert!(!capped.iter().any(|s| s == "new1"));
    assert!(!capped.iter().any(|s| s == "new2"));
}

#[test]
fn contract_la4_keep_cap_does_not_wipe_minority_doc() {
    let mut ids = Vec::new();
    for i in 0..20 {
        ids.push(format!("doc-a-chunk-{i}"));
    }
    ids.push("doc-b-chunk-0".into());
    let capped = apply_source_ids_limit(&ids, 5, SourceIdsLimitMethod::Keep);
    assert!(capped.iter().any(|s| s.starts_with("doc-b-")));
    assert!(capped.iter().any(|s| s.starts_with("doc-a-")));
}

#[test]
fn contract_parse_merge_max_async_clamps() {
    assert_eq!(parse_merge_max_async("16"), Some(16));
    assert_eq!(parse_merge_max_async("999"), Some(64));
    assert_eq!(parse_merge_max_async("0"), None);
    assert_eq!(parse_merge_max_async(""), None);
}

#[test]
fn contract_local_merge_async_clamp() {
    assert_eq!(
        apply_local_merge_async_clamp(8, "ollama"),
        LOCAL_MERGE_MAX_ASYNC
    );
    assert_eq!(apply_local_merge_async_clamp(8, "openai"), 8);
    assert_eq!(apply_local_merge_async_clamp(1, "ollama"), 1);
}
