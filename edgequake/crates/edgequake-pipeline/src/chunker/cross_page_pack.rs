//! SPEC-135 P2 — merge undersize remainders across page attribution (not a second packer).
//!
//! Page is attribution. Prefer same-page packing. When the last chunk of page N
//! plus the first of page N+1 fit the budget and are not blocked, emit one chunk
//! with `page_start=N`, `page_end=M`.

use super::types::{ChunkResult, ChunkerConfig};
use crate::markdown_ir::parse_atx_heading;
use crate::token_estimator::count_tokens;

pub const PDF_CROSS_PAGE_PACK_ENV: &str = "EDGEQUAKE_PDF_CROSS_PAGE_PACK";

pub fn pdf_cross_page_pack_enabled() -> bool {
    super::env_flags::env_flag_default_on_var(PDF_CROSS_PAGE_PACK_ENV)
}

/// Merge adjacent page-stamped chunks when LAW-135-8 allows.
pub fn merge_cross_page_remainders(
    chunks: Vec<ChunkResult>,
    config: &ChunkerConfig,
) -> Vec<ChunkResult> {
    if chunks.len() < 2 {
        return reindex(chunks);
    }
    let budget = config.chunk_size.max(1);
    let floor = config.min_chunk_size.min(budget).max(1);

    let mut out: Vec<ChunkResult> = Vec::with_capacity(chunks.len());
    let mut iter = chunks.into_iter().peekable();

    while let Some(cur) = iter.next() {
        let Some(next) = iter.peek() else {
            out.push(cur);
            break;
        };
        if can_merge(&cur, next, budget, floor) {
            let next = iter.next().expect("peeked");
            out.push(merge_pair(cur, next));
        } else {
            out.push(cur);
        }
    }
    reindex(out)
}

fn can_merge(left: &ChunkResult, right: &ChunkResult, budget: usize, floor: usize) -> bool {
    let Some(ls) = left.page_start else {
        return false;
    };
    let Some(le) = left.page_end else {
        return false;
    };
    let Some(rs) = right.page_start else {
        return false;
    };
    // Only merge a page-N tail into the immediately following page (N+1 or same).
    if rs < le || rs > le.saturating_add(1) {
        return false;
    }
    if rs == ls && le == right.page_end.unwrap_or(rs) {
        // Same-page neighbors are the inner packer's job.
        return false;
    }
    if starts_with_h1(&right.content) {
        return false;
    }
    if right.tokens > budget || left.tokens > budget {
        return false;
    }
    if script_change(&left.content, &right.content) {
        return false;
    }
    let combined = count_tokens(&format!(
        "{}\n{}",
        left.content.trim_end(),
        right.content.trim_start()
    ));
    if combined > budget {
        return false;
    }
    let under_floor = left.tokens < floor;
    let same_section = match (&left.section, &right.section) {
        (Some(a), Some(b)) => !a.heading_path.is_empty() && a.heading_path == b.heading_path,
        _ => false,
    };
    under_floor || same_section || left.tokens + right.tokens <= budget
}

fn starts_with_h1(content: &str) -> bool {
    content
        .lines()
        .find(|l| !l.trim().is_empty())
        .is_some_and(|line| parse_atx_heading(line).is_some_and(|(level, _)| level == 1))
}

/// CJK vs Latin letter-ratio flip (LAW-135 E3).
fn script_change(a: &str, b: &str) -> bool {
    let (a_cjk, a_lat) = script_counts(a);
    let (b_cjk, b_lat) = script_counts(b);
    let a_total = a_cjk + a_lat;
    let b_total = b_cjk + b_lat;
    if a_total < 12 || b_total < 12 {
        return false;
    }
    let a_cjk_dom = a_cjk * 3 > a_lat * 2;
    let b_cjk_dom = b_cjk * 3 > b_lat * 2;
    a_cjk_dom != b_cjk_dom && ((a_cjk_dom && b_lat > b_cjk) || (b_cjk_dom && a_lat > a_cjk))
}

fn script_counts(s: &str) -> (usize, usize) {
    let mut cjk = 0usize;
    let mut lat = 0usize;
    for ch in s.chars() {
        if is_cjk(ch) {
            cjk += 1;
        } else if ch.is_ascii_alphabetic() {
            lat += 1;
        }
    }
    (cjk, lat)
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch,
        '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}'
    )
}

fn merge_pair(left: ChunkResult, right: ChunkResult) -> ChunkResult {
    let content = format!(
        "{}\n{}",
        left.content.trim_end(),
        right.content.trim_start()
    );
    let tokens = count_tokens(&content);
    let page_start = left.page_start.or(right.page_start);
    let page_end = match (left.page_end, right.page_end, right.page_start) {
        (Some(a), Some(b), _) => Some(a.max(b)),
        (Some(a), None, Some(b)) => Some(a.max(b)),
        (_, b, _) => b.or(right.page_start).or(left.page_end),
    };
    ChunkResult {
        content,
        tokens,
        chunk_order_index: left.chunk_order_index,
        section: left.section.or(right.section),
        start_offset: left.start_offset.or(right.start_offset),
        end_offset: right.end_offset.or(left.end_offset),
        page_start,
        page_end,
    }
}

fn reindex(mut chunks: Vec<ChunkResult>) -> Vec<ChunkResult> {
    for (i, c) in chunks.iter_mut().enumerate() {
        c.chunk_order_index = i;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ChunkerConfig {
        ChunkerConfig {
            chunk_size: 1200,
            chunk_overlap: 100,
            min_chunk_size: 100,
            ..Default::default()
        }
    }

    fn chunk(text: &str, start: u32, end: u32) -> ChunkResult {
        ChunkResult {
            content: text.to_string(),
            tokens: count_tokens(text),
            page_start: Some(start),
            page_end: Some(end),
            ..Default::default()
        }
    }

    #[test]
    fn merges_undersize_consecutive_pages() {
        let a = chunk("Short remainder on page one. EQ135_SPAN_P1.", 1, 1);
        let b = chunk("Continuation on page two. EQ135_SPAN_P2.", 2, 2);
        let out = merge_cross_page_remainders(vec![a, b], &cfg());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].page_start, Some(1));
        assert_eq!(out[0].page_end, Some(2));
    }

    #[test]
    fn blocks_new_h1() {
        let a = chunk("Short remainder on page one.", 1, 1);
        let b = chunk("# Completely different topic\n\nBody.", 2, 2);
        let out = merge_cross_page_remainders(vec![a, b], &cfg());
        assert_eq!(out.len(), 2);
    }
}
