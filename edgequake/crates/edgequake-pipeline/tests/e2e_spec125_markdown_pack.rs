//! SPEC-125 e2e: Markdown strategy packs heading-dense fixture; Recursive Acc unchanged.

use edgequake_pipeline::{
    build_chunker_config_with_policy, count_tokens, is_atx_heading_only_text, resolve_chunker,
    ChunkStrategy, ChunkingPolicy,
};

fn heading_dense() -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/spec125/heading_dense.md",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("heading_dense.md fixture")
}

#[test]
fn e2e_spec125_markdown_heading_dense_chunk_count() {
    let text = heading_dense();
    let config = build_chunker_config_with_policy(
        text.len(),
        ChunkStrategy::Markdown,
        Some(&ChunkingPolicy::acc_fair()),
        None,
    );
    let chunker = resolve_chunker(ChunkStrategy::Markdown, config);
    let chunks = chunker
        .chunk(&text, "spec125-heading-dense")
        .expect("chunk");
    assert_eq!(
        chunks.len(),
        1,
        "heading-dense markdown must pack to 1 chunk, got {}",
        chunks.len()
    );
    assert!(!is_atx_heading_only_text(&chunks[0].content));
    assert_eq!(chunks[0].token_count, count_tokens(&chunks[0].content));
    let (_, output, dist) = edgequake_pipeline::ingest_chunking_observation(
        text.len(),
        chunks.iter().map(|c| (c.token_count, c.content.as_str())),
    );
    assert_eq!(dist.orphan_heading_chunks, 0);
    assert_eq!(dist.token_min, chunks[0].token_count);
    assert!(!output.contains("PACKPROBE"));
}

#[test]
fn e2e_spec125_pipeline_chunker_heading_dense() {
    use edgequake_pipeline::{Chunker, MarkdownChunking};
    use std::sync::Arc;

    let text = heading_dense();
    let config = build_chunker_config_with_policy(
        text.len(),
        ChunkStrategy::Markdown,
        Some(&ChunkingPolicy::acc_fair()),
        None,
    );
    let chunker = Chunker::with_strategy(config, Arc::new(MarkdownChunking));
    let chunks = chunker
        .chunk(&text, "spec125-pipeline-chunker")
        .expect("pipeline chunker");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].token_count, count_tokens(&chunks[0].content));
    let (_, output, dist) = edgequake_pipeline::ingest_chunking_observation(
        text.len(),
        chunks.iter().map(|c| (c.token_count, c.content.as_str())),
    );
    assert_eq!(dist.orphan_heading_chunks, 0);
    assert!(!output.contains("PACKPROBE"));
}

#[test]
fn e2e_spec125_recursive_acc_geometry_untouched() {
    let unit = "The quick brown fox jumps over the lazy dog. ";
    let text = unit.repeat(2000);
    let config = build_chunker_config_with_policy(
        text.len(),
        ChunkStrategy::Recursive,
        Some(&ChunkingPolicy::acc_fair()),
        None,
    );
    let chunker = resolve_chunker(ChunkStrategy::Recursive, config);
    let n = chunker.chunk(&text, "spec125-acc").expect("chunk").len();
    assert!(n >= 2, "recursive Acc-fair still splits long prose, n={n}");
}
