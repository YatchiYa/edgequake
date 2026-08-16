//! Markdown-aware chunking with heading breadcrumbs (SPEC-026 Phase 2 P-03).
//!
//! SPEC-125: default path packs sibling sections to the token budget
//! (`markdown_pack`). `EDGEQUAKE_MARKDOWN_PACK=0` restores heading-hard splits.

use async_trait::async_trait;

use super::markdown_pack::{markdown_chunk, markdown_pack_enabled};
use super::types::{ChunkResult, ChunkerConfig, ChunkingStrategy};
use crate::error::Result;

/// Split markdown at heading boundaries; attach section metadata to each chunk.
pub struct MarkdownChunking;

#[async_trait]
impl ChunkingStrategy for MarkdownChunking {
    async fn chunk(&self, content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
        markdown_chunk(content, config, markdown_pack_enabled()).await
    }

    fn name(&self) -> &str {
        "markdown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn markdown_chunk_carries_section_metadata() {
        let md = "# Guide\n\nIntro text.\n\n## Setup\n\nSetup steps here.";
        let config = ChunkerConfig {
            chunk_size: 200,
            chunk_overlap: 10,
            ..Default::default()
        };
        let chunks = MarkdownChunking.chunk(md, &config).await.unwrap();
        assert!(!chunks.is_empty());
        let setup = chunks
            .iter()
            .find(|c| {
                c.section
                    .as_ref()
                    .is_some_and(|s| s.heading_path.contains(&"Setup".to_string()))
            })
            .expect("setup section chunk");
        assert!(setup
            .section
            .as_ref()
            .unwrap()
            .heading_path
            .contains(&"Guide".to_string()));
    }

    #[tokio::test]
    async fn packed_guide_is_single_chunk_at_200() {
        let md = "# Guide\n\nIntro text.\n\n## Setup\n\nSetup steps here.";
        let config = ChunkerConfig {
            chunk_size: 200,
            chunk_overlap: 10,
            min_chunk_size: 1,
            ..Default::default()
        };
        let chunks = MarkdownChunking.chunk(md, &config).await.unwrap();
        assert_eq!(chunks.len(), 1, "small guide must pack, got {:?}", chunks);
    }
}
