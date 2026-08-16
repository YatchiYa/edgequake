//! Recursive character chunking (LightRAG strategy `R` parity — SPEC-026 Phase 2).
//!
//! Mirrors LangChain `RecursiveCharacterTextSplitter` control flow and
//! `lightrag/chunker/recursive_character.py` span-preserving merge.

use async_trait::async_trait;

use super::atomic_blocks::split_preserving_atomic_regions;
use super::text_utils::{estimate_tokens, floor_char_boundary};
use super::types::{ChunkResult, ChunkerConfig, ChunkingStrategy};
use crate::error::Result;

type SpanPiece = (String, usize, usize);

/// LightRAG production default separator cascade (Jun 2026).
pub fn default_recursive_separators() -> Vec<String> {
    vec![
        "\n\n".to_string(),
        "\n".to_string(),
        "。".to_string(),
        "！".to_string(),
        "？".to_string(),
        "；".to_string(),
        "，".to_string(),
        " ".to_string(),
        String::new(),
    ]
}

/// Token length for recursive merge decisions — aligned with LightRAG's
/// tokenizer-backed `length_function` (word-aware Latin, char-aware CJK).
fn recursive_token_len(text: &str) -> usize {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return 0;
    }

    let cjk_chars = trimmed.chars().filter(|c| is_cjk_char(*c)).count();
    let mut base = if cjk_chars * 2 >= trimmed.chars().count() {
        ((trimmed.chars().count() as f32 / 1.5).ceil() as usize).max(1)
    } else {
        let words = trimmed.split_whitespace().filter(|w| !w.is_empty()).count();
        if words > 0 {
            words
        } else {
            estimate_tokens(trimmed)
        }
    };

    // keep_separator=true: leading paragraph/line breaks count toward merge budget
    // (tiktoken typically counts `\n\n` as ~2 tokens).
    if text.starts_with("\n\n") {
        base += 2;
    } else if text.starts_with('\n') {
        base += 1;
    }

    base.max(1)
}

fn is_cjk_char(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x3000..=0x303F)
}

/// Recursive character chunking with token-aware merge and overlap.
pub struct RecursiveCharacterChunking;

impl RecursiveCharacterChunking {
    fn effective_separators(config: &ChunkerConfig) -> Vec<String> {
        if config.separators.is_empty() {
            default_recursive_separators()
        } else {
            config.separators.clone()
        }
    }

    /// Split on literal `separator`, keeping the separator at the start of each
    /// piece after the first (LangChain `keep_separator=true`).
    fn split_literal_with_spans(text: &str, separator: &str, base_offset: usize) -> Vec<SpanPiece> {
        if separator.is_empty() {
            return text
                .char_indices()
                .filter(|(_, ch)| !ch.is_whitespace())
                .map(|(idx, ch)| {
                    let start = base_offset + idx;
                    (ch.to_string(), start, start + ch.len_utf8())
                })
                .collect();
        }

        if !text.contains(separator) {
            if text.is_empty() {
                return Vec::new();
            }
            return vec![(text.to_string(), base_offset, base_offset + text.len())];
        }

        let mut positions = Vec::new();
        let mut cursor = 0usize;
        while cursor < text.len() {
            if let Some(rel) = text[cursor..].find(separator) {
                let sep_start = cursor + rel;
                positions.push(sep_start);
                cursor = sep_start + separator.len();
            } else {
                break;
            }
        }

        let mut pieces = Vec::new();
        if positions[0] > 0 {
            let end = floor_char_boundary(text, positions[0]);
            let piece = text[..end].to_string();
            if !piece.trim().is_empty() {
                pieces.push((piece, base_offset, base_offset + end));
            }
        }

        for (index, &sep_start) in positions.iter().enumerate() {
            let end = positions.get(index + 1).copied().unwrap_or(text.len());
            if end > sep_start {
                let start = floor_char_boundary(text, sep_start);
                let end = floor_char_boundary(text, end);
                let piece = text[start..end].to_string();
                if !piece.trim().is_empty() {
                    pieces.push((piece, base_offset + start, base_offset + end));
                }
            }
        }

        pieces
    }

    fn join_span_pieces(
        pieces: &[SpanPiece],
        separator: &str,
        strip_whitespace: bool,
    ) -> Option<SpanPiece> {
        if pieces.is_empty() {
            return None;
        }

        let mut chars = String::new();
        let mut char_offsets: Vec<usize> = Vec::new();
        for (index, (fragment, start, end)) in pieces.iter().enumerate() {
            if index > 0 && !separator.is_empty() {
                let previous_end = pieces[index - 1].2;
                for (sep_index, sep_char) in separator.chars().enumerate() {
                    chars.push(sep_char);
                    char_offsets.push(previous_end + sep_index);
                }
            }
            chars.push_str(fragment);
            char_offsets.extend(*start..*end);
        }

        let (left, right) = if strip_whitespace {
            let mut l = 0;
            let mut r = chars.len();
            while l < r && chars.as_bytes()[l].is_ascii_whitespace() {
                l += 1;
            }
            while r > l && chars.as_bytes()[r - 1].is_ascii_whitespace() {
                r -= 1;
            }
            (l, r)
        } else {
            (0, chars.len())
        };

        if left >= right {
            return None;
        }

        let text: String = chars.chars().skip(left).take(right - left).collect();
        let start = char_offsets[left];
        let end = char_offsets[right - 1] + text.chars().last().map(|c| c.len_utf8()).unwrap_or(1);
        Some((text, start, end))
    }

    /// Mirror LightRAG `_merge_splits_with_spans`.
    fn merge_splits_with_spans(
        splits: &[SpanPiece],
        separator: &str,
        chunk_size: usize,
        chunk_overlap: usize,
        strip_whitespace: bool,
        len: fn(&str) -> usize,
    ) -> Vec<SpanPiece> {
        if splits.is_empty() {
            return Vec::new();
        }

        let separator_len = if separator.is_empty() {
            0
        } else {
            len(separator)
        };

        let mut docs: Vec<SpanPiece> = Vec::new();
        let mut current_doc: Vec<SpanPiece> = Vec::new();
        let mut total = 0usize;

        for split in splits {
            let split_len = len(&split.0);
            let with_sep = total
                + split_len
                + if current_doc.is_empty() {
                    0
                } else {
                    separator_len
                };

            if with_sep > chunk_size {
                if total > chunk_size {
                    tracing::warn!(
                        total,
                        chunk_size,
                        "recursive chunk exceeded token budget before flush"
                    );
                }
                if !current_doc.is_empty() {
                    if let Some(doc) =
                        Self::join_span_pieces(&current_doc, separator, strip_whitespace)
                    {
                        docs.push(doc);
                    }
                    while total > chunk_overlap
                        || (total
                            + split_len
                            + if current_doc.is_empty() {
                                0
                            } else {
                                separator_len
                            }
                            > chunk_size
                            && total > 0)
                    {
                        let removed_len = len(&current_doc[0].0)
                            + if current_doc.len() > 1 {
                                separator_len
                            } else {
                                0
                            };
                        total = total.saturating_sub(removed_len);
                        current_doc.remove(0);
                    }
                }
            }

            current_doc.push(split.clone());
            total += split_len
                + if current_doc.len() > 1 {
                    separator_len
                } else {
                    0
                };
        }

        if let Some(doc) = Self::join_span_pieces(&current_doc, separator, strip_whitespace) {
            docs.push(doc);
        }
        docs
    }

    /// Mirror LightRAG `_split_text_with_spans`.
    fn split_text_with_spans(
        text: &str,
        base_offset: usize,
        separators: &[String],
        chunk_size: usize,
        chunk_overlap: usize,
        len: fn(&str) -> usize,
    ) -> Vec<SpanPiece> {
        let mut separator = separators.last().cloned().unwrap_or_default();
        let mut rest: &[String] = &[];

        for (index, candidate) in separators.iter().enumerate() {
            if candidate.is_empty() {
                separator = String::new();
                break;
            }
            if text.contains(candidate.as_str()) {
                separator = candidate.clone();
                rest = &separators[index + 1..];
                break;
            }
        }

        let splits = if separator.is_empty() {
            Self::split_literal_with_spans(text, "", base_offset)
        } else {
            Self::split_literal_with_spans(text, &separator, base_offset)
        };

        let merge_separator = "";
        let mut final_chunks: Vec<SpanPiece> = Vec::new();
        let mut good_splits: Vec<SpanPiece> = Vec::new();

        for split in splits {
            if len(&split.0) < chunk_size {
                good_splits.push(split);
            } else if !good_splits.is_empty() {
                final_chunks.extend(Self::merge_splits_with_spans(
                    &good_splits,
                    merge_separator,
                    chunk_size,
                    chunk_overlap,
                    true,
                    len,
                ));
                good_splits.clear();

                if rest.is_empty() {
                    final_chunks.push(split);
                } else {
                    final_chunks.extend(Self::split_text_with_spans(
                        &split.0,
                        split.1,
                        rest,
                        chunk_size,
                        chunk_overlap,
                        len,
                    ));
                }
            } else if rest.is_empty() {
                final_chunks.push(split);
            } else {
                final_chunks.extend(Self::split_text_with_spans(
                    &split.0,
                    split.1,
                    rest,
                    chunk_size,
                    chunk_overlap,
                    len,
                ));
            }
        }

        if !good_splits.is_empty() {
            final_chunks.extend(Self::merge_splits_with_spans(
                &good_splits,
                merge_separator,
                chunk_size,
                chunk_overlap,
                true,
                len,
            ));
        }

        final_chunks
    }

    fn trim_span_piece(raw: SpanPiece) -> Option<SpanPiece> {
        let (raw_body, mut start_index, mut end_index) = raw;
        let bytes = raw_body.as_bytes();
        let mut left = 0usize;
        let mut right = raw_body.len();
        while left < right && bytes[left].is_ascii_whitespace() {
            left += 1;
        }
        while right > left && bytes[right - 1].is_ascii_whitespace() {
            right -= 1;
        }
        if left >= right {
            return None;
        }
        let body = raw_body[left..right].to_string();
        start_index += left;
        end_index -= raw_body.len() - right;
        Some((body, start_index, end_index))
    }

    fn chunk_inner(
        content: &str,
        config: &ChunkerConfig,
        len: fn(&str) -> usize,
    ) -> Result<Vec<ChunkResult>> {
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let separators = Self::effective_separators(config);
        let chunk_size = config.chunk_size.max(1);
        let chunk_overlap = config.chunk_overlap;

        let regions = split_preserving_atomic_regions(content);
        let mut results = Vec::new();
        let mut order = 0usize;

        for region in regions {
            let pieces = if region.atomic.is_some() {
                if len(&region.text) > chunk_size {
                    Self::split_text_with_spans(
                        &region.text,
                        region.start,
                        &separators,
                        chunk_size,
                        chunk_overlap,
                        len,
                    )
                } else {
                    vec![(
                        region.text.clone(),
                        region.start,
                        region.end.min(content.len()),
                    )]
                }
            } else {
                Self::split_text_with_spans(
                    &region.text,
                    region.start,
                    &separators,
                    chunk_size,
                    chunk_overlap,
                    len,
                )
            };

            for (text, start, end) in pieces {
                if let Some((body, start, end)) = Self::trim_span_piece((text, start, end)) {
                    results.push(ChunkResult {
                        content: body.clone(),
                        tokens: len(&body),
                        chunk_order_index: order,
                        section: None,
                        start_offset: Some(start),
                        end_offset: Some(end),
                        page_start: None,
                        page_end: None,
                    });
                    order += 1;
                }
            }
        }

        Ok(results
            .into_iter()
            .filter(|c| !c.content.is_empty())
            .collect())
    }

    /// SPEC-125: overflow split using tiktoken (or any length fn). Acc path stays
    /// on [`recursive_token_len`] via [`ChunkingStrategy::chunk`].
    pub(crate) fn chunk_with_len(
        &self,
        content: &str,
        config: &ChunkerConfig,
        len: fn(&str) -> usize,
    ) -> Result<Vec<ChunkResult>> {
        Self::chunk_inner(content, config, len)
    }
}

#[async_trait]
impl ChunkingStrategy for RecursiveCharacterChunking {
    async fn chunk(&self, content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
        Self::chunk_inner(content, config, recursive_token_len)
    }

    fn name(&self) -> &str {
        "recursive_character"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mm_chart_block_not_split_on_internal_paragraphs() {
        let block =
            "[Chart Name]rev_q4\n[Image Type]Chart\n\n**Key values:**\n- Q4: 42\n\nRevenue rose.";
        let md = format!("Preamble text.\n\n{block}\n\nEpilogue text.");
        // chunk_size above the chart's token length so C-16 does not force a split;
        // the invariant under test is "no split on internal paragraph breaks".
        let config = ChunkerConfig {
            chunk_size: 64,
            chunk_overlap: 0,
            min_chunk_size: 1,
            separators: default_recursive_separators(),
            ..Default::default()
        };
        let chunks = RecursiveCharacterChunking
            .chunk(&md, &config)
            .await
            .unwrap();
        let with_header: Vec<_> = chunks
            .iter()
            .filter(|c| c.content.contains("[Chart Name]"))
            .collect();
        assert_eq!(
            with_header.len(),
            1,
            "chart block must stay in one chunk when under chunk_size, got: {:?}",
            chunks.iter().map(|c| &c.content).collect::<Vec<_>>()
        );
        assert!(with_header[0].content.contains("42"));
    }

    #[tokio::test]
    async fn unit_atomic_respects_max_chunk_size() {
        // C-16: oversized atomic regions must split.
        let row = "| a | b | c | d | e |\n";
        let huge_table = format!("[Table Name]big\n[Image Type]Table\n\n{}", row.repeat(80));
        let md = format!("Intro.\n\n{huge_table}\n\nOutro.");
        let config = ChunkerConfig {
            chunk_size: 8,
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
            .filter(|c| c.content.contains("| a |") || c.content.contains("[Table Name]"))
            .collect();
        assert!(
            table_chunks.len() > 1,
            "huge atomic table must split across chunks, got {}",
            table_chunks.len()
        );
    }

    /// SPEC-083 C-16 matrix name: huge atomic table must not stay one oversize chunk.
    #[tokio::test]
    async fn e2e_huge_table_splits() {
        let row = "| col1 | col2 | col3 | value |\n";
        let huge_table = format!("[Table Name]mega\n[Image Type]Table\n\n{}", row.repeat(200));
        let md = format!("Preamble.\n\n{huge_table}\n\nEpilogue.");
        let config = ChunkerConfig {
            chunk_size: 16,
            chunk_overlap: 2,
            min_chunk_size: 1,
            ..ChunkerConfig::default()
        };
        assert!(
            config.separators.last().is_some_and(|s| s.is_empty()),
            "prod default must include LightRAG final empty separator"
        );
        let chunks = RecursiveCharacterChunking
            .chunk(&md, &config)
            .await
            .unwrap();
        let max_tokens = chunks.iter().map(|c| c.tokens).max().unwrap_or(0);
        assert!(
            chunks.len() > 1,
            "e2e_huge_table_splits: expected multiple chunks, got {}",
            chunks.len()
        );
        assert!(
            max_tokens <= config.chunk_size * 2,
            "e2e_huge_table_splits: chunk tokens {max_tokens} far over chunk_size {}",
            config.chunk_size
        );
    }

    #[test]
    fn contract_x_14_default_separators_are_lightrag_cascade() {
        let cfg = ChunkerConfig::default();
        assert_eq!(cfg.separators, default_recursive_separators());
        assert_eq!(
            cfg.separators.last().map(String::as_str),
            Some(""),
            "LightRAG cascade must end with empty string for char split"
        );
    }

    #[test]
    fn default_separators_match_lightrag() {
        assert_eq!(
            default_recursive_separators(),
            vec!["\n\n", "\n", "。", "！", "？", "；", "，", " ", ""]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn splits_on_paragraphs() {
        let text = "Para one.\n\nPara two.\n\nPara three.";
        let config = ChunkerConfig {
            chunk_size: 5,
            chunk_overlap: 0,
            min_chunk_size: 1,
            separators: default_recursive_separators(),
            ..Default::default()
        };
        let chunks = RecursiveCharacterChunking
            .chunk(text, &config)
            .await
            .unwrap();
        assert!(chunks.len() >= 2);
    }

    #[tokio::test]
    async fn plain_en_fixture_matches_lightrag_chunk_count() {
        let text = std::fs::read_to_string(format!(
            "{}/tests/fixtures/spec026/plain_en.txt",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let config = ChunkerConfig {
            chunk_size: 15,
            chunk_overlap: 0,
            min_chunk_size: 1,
            separators: default_recursive_separators(),
            ..Default::default()
        };
        let chunks = RecursiveCharacterChunking
            .chunk(&text, &config)
            .await
            .unwrap();
        let starts: Vec<_> = chunks.iter().filter_map(|c| c.start_offset).collect();
        assert_eq!(
            chunks.len(),
            3,
            "expected 3 paragraph-aligned chunks at chunk_size=15, got {} starts={starts:?}",
            chunks.len()
        );
    }

    #[test]
    fn recursive_token_len_counts_leading_paragraph_break() {
        let s = "\n\nParagraph two continues the document with more content.";
        assert_eq!(recursive_token_len(s), 10);
    }

    #[test]
    fn merge_plain_en_paragraphs_respects_token_budget() {
        let text = std::fs::read_to_string(format!(
            "{}/tests/fixtures/spec026/plain_en.txt",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let splits = RecursiveCharacterChunking::split_literal_with_spans(text.as_str(), "\n\n", 0);
        let merged = RecursiveCharacterChunking::merge_splits_with_spans(
            &splits,
            "",
            15,
            0,
            true,
            recursive_token_len,
        );
        assert_eq!(
            merged.len(),
            3,
            "expected 3 merged chunks, got {}: {:?}",
            merged.len(),
            merged.iter().map(|p| (p.1, &p.0)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn split_literal_plain_en_paragraphs() {
        let text = std::fs::read_to_string(format!(
            "{}/tests/fixtures/spec026/plain_en.txt",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let pieces = RecursiveCharacterChunking::split_literal_with_spans(text.as_str(), "\n\n", 0);
        assert_eq!(pieces.len(), 3, "expected 3 paragraph splits");
        assert_eq!(pieces[0].1, 0);
        assert_eq!(pieces[1].1, 42);
        assert!(pieces[1].0.starts_with("\n\n"));
        assert_eq!(pieces[2].1, 99);
    }

    #[test]
    fn keep_separator_includes_newlines_on_later_pieces() {
        let text = "Alpha.\n\nBeta.\n\nGamma.";
        let pieces = RecursiveCharacterChunking::split_literal_with_spans(text, "\n\n", 0);
        assert_eq!(pieces.len(), 3);
        assert!(!pieces[0].0.starts_with('\n'));
        assert!(pieces[1].0.starts_with("\n\n"));
    }

    #[tokio::test]
    async fn acc_chunk_stays_on_word_count_not_tiktoken() {
        let text = "alpha beta gamma delta epsilon zeta eta theta iota kappa. ".repeat(40);
        let config = ChunkerConfig {
            chunk_size: 20,
            chunk_overlap: 0,
            min_chunk_size: 1,
            separators: default_recursive_separators(),
            ..Default::default()
        };
        let acc = RecursiveCharacterChunking
            .chunk(&text, &config)
            .await
            .unwrap();
        let tik = RecursiveCharacterChunking
            .chunk_with_len(&text, &config, crate::token_estimator::count_tokens)
            .unwrap();
        assert!(!acc.is_empty() && !tik.is_empty());
        assert_eq!(
            acc[0].tokens,
            recursive_token_len(&acc[0].content),
            "Acc recursive must stamp word-count tokens"
        );
        assert_eq!(
            tik[0].tokens,
            crate::token_estimator::count_tokens(&tik[0].content),
            "chunk_with_len must stamp tiktoken"
        );
    }
}
