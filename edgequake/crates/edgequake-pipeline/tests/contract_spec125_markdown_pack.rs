//! SPEC-125 — markdown packer contracts (heading-dense fixture, fences, tables, kill switch).

use edgequake_pipeline::{
    count_tokens, ingest_chunking_observation, is_atx_heading_only_text, markdown_chunk,
    ChunkStrategy, ChunkerConfig, ChunkingStrategy, MarkdownChunking,
};

fn heading_dense() -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/spec125/heading_dense.md",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("heading_dense.md fixture must exist — do not inline a weaker substitute")
}

fn cfg(size: usize) -> ChunkerConfig {
    ChunkerConfig {
        chunk_size: size,
        chunk_overlap: 10,
        min_chunk_size: 1,
        ..Default::default()
    }
}

#[tokio::test]
async fn spec125_heading_dense_one_chunk() {
    let text = heading_dense();
    let chunks = MarkdownChunking.chunk(&text, &cfg(600)).await.unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(!is_atx_heading_only_text(&chunks[0].content));
    assert_eq!(chunks[0].tokens, count_tokens(&chunks[0].content));
    let (_, output, dist) = ingest_chunking_observation(
        text.len(),
        chunks.iter().map(|c| (c.tokens, c.content.as_str())),
    );
    assert_eq!(dist.orphan_heading_chunks, 0);
    assert!(!output.contains("PACKPROBE"));
}

#[tokio::test]
async fn spec125_kill_switch_via_markdown_chunk() {
    let text = heading_dense();
    let packed = markdown_chunk(&text, &cfg(1200), true).await.unwrap();
    let hard = markdown_chunk(&text, &cfg(1200), false).await.unwrap();
    assert_eq!(packed.len(), 1);
    assert!(hard.len() >= 4);
    assert!(is_atx_heading_only_text(&hard[0].content));
}

#[tokio::test]
async fn spec125_yaml_frontmatter_packs() {
    let md = "---\ntitle: x\n---\n\n## A\n\nHello.\n\n## B\n\nWorld.\n";
    let chunks = MarkdownChunking.chunk(md, &cfg(800)).await.unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("title: x"));
}

#[tokio::test]
async fn spec125_unicode_heading() {
    let md = "## Café 🎉\n\nBody with 中文.\n\n### Nested\n\nMore.\n";
    let chunks = MarkdownChunking.chunk(md, &cfg(800)).await.unwrap();
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].content.contains("Café"));
    assert!(chunks[0].content.contains("中文"));
}

#[tokio::test]
async fn spec125_nested_lists_pack() {
    let md = "## List\n\n- a\n  - nested\n- b\n\n## Next\n\nPara.\n";
    let chunks = MarkdownChunking.chunk(md, &cfg(800)).await.unwrap();
    assert_eq!(chunks.len(), 1);
}

#[test]
fn spec125_registry_still_selects_markdown() {
    assert_eq!(
        ChunkStrategy::resolve_for_upload(None, None, "notes.md"),
        ChunkStrategy::Markdown
    );
}

#[test]
fn spec125_pdf_upload_stays_on_pdf_strategy() {
    assert_eq!(
        ChunkStrategy::resolve_for_upload(None, None, "scan.pdf"),
        ChunkStrategy::Pdf
    );
}
