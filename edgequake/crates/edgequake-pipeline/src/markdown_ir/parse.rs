//! Markdown block extraction with heading stack (LightRAG extract.py parity).

use regex::Regex;
use std::sync::LazyLock;

pub const PREFACE_HEADING: &str = "Preface/Uncategorized";

static ATX_HEADING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(#{1,6})\s+(.*?)\s*$").unwrap());

/// Parse a CommonMark ATX heading line (`#`–`######` + space + text).
///
/// Up to three leading spaces are allowed (CommonMark). A blockquote
/// (`> ## x`) is not a heading. Four-space indent is a code block, not ATX.
pub fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_end_matches('\r').trim_end();
    if trimmed.starts_with('>') {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let mut indent = 0usize;
    while indent < 3 && indent < bytes.len() && bytes[indent] == b' ' {
        indent += 1;
    }
    let rest = &trimmed[indent..];
    let caps = ATX_HEADING.captures(rest)?;
    let level = caps.get(1).map(|m| m.as_str().len()).unwrap_or(1) as u8;
    let heading_text = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
    Some((level, clean_heading(&heading_text)))
}

/// True when every non-empty line is an ATX heading (orphan / heading-only chunk).
pub fn is_atx_heading_only_text(text: &str) -> bool {
    let mut any = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        any = true;
        if parse_atx_heading(line).is_none() {
            return false;
        }
    }
    any
}

/// A contiguous markdown block under one heading context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownBlock {
    pub heading: String,
    pub level: u8,
    pub parent_headings: Vec<String>,
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Walk markdown lines and produce heading-aware blocks.
pub fn extract_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut heading_stack: Vec<(u8, String)> = Vec::new();
    let mut cur_heading = PREFACE_HEADING.to_string();
    let mut cur_level: u8 = 0;
    let mut cur_parents: Vec<String> = Vec::new();
    let mut cur_lines: Vec<String> = Vec::new();
    let mut block_start: usize = 0;
    let mut byte_offset: usize = 0;

    let flush = |blocks: &mut Vec<MarkdownBlock>,
                 cur_lines: &mut Vec<String>,
                 cur_heading: &str,
                 cur_level: u8,
                 cur_parents: &[String],
                 block_start: usize,
                 byte_offset: usize| {
        if cur_lines.is_empty() {
            return;
        }
        let text = cur_lines.join("\n");
        if text.trim().is_empty() {
            cur_lines.clear();
            return;
        }
        blocks.push(MarkdownBlock {
            heading: cur_heading.to_string(),
            level: cur_level,
            parent_headings: cur_parents.to_vec(),
            content: text,
            start_offset: block_start,
            end_offset: byte_offset,
        });
        cur_lines.clear();
    };

    for line in content.split_inclusive('\n') {
        let line_len = line.len();
        let trimmed = line.trim_end_matches('\n');

        if let Some((level, heading_text)) = parse_atx_heading(trimmed) {
            flush(
                &mut blocks,
                &mut cur_lines,
                &cur_heading,
                cur_level,
                &cur_parents,
                block_start,
                byte_offset,
            );

            heading_stack.truncate(level.saturating_sub(1) as usize);
            cur_parents = heading_stack.iter().map(|(_, h)| h.clone()).collect();
            heading_stack.push((level, heading_text.clone()));

            cur_heading = heading_text;
            cur_level = level;
            block_start = byte_offset;
            cur_lines.push(trimmed.to_string());
        } else {
            if cur_lines.is_empty() {
                block_start = byte_offset;
            }
            cur_lines.push(trimmed.to_string());
        }

        byte_offset += line_len;
    }

    flush(
        &mut blocks,
        &mut cur_lines,
        &cur_heading,
        cur_level,
        &cur_parents,
        block_start,
        byte_offset,
    );

    if blocks.is_empty() && !content.trim().is_empty() {
        blocks.push(MarkdownBlock {
            heading: PREFACE_HEADING.to_string(),
            level: 0,
            parent_headings: vec![],
            content: content.to_string(),
            start_offset: 0,
            end_offset: content.len(),
        });
    }

    blocks
}

fn clean_heading(raw: &str) -> String {
    raw.trim().trim_end_matches('#').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_heading_stack() {
        let md = "# Install\n\nBody one.\n\n## Prerequisites\n\nBody two.";
        let blocks = extract_markdown_blocks(md);
        assert!(blocks.len() >= 2);
        let prereq = blocks
            .iter()
            .find(|b| b.heading == "Prerequisites")
            .unwrap();
        assert_eq!(prereq.parent_headings, vec!["Install"]);
        assert_eq!(prereq.level, 2);
    }

    #[test]
    fn resets_stack_on_h1() {
        let md = "# A\n\n## B\n\n# C\n\nText under C.";
        let blocks = extract_markdown_blocks(md);
        let c_block = blocks.iter().find(|b| b.heading == "C").unwrap();
        assert!(c_block.parent_headings.is_empty());
    }

    #[test]
    fn commonmark_indent_atx() {
        assert!(parse_atx_heading("## Ok").is_some());
        assert!(parse_atx_heading("   ## Three").is_some());
        assert!(parse_atx_heading("    ## Four").is_none());
        assert!(parse_atx_heading("> ## Quote").is_none());
        assert!(parse_atx_heading("Title").is_none());
        assert!(parse_atx_heading("=====").is_none());
    }
}
