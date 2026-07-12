//! SPEC-047 P3a — extract futures must stream (O(k) peak), not materialize O(C).

#[test]
fn resilient_and_parallel_extract_stream_chunks_lazily() {
    let src = include_str!("../src/pipeline/extraction.rs");
    assert!(
        !src.contains("let futures: Vec<_> = chunks"),
        "must not pre-allocate all async futures (SPEC-047 P3a)"
    );
    assert!(
        src.contains("stream::iter(owned)"),
        "must stream over owned chunk clones (Send-safe)"
    );
    assert!(
        !src.contains("stream::iter(chunks.iter()"),
        "must not borrow chunks slice across await (breaks TaskProcessor Send)"
    );
    assert!(
        src.matches("buffer_unordered").count() >= 2,
        "both extract paths should use buffer_unordered"
    );
}

#[test]
fn entity_lineage_batch_api_exists() {
    let merger = include_str!("../src/merger/mod.rs");
    assert!(
        merger.contains("record_entity_links_batch"),
        "LineageSink must expose entity batch API (SPEC-047 P4)"
    );
    assert!(
        merger.contains("struct EntityLineageLink"),
        "EntityLineageLink must exist"
    );
}
