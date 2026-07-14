//! SPEC-047 MV-22 — atomic mm/table blocks survive recursive chunking (parallel-safe).

use edgequake_pipeline::chunker::{
    default_recursive_separators, is_mm_chunk_header, split_preserving_atomic_regions, AtomicKind,
    ChunkerConfig, PageAwareChunking, RecursiveCharacterChunking,
};
use edgequake_pipeline::ChunkingStrategy;

#[test]
fn mm_chunk_header_detector_matches_lightrag_labels() {
    assert!(is_mm_chunk_header("[Chart Name]rev"));
    assert!(is_mm_chunk_header("[Table Name]t1"));
    assert!(!is_mm_chunk_header("# Not a mm header"));
}

#[test]
fn atomic_region_covers_full_chart_block() {
    let block = "[Chart Name]rev\n[Image Type]Chart\n\nQ4: 42\n\nNotes.";
    let md = format!("Intro.\n\n{block}\n\nOutro.");
    let regions = split_preserving_atomic_regions(&md);
    let mm = regions
        .iter()
        .find(|r| r.atomic == Some(AtomicKind::MultimodalChunk))
        .expect("mm atomic region");
    assert!(mm.text.contains("42"));
    assert!(mm.text.contains("[Image Type]Chart"));
}

#[tokio::test]
async fn recursive_chunking_keeps_chart_block_whole() {
    let block =
        "[Chart Name]rev_q4\n[Image Type]Chart\n\n**Key values:**\n- Q4: 42\n\nRevenue rose.";
    let md = format!("<!-- edgequake-page:3 -->\nBefore.\n\n{block}\n\nAfter.");
    let config = ChunkerConfig {
        chunk_size: 5,
        chunk_overlap: 0,
        min_chunk_size: 1,
        separators: default_recursive_separators(),
        ..Default::default()
    };
    let chunks = RecursiveCharacterChunking
        .chunk(&md, &config)
        .await
        .unwrap();
    let chart_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.content.contains("[Chart Name]"))
        .collect();
    assert_eq!(chart_chunks.len(), 1);
    assert!(chart_chunks[0].content.contains("42"));
}

#[tokio::test]
async fn recursive_chunking_keeps_vlm_chart_block_whole() {
    let block = "# rev q4\n\n**Type:** Chart\n\n**Key values:**\n- Q4: 42\n\nRevenue rose.";
    let md = format!("<!-- edgequake-page:2 -->\nBefore.\n\n{block}\n\nAfter.");
    let config = ChunkerConfig {
        chunk_size: 5,
        chunk_overlap: 0,
        min_chunk_size: 1,
        separators: default_recursive_separators(),
        ..Default::default()
    };
    let chunks = RecursiveCharacterChunking
        .chunk(&md, &config)
        .await
        .unwrap();
    let chart_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.content.contains("**Type:** Chart"))
        .collect();
    assert_eq!(chart_chunks.len(), 1);
    assert!(chart_chunks[0].content.contains("42"));
}

#[tokio::test]
async fn page_aware_inner_recursive_preserves_chart_block() {
    let block = "[Figure Name]fig_a\n[Image Type]Illustration\n\nComponent A connects to B.";
    let md =
        format!("<!-- edgequake-page:1 -->\nPage one.\n\n<!-- edgequake-page:2 -->\n{block}\n");
    let inner = Box::new(RecursiveCharacterChunking);
    let chunker = PageAwareChunking::new(inner);
    let config = ChunkerConfig {
        chunk_size: 4,
        chunk_overlap: 0,
        min_chunk_size: 1,
        separators: default_recursive_separators(),
        ..Default::default()
    };
    let chunks = chunker.chunk(&md, &config).await.unwrap();
    let figure_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.content.contains("[Figure Name]"))
        .collect();
    assert_eq!(figure_chunks.len(), 1);
    assert_eq!(figure_chunks[0].page_start, Some(2));
    assert!(figure_chunks[0].content.contains("Component A"));
}

#[tokio::test]
async fn pipe_table_not_split_mid_row() {
    let table = "| Year | Revenue |\n| --- | --- |\n| 2023 | 42M |\n| 2024 | 50M |";
    let md = format!("Context.\n\n{table}\n\nAfter.");
    let config = ChunkerConfig {
        chunk_size: 4,
        chunk_overlap: 0,
        min_chunk_size: 1,
        separators: default_recursive_separators(),
        ..Default::default()
    };
    let chunks = RecursiveCharacterChunking
        .chunk(&md, &config)
        .await
        .unwrap();
    let table_chunks: Vec<_> = chunks
        .iter()
        .filter(|c| c.content.contains("| 2023 |"))
        .collect();
    assert_eq!(table_chunks.len(), 1);
    assert!(table_chunks[0].content.contains("| 2024 |"));
}
