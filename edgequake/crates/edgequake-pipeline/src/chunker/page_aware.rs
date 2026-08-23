//! Page-aware PDF chunking (SPEC-032 W-09 / SPEC-135 pack-to-budget).
//!
//! ## Design Contract
//!
//! Page is **attribution**. Prefer packing within a page. SPEC-135 P2 may emit a
//! chunk with `page_end ≥ page_start` when an undersize remainder would otherwise
//! become an orphan extract job. Deep-links open `page_start`.
//!
//! Inner strategy defaults to the SPEC-125 markdown packer (tiktoken). Set
//! `EDGEQUAKE_PDF_PACK=0` to restore Recursive word-count (pre-135). Set
//! `EDGEQUAKE_PDF_CROSS_PAGE_PACK=0` to keep hard page emit (`page_start == page_end`).
//!
//! ## Marker format
//!
//! ```text
//! <!-- edgequake-page:1 -->
//! ...page 1 content...
//!
//! <!-- edgequake-page:2 -->
//! ...page 2 content...
//! ```

use async_trait::async_trait;

use super::cross_page_pack::{merge_cross_page_remainders, pdf_cross_page_pack_enabled};
use super::env_flags::env_flag_default_on_var;
use super::markdown_chunking::MarkdownChunking;
use super::page_marker::{parse_page_marker, PAGE_MARKER_PREFIX};
use super::recursive::RecursiveCharacterChunking;
use super::types::{ChunkResult, ChunkerConfig, ChunkingStrategy};
use crate::error::Result;
use crate::token_estimator::count_tokens;

/// Fleet kill switch. Default **on** (unset). `0` restores Recursive inner.
pub const PDF_PACK_ENV: &str = "EDGEQUAKE_PDF_PACK";

pub fn pdf_pack_enabled() -> bool {
    env_flag_default_on_var(PDF_PACK_ENV)
}

fn pdf_inner_strategy() -> Box<dyn ChunkingStrategy> {
    if pdf_pack_enabled() {
        Box::new(MarkdownChunking)
    } else {
        Box::new(RecursiveCharacterChunking)
    }
}

/// Wraps an inner chunker; splits content at page markers, then optionally
/// merges undersize remainders across pages (P2).
///
/// @implements SPEC-032 W-09 / SPEC-135
pub struct PageAwareChunking {
    /// Inner strategy used within each page segment.
    pub inner: Box<dyn ChunkingStrategy>,
}

impl Default for PageAwareChunking {
    fn default() -> Self {
        Self {
            inner: pdf_inner_strategy(),
        }
    }
}

impl PageAwareChunking {
    pub fn new(inner: Box<dyn ChunkingStrategy>) -> Self {
        Self { inner }
    }
}

/// A page-delimited segment extracted from the markdown.
#[derive(Debug)]
pub struct PageSegment {
    /// 1-indexed PDF page number.
    page: u32,
    /// Content of this page (page marker stripped).
    content: String,
    /// Byte offset of `content` within the original document (C-15).
    base_offset: usize,
}

/// Split markdown at `<!-- edgequake-page:N -->` markers.
///
/// Content before the first marker is attributed to page 1.
pub fn split_into_page_segments(content: &str) -> Vec<PageSegment> {
    if content.trim().is_empty() {
        return Vec::new();
    }

    if !content.contains(PAGE_MARKER_PREFIX) {
        return vec![PageSegment {
            page: 1,
            content: content.to_string(),
            base_offset: 0,
        }];
    }

    let mut segments: Vec<PageSegment> = Vec::new();
    let mut current_page: u32 = 1;
    let mut current_lines: Vec<&str> = Vec::new();
    let mut segment_base: Option<usize> = None;
    let mut byte_offset = 0usize;

    for line in content.lines() {
        let line_start = byte_offset;
        byte_offset += line.len();
        if byte_offset < content.len() && content.as_bytes()[byte_offset] == b'\n' {
            byte_offset += 1;
        }

        if let Some(page_num) = parse_page_marker(line) {
            let text = current_lines.join("\n");
            if !text.trim().is_empty() {
                segments.push(PageSegment {
                    page: current_page,
                    content: text,
                    base_offset: segment_base.unwrap_or(0),
                });
            }
            current_page = page_num;
            current_lines.clear();
            segment_base = None;
        } else {
            if segment_base.is_none() {
                segment_base = Some(line_start);
            }
            current_lines.push(line);
        }
    }

    let text = current_lines.join("\n");
    if !text.trim().is_empty() {
        segments.push(PageSegment {
            page: current_page,
            content: text,
            base_offset: segment_base.unwrap_or(0),
        });
    }

    segments
}

#[async_trait]
impl ChunkingStrategy for PageAwareChunking {
    async fn chunk(&self, content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let segments = split_into_page_segments(content);

        if segments.len() == 1 && segments[0].page == 1 && !content.contains(PAGE_MARKER_PREFIX) {
            return self.inner.chunk(content, config).await;
        }

        let mut results = Vec::new();
        let mut order = 0usize;

        for seg in segments {
            let page = seg.page;
            let base_offset = seg.base_offset;
            let sub_chunks = self.inner.chunk(&seg.content, config).await?;

            if sub_chunks.is_empty() && !seg.content.trim().is_empty() {
                results.push(ChunkResult {
                    content: seg.content.trim().to_string(),
                    tokens: count_tokens(&seg.content),
                    chunk_order_index: order,
                    page_start: Some(page),
                    page_end: Some(page),
                    start_offset: Some(base_offset),
                    end_offset: Some(base_offset.saturating_add(seg.content.len())),
                    ..Default::default()
                });
                order += 1;
            } else {
                for mut sub in sub_chunks {
                    if let Some(start) = sub.start_offset.as_mut() {
                        *start = start.saturating_add(base_offset);
                    }
                    if let Some(end) = sub.end_offset.as_mut() {
                        *end = end.saturating_add(base_offset);
                    }
                    sub.chunk_order_index = order;
                    sub.page_start = Some(page);
                    sub.page_end = Some(page);
                    results.push(sub);
                    order += 1;
                }
            }
        }

        if pdf_cross_page_pack_enabled() {
            results = merge_cross_page_remainders(results, config);
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "page_aware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::page_marker::make_page_marker;
    use serial_test::serial;

    fn make_pdf_markdown(pages: &[(u32, &str)]) -> String {
        pages
            .iter()
            .map(|(n, content)| format!("{}\n{}", make_page_marker(*n), content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn pin_hard_pages() {
        std::env::set_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK", "0");
    }

    fn unpin_hard_pages() {
        std::env::remove_var("EDGEQUAKE_PDF_CROSS_PAGE_PACK");
    }

    /// Hard-page invariant when P2 is killed.
    #[tokio::test]
    #[serial]
    async fn chunks_never_cross_page_boundary_when_p2_off() {
        pin_hard_pages();
        let md = make_pdf_markdown(&[
            (
                1,
                "Page one content with several sentences. More text here.",
            ),
            (
                2,
                "Page two has different content. Another sentence on page two.",
            ),
            (3, "Page three content. Final page."),
        ]);

        let config = ChunkerConfig {
            chunk_size: 20,
            chunk_overlap: 0,
            ..Default::default()
        };
        let chunker = PageAwareChunking::default();
        let chunks = chunker.chunk(&md, &config).await.unwrap();
        unpin_hard_pages();

        assert!(!chunks.is_empty(), "Must produce chunks");

        for chunk in &chunks {
            assert!(
                chunk.page_start.is_some(),
                "Every chunk must have page_start"
            );
            assert_eq!(
                chunk.page_start, chunk.page_end,
                "page_start must equal page_end when CROSS_PAGE_PACK=0"
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn e2e_page_aware_offsets_rebase() {
        pin_hard_pages();
        let page2 = "UNIQUE_PAGE_TWO_MARKER_TEXT for offset rebase.";
        let md = make_pdf_markdown(&[
            (1, "First page padding content that is long enough."),
            (2, page2),
        ]);
        let config = ChunkerConfig {
            chunk_size: 200,
            chunk_overlap: 0,
            ..Default::default()
        };
        let chunks = PageAwareChunking::default()
            .chunk(&md, &config)
            .await
            .unwrap();
        unpin_hard_pages();
        let page2_chunks: Vec<_> = chunks.iter().filter(|c| c.page_start == Some(2)).collect();
        assert!(!page2_chunks.is_empty());
        for c in page2_chunks {
            let (start, end) = (c.start_offset.expect("start"), c.end_offset.expect("end"));
            assert!(end > start && end <= md.len());
            let sliced = &md[start..end.min(md.len())];
            assert!(
                sliced.contains(c.content.trim()) || c.content.trim().contains(sliced.trim()),
                "slice(doc)=chunk.text invariant failed: slice={sliced:?} chunk={:?}",
                c.content
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn page_numbers_assigned_correctly() {
        pin_hard_pages();
        let md = make_pdf_markdown(&[(1, "Alpha beta gamma delta"), (2, "Epsilon zeta eta theta")]);

        let config = ChunkerConfig {
            chunk_size: 5,
            chunk_overlap: 0,
            ..Default::default()
        };
        let chunker = PageAwareChunking::default();
        let chunks = chunker.chunk(&md, &config).await.unwrap();
        unpin_hard_pages();

        let page1_chunks: Vec<_> = chunks.iter().filter(|c| c.page_start == Some(1)).collect();
        let page2_chunks: Vec<_> = chunks.iter().filter(|c| c.page_start == Some(2)).collect();

        assert!(!page1_chunks.is_empty(), "Must have page 1 chunks");
        assert!(!page2_chunks.is_empty(), "Must have page 2 chunks");
    }

    #[tokio::test]
    async fn plain_text_no_page_markers_fallback() {
        let config = ChunkerConfig {
            chunk_size: 50,
            chunk_overlap: 0,
            ..Default::default()
        };
        let chunker = PageAwareChunking::default();
        let chunks = chunker
            .chunk(
                "Hello world. This is plain text without any page markers.",
                &config,
            )
            .await
            .unwrap();

        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(
                chunk.page_start.is_none() || chunk.page_start == Some(1),
                "Plain text chunks should have no page attribution"
            );
        }
    }

    #[test]
    fn split_extracts_page_segments() {
        let md = make_pdf_markdown(&[(1, "Content of page one."), (2, "Content of page two.")]);
        let segs = split_into_page_segments(&md);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].page, 1);
        assert!(segs[0].content.contains("page one"));
        assert_eq!(segs[1].page, 2);
        assert!(segs[1].content.contains("page two"));
    }

    #[tokio::test]
    async fn empty_content_returns_empty() {
        let config = ChunkerConfig::default();
        let chunks = PageAwareChunking::default()
            .chunk("", &config)
            .await
            .unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn marker_roundtrip() {
        use crate::chunker::page_marker::parse_page_marker;
        for page in [1, 5, 100, 999] {
            let marker = make_page_marker(page);
            assert_eq!(parse_page_marker(&marker), Some(page));
        }
    }
}
