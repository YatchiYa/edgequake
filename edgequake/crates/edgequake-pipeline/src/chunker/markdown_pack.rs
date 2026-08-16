//! SPEC-125 — structure-aware markdown packing (soft heading boundaries).
//!
//! Headings are preferred split points, not mandatory cuts. Continuation
//! pieces repeat ATX hierarchy; oversized tables repeat header + separator.
//! Sibling/overflow continuations use **boundary overlap** (last sentence of
//! the previous body after the ATX path), not a raw mid-sentence token slice.

use super::atomic_blocks::{split_preserving_atomic_regions, AtomicKind};
use super::recursive::RecursiveCharacterChunking;
use super::types::{ChunkResult, ChunkerConfig, SectionMetadata};
use crate::error::Result;
use crate::markdown_ir::{is_atx_heading_only_text, parse_atx_heading, PREFACE_HEADING};
use crate::table_preprocessor::is_separator_line;
use crate::token_estimator::count_tokens;

/// Fleet kill switch. Default **on** (unset). `0`/`false`/`off`/`no` restores heading-hard split.
pub const MARKDOWN_PACK_ENV: &str = "EDGEQUAKE_MARKDOWN_PACK";

/// Whether SPEC-125 packing is enabled (default true).
pub fn markdown_pack_enabled() -> bool {
    markdown_pack_flag(std::env::var(MARKDOWN_PACK_ENV).ok().as_deref())
}

pub(crate) fn markdown_pack_flag(raw: Option<&str>) -> bool {
    match raw {
        Some(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off" | "no"
        ),
        None => true,
    }
}

/// Token min/p50/max + heading-only orphan count (LAW-125-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkTokenStats {
    pub token_min: usize,
    pub token_p50: usize,
    pub token_max: usize,
    pub orphan_heading_chunks: usize,
}

impl ChunkTokenStats {
    /// Build stats from (token_count, content) pairs.
    pub fn from_pairs<'a, I>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (usize, &'a str)>,
    {
        let mut tokens = Vec::new();
        let mut orphan_heading_chunks = 0usize;
        for (n, text) in pairs {
            tokens.push(n);
            if is_atx_heading_only_text(text) {
                orphan_heading_chunks += 1;
            }
        }
        if tokens.is_empty() {
            return Self {
                token_min: 0,
                token_p50: 0,
                token_max: 0,
                orphan_heading_chunks: 0,
            };
        }
        tokens.sort_unstable();
        let token_min = tokens[0];
        let token_max = *tokens.last().unwrap_or(&0);
        let token_p50 = tokens[tokens.len() / 2];
        Self {
            token_min,
            token_p50,
            token_max,
            orphan_heading_chunks,
        }
    }

    /// LAW-124-8 / LAW-125-10: counts only — never chunk text.
    pub fn observation_output_json(&self, chunk_count: usize) -> String {
        format!(
            "{{\"chunks\":{},\"token_min\":{},\"token_p50\":{},\"token_max\":{},\"orphan_heading_chunks\":{}}}",
            chunk_count, self.token_min, self.token_p50, self.token_max, self.orphan_heading_chunks
        )
    }
}

/// Input/output JSON for `ingest.chunking` (SSOT used by pipeline + tests).
pub fn ingest_chunking_observation<'a, I>(
    content_chars: usize,
    pairs: I,
) -> (String, String, ChunkTokenStats)
where
    I: IntoIterator<Item = (usize, &'a str)>,
{
    let pairs: Vec<(usize, &str)> = pairs.into_iter().collect();
    let dist = ChunkTokenStats::from_pairs(pairs.iter().copied());
    let input = format!("{{\"chars\":{content_chars}}}");
    let output = dist.observation_output_json(pairs.len());
    (input, output, dist)
}

/// Markdown strategy body. `pack=true` is SPEC-125; `false` is heading-hard kill switch.
/// Structure induction (FAQ) still runs so Acc blobs keep breadcrumbs.
pub async fn markdown_chunk(
    content: &str,
    config: &ChunkerConfig,
    pack: bool,
) -> Result<Vec<ChunkResult>> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let induced = crate::structure_induce::maybe_induce_structure(content);
    if pack {
        pack_markdown(induced.as_str(), config).await
    } else {
        chunk_heading_hard(induced.as_str(), config).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitKind {
    Prose,
    Table,
    Fence,
    Multimodal,
}

#[derive(Debug, Clone)]
struct HeadingFrame {
    level: u8,
    text: String,
}

#[derive(Debug, Clone)]
struct PackUnit {
    content: String,
    start: usize,
    end: usize,
    stack: Vec<HeadingFrame>,
    kind: UnitKind,
}

/// Pack markdown into token-budgeted chunks (LAW-125-1..8).
pub async fn pack_markdown(content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let units = collect_units(content);
    pack_units(content, units, config).await
}

fn collect_units(content: &str) -> Vec<PackUnit> {
    let regions = split_preserving_atomic_regions(content);
    let mut units = Vec::new();
    let mut stack: Vec<HeadingFrame> = Vec::new();

    for region in regions {
        match region.atomic {
            Some(AtomicKind::PipeTable) => {
                push_nonempty(
                    &mut units,
                    PackUnit {
                        content: region.text,
                        start: region.start,
                        end: region.end,
                        stack: stack.clone(),
                        kind: UnitKind::Table,
                    },
                );
            }
            Some(AtomicKind::FencedCode) => {
                push_nonempty(
                    &mut units,
                    PackUnit {
                        content: region.text,
                        start: region.start,
                        end: region.end,
                        stack: stack.clone(),
                        kind: UnitKind::Fence,
                    },
                );
            }
            Some(AtomicKind::MultimodalChunk) => {
                push_nonempty(
                    &mut units,
                    PackUnit {
                        content: region.text,
                        start: region.start,
                        end: region.end,
                        stack: stack.clone(),
                        kind: UnitKind::Multimodal,
                    },
                );
            }
            None => {
                units.extend(plain_heading_units(&region.text, region.start, &mut stack));
            }
        }
    }
    units
}

fn push_nonempty(units: &mut Vec<PackUnit>, unit: PackUnit) {
    if !unit.content.trim().is_empty() {
        units.push(unit);
    }
}

fn plain_heading_units(text: &str, base: usize, stack: &mut Vec<HeadingFrame>) -> Vec<PackUnit> {
    let mut units = Vec::new();
    let mut cur_lines: Vec<String> = Vec::new();
    let mut block_start = base;
    let mut local = 0usize;
    let mut cur_stack = stack.clone();

    let flush = |units: &mut Vec<PackUnit>,
                 cur_lines: &mut Vec<String>,
                 cur_stack: &[HeadingFrame],
                 block_start: usize,
                 local_end: usize,
                 base: usize| {
        if cur_lines.is_empty() {
            return;
        }
        let body = cur_lines.join("\n");
        cur_lines.clear();
        if body.trim().is_empty() {
            return;
        }
        units.push(PackUnit {
            content: body,
            start: block_start,
            end: base + local_end,
            stack: cur_stack.to_vec(),
            kind: UnitKind::Prose,
        });
    };

    for line in text.split_inclusive('\n') {
        let line_len = line.len();
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some((level, heading_text)) = parse_atx_heading(trimmed) {
            flush(
                &mut units,
                &mut cur_lines,
                &cur_stack,
                block_start,
                local,
                base,
            );
            while stack.last().is_some_and(|h| h.level >= level) {
                stack.pop();
            }
            stack.push(HeadingFrame {
                level,
                text: heading_text,
            });
            cur_stack = stack.clone();
            block_start = base + local;
            cur_lines.push(trimmed.to_string());
        } else {
            if cur_lines.is_empty() {
                block_start = base + local;
                cur_stack = stack.clone();
            }
            cur_lines.push(trimmed.to_string());
        }
        local += line_len;
    }
    flush(
        &mut units,
        &mut cur_lines,
        &cur_stack,
        block_start,
        local,
        base,
    );
    units
}

fn atx_prefix(stack: &[HeadingFrame]) -> String {
    stack
        .iter()
        .filter(|h| !h.text.is_empty() && h.text != PREFACE_HEADING)
        .map(|h| {
            let hashes = "#".repeat(h.level.max(1) as usize);
            format!("{hashes} {}", h.text)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn with_atx_prefix(content: &str, stack: &[HeadingFrame]) -> String {
    let prefix = atx_prefix(stack);
    if prefix.is_empty() {
        return content.to_string();
    }
    let rest = strip_leading_stack_atx(content, stack);
    let rest = rest.trim_start();
    if rest.is_empty() {
        return content.to_string();
    }
    format!("{prefix}\n\n{rest}")
}

/// Body text with ATX lines removed (for overlap sentence extraction).
fn body_without_atx(content: &str) -> String {
    content
        .lines()
        .filter(|line| parse_atx_heading(line.trim()).is_none())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Last sentence (Latin `.!?` or CJK `。！？`), including the terminator.
fn last_full_sentence(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        return String::new();
    }
    let mut ends = vec![0usize];
    for (i, ch) in t.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            ends.push(i + ch.len_utf8());
        }
    }
    let last = *ends.last().unwrap_or(&0);
    if last < t.len() {
        return t[last..].trim().to_string();
    }
    if ends.len() >= 2 {
        let start = ends[ends.len() - 2];
        return t[start..last].trim().to_string();
    }
    t.to_string()
}

fn cap_to_tokens(text: String, max_tokens: usize) -> String {
    if max_tokens == 0 || text.is_empty() {
        return String::new();
    }
    if count_tokens(&text) <= max_tokens {
        return text;
    }
    let mut parts: Vec<&str> = text.split_inclusive(char::is_whitespace).collect();
    while parts.len() > 1 && count_tokens(&parts.concat()) > max_tokens {
        parts.remove(0);
    }
    parts.concat().trim().to_string()
}

/// SOTA overlap: ATX path (once) + last full sentence of previous body.
fn apply_boundary_overlap(
    prev: &str,
    next_prefixed: &str,
    stack: &[HeadingFrame],
    overlap_tokens: usize,
) -> String {
    if overlap_tokens == 0 {
        return next_prefixed.to_string();
    }
    let overlap = cap_to_tokens(last_full_sentence(&body_without_atx(prev)), overlap_tokens);
    if overlap.is_empty() {
        return next_prefixed.to_string();
    }
    let rest = strip_leading_stack_atx(next_prefixed, stack);
    let rest = rest.trim_start();
    if rest.starts_with(&overlap) {
        return next_prefixed.to_string();
    }
    let prefix = atx_prefix(stack);
    if prefix.is_empty() {
        format!("{overlap}\n\n{rest}")
    } else {
        format!("{prefix}\n\n{overlap}\n\n{rest}")
    }
}

fn finalize_emitted(
    raw: &str,
    stack: &[HeadingFrame],
    prev: Option<&str>,
    overlap_tokens: usize,
) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    match prev {
        None => raw.to_string(),
        Some(prev) => {
            let prefixed = with_atx_prefix(raw, stack);
            apply_boundary_overlap(prev, &prefixed, stack, overlap_tokens)
        }
    }
}

fn strip_leading_stack_atx(content: &str, stack: &[HeadingFrame]) -> String {
    let wanted: Vec<(u8, &str)> = stack
        .iter()
        .filter(|h| !h.text.is_empty() && h.text != PREFACE_HEADING)
        .map(|h| (h.level, h.text.as_str()))
        .collect();
    if wanted.is_empty() {
        return content.to_string();
    }
    let mut idx = 0usize;
    for line in content.split_inclusive('\n') {
        let raw = line.trim_end_matches('\n').trim_end_matches('\r');
        if raw.trim().is_empty() {
            idx += line.len();
            continue;
        }
        if let Some((lvl, text)) = parse_atx_heading(raw) {
            if wanted.iter().any(|(l, t)| *l == lvl && *t == text) {
                idx += line.len();
                continue;
            }
        }
        break;
    }
    content[idx.min(content.len())..].to_string()
}

fn section_of(stack: &[HeadingFrame]) -> Option<SectionMetadata> {
    let frames: Vec<&HeadingFrame> = stack
        .iter()
        .filter(|h| !h.text.is_empty() && h.text != PREFACE_HEADING)
        .collect();
    if frames.is_empty() {
        return None;
    }
    let leaf = frames.last()?;
    let parents: Vec<String> = frames[..frames.len() - 1]
        .iter()
        .map(|h| h.text.clone())
        .collect();
    Some(SectionMetadata::from_block(
        &parents, &leaf.text, leaf.level,
    ))
}

fn effective_min(config: &ChunkerConfig) -> usize {
    let budget = config.chunk_size.max(1);
    config.min_chunk_size.min(budget).max(1)
}

fn source_slice(source: &str, start: usize, end: usize) -> String {
    let start = source.floor_char_boundary(start.min(source.len()));
    let end = source.floor_char_boundary(end.min(source.len()).max(start));
    source[start..end].to_string()
}

fn buffer_text(source: &str, units: &[PackUnit]) -> String {
    if units.is_empty() {
        return String::new();
    }
    let start = units[0].start;
    let end = units.last().map(|u| u.end).unwrap_or(start);
    if end >= start && end <= source.len() {
        return source_slice(source, start, end);
    }
    units
        .iter()
        .map(|u| u.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_tokens(source: &str, units: &[PackUnit]) -> usize {
    count_tokens(&buffer_text(source, units))
}

async fn pack_units(
    source: &str,
    units: Vec<PackUnit>,
    config: &ChunkerConfig,
) -> Result<Vec<ChunkResult>> {
    let budget = config.chunk_size.max(1);
    let min_size = effective_min(config);
    let overlap = config.chunk_overlap;
    let mut results = Vec::new();
    let mut order = 0usize;
    let mut current: Vec<PackUnit> = Vec::new();
    let mut prev: Option<String> = None;

    for unit in units {
        let unit_tokens = count_tokens(&unit.content);
        if current.is_empty() {
            if unit_tokens > budget {
                let pieces = split_oversized(&unit, config, prev.as_deref()).await?;
                for piece in pieces {
                    prev = Some(piece.content.clone());
                    push_result(&mut results, &mut order, piece);
                }
            } else {
                current.push(unit);
            }
            continue;
        }

        let mut trial = current.clone();
        trial.push(unit.clone());
        let combined = buffer_tokens(source, &trial);
        if combined <= budget {
            current.push(unit);
            continue;
        }

        let current_text = buffer_text(source, &current);
        let heading_only = is_atx_heading_only_text(&current_text);
        let current_tok = count_tokens(&current_text);

        if heading_only || current_tok < min_size {
            // LAW-125-3: never emit orphan heading if a body follows.
            if unit_tokens > budget {
                let folded = fold_prefix_unit(&current, unit);
                let pieces = split_oversized(&folded, config, prev.as_deref()).await?;
                for piece in pieces {
                    prev = Some(piece.content.clone());
                    push_result(&mut results, &mut order, piece);
                }
                current.clear();
            } else {
                current.push(unit);
            }
            continue;
        }

        emit_buffer(
            source,
            &current,
            &mut results,
            &mut order,
            &mut prev,
            overlap,
        );
        current.clear();
        if unit_tokens > budget {
            let pieces = split_oversized(&unit, config, prev.as_deref()).await?;
            for piece in pieces {
                prev = Some(piece.content.clone());
                push_result(&mut results, &mut order, piece);
            }
        } else {
            current.push(unit);
        }
    }

    if !current.is_empty() {
        emit_buffer(
            source,
            &current,
            &mut results,
            &mut order,
            &mut prev,
            overlap,
        );
    }
    Ok(results
        .into_iter()
        .filter(|c| !c.content.trim().is_empty())
        .collect())
}

fn fold_prefix_unit(prefix: &[PackUnit], mut unit: PackUnit) -> PackUnit {
    let pre = prefix
        .iter()
        .map(|u| u.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    unit.content = format!("{}\n{}", pre.trim_end(), unit.content);
    if let Some(first) = prefix.first() {
        unit.start = first.start;
    }
    unit
}

fn emit_buffer(
    source: &str,
    units: &[PackUnit],
    results: &mut Vec<ChunkResult>,
    order: &mut usize,
    prev: &mut Option<String>,
    overlap_tokens: usize,
) {
    let raw = buffer_text(source, units);
    let stack = units.last().map(|u| u.stack.as_slice()).unwrap_or(&[]);
    let content = finalize_emitted(&raw, stack, prev.as_deref(), overlap_tokens);
    if content.is_empty() {
        return;
    }
    let section = units.last().and_then(|u| section_of(&u.stack));
    let start = units.first().map(|u| u.start);
    let end = units.last().map(|u| u.end);
    push_result(
        results,
        order,
        ChunkResult {
            content: content.clone(),
            tokens: count_tokens(&content),
            chunk_order_index: 0,
            section,
            start_offset: start,
            end_offset: end,
            page_start: None,
            page_end: None,
        },
    );
    *prev = Some(content);
}

fn push_result(results: &mut Vec<ChunkResult>, order: &mut usize, mut chunk: ChunkResult) {
    chunk.chunk_order_index = *order;
    if chunk.tokens == 0 {
        chunk.tokens = count_tokens(&chunk.content);
    }
    *order += 1;
    results.push(chunk);
}

async fn split_oversized(
    unit: &PackUnit,
    config: &ChunkerConfig,
    prev: Option<&str>,
) -> Result<Vec<ChunkResult>> {
    let mut pieces = match unit.kind {
        UnitKind::Table => split_table_unit(unit, config),
        UnitKind::Fence => split_fence_unit(unit, config),
        UnitKind::Multimodal => vec![prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        )],
        UnitKind::Prose => split_prose_unit(unit, config)?,
    };
    if let Some(prev) = prev {
        if let Some(first) = pieces.first_mut() {
            first.content = finalize_emitted(
                &first.content,
                &unit.stack,
                Some(prev),
                config.chunk_overlap,
            );
            first.tokens = count_tokens(&first.content);
        }
    }
    Ok(pieces)
}

fn is_fence_marker_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("```") || t.starts_with("~~~")
}

/// Oversized fence: keep opener (and language tag) + closer on every piece.
fn split_fence_unit(unit: &PackUnit, config: &ChunkerConfig) -> Vec<ChunkResult> {
    let budget = config.chunk_size.max(1);
    let lines: Vec<&str> = unit.content.split_inclusive('\n').collect();
    if lines.is_empty() {
        return vec![prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        )];
    }
    let opener_idx = lines
        .iter()
        .position(|l| is_fence_marker_line(l))
        .unwrap_or(0);
    let heading_prefix: String = lines[..opener_idx].concat();
    let opener = lines[opener_idx].to_string();
    let last = lines.len() - 1;
    let closer = if last > opener_idx && is_fence_marker_line(lines[last]) {
        lines[last].to_string()
    } else {
        "```\n".to_string()
    };
    let interior: Vec<&str> = if last > opener_idx && is_fence_marker_line(lines[last]) {
        lines[opener_idx + 1..last].to_vec()
    } else {
        lines.get(opener_idx + 1..).unwrap_or(&[]).to_vec()
    };

    let mut pieces = Vec::new();
    let mut batch: Vec<&str> = Vec::new();
    let mut batch_start = unit.start;
    let mut offset = unit.start + heading_prefix.len() + opener.len();

    let flush = |batch: &mut Vec<&str>,
                 pieces: &mut Vec<ChunkResult>,
                 batch_start: usize,
                 batch_end: usize| {
        if batch.is_empty() {
            return;
        }
        let mut body = String::new();
        if !heading_prefix.trim().is_empty() {
            body.push_str(heading_prefix.trim_end());
            body.push_str("\n\n");
        }
        body.push_str(&opener);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        for line in batch.iter() {
            body.push_str(line);
        }
        if !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(closer.trim_end());
        pieces.push(prefixed_result(unit, body, batch_start, batch_end, true));
        batch.clear();
    };

    for line in interior {
        let line_end = offset + line.len();
        batch.push(line);
        let mut trial = opener.clone();
        for l in &batch {
            trial.push_str(l);
        }
        trial.push_str(&closer);
        if count_tokens(&trial) > budget && batch.len() > 1 {
            batch.pop();
            flush(&mut batch, &mut pieces, batch_start, offset);
            batch.push(line);
            batch_start = offset;
        }
        offset = line_end;
    }
    flush(&mut batch, &mut pieces, batch_start, offset);
    if pieces.is_empty() {
        pieces.push(prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        ));
    }
    pieces
}

fn split_table_unit(unit: &PackUnit, config: &ChunkerConfig) -> Vec<ChunkResult> {
    let budget = config.chunk_size.max(1);
    let parsed = parse_pipe_table(&unit.content);
    if parsed.rows.is_empty() {
        return vec![prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        )];
    }
    let prefix = atx_prefix(&unit.stack);
    let mut pieces = Vec::new();
    let mut batch: Vec<TableRow> = Vec::new();

    let flush_batch = |batch: &mut Vec<TableRow>, pieces: &mut Vec<ChunkResult>| {
        if batch.is_empty() {
            return;
        }
        let mut body = String::new();
        if !prefix.is_empty() {
            body.push_str(&prefix);
            body.push_str("\n\n");
        }
        if !parsed.header.is_empty() {
            body.push_str(&parsed.header);
            body.push('\n');
        }
        let sep = if parsed.sep.is_empty() {
            synthesize_separator(&parsed.header)
        } else {
            parsed.sep.clone()
        };
        if !sep.is_empty() {
            body.push_str(&sep);
            body.push('\n');
        }
        for row in batch.iter() {
            body.push_str(&row.text);
            body.push('\n');
        }
        let start = unit.start + batch.first().map(|r| r.start).unwrap_or(0);
        let end = unit.start + batch.last().map(|r| r.end).unwrap_or(unit.content.len());
        pieces.push(prefixed_result(
            unit,
            body.trim_end().to_string(),
            start,
            end,
            false,
        ));
        batch.clear();
    };

    for row in &parsed.rows {
        batch.push(row.clone());
        let mut trial = String::new();
        if !prefix.is_empty() {
            trial.push_str(&prefix);
            trial.push_str("\n\n");
        }
        trial.push_str(&parsed.header);
        trial.push('\n');
        trial.push_str(
            if parsed.sep.is_empty() {
                synthesize_separator(&parsed.header)
            } else {
                parsed.sep.clone()
            }
            .as_str(),
        );
        trial.push('\n');
        for r in &batch {
            trial.push_str(&r.text);
            trial.push('\n');
        }
        if count_tokens(&trial) > budget && batch.len() > 1 {
            batch.pop();
            flush_batch(&mut batch, &mut pieces);
            batch.push(row.clone());
        }
    }
    flush_batch(&mut batch, &mut pieces);
    if pieces.is_empty() {
        pieces.push(prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        ));
    }
    pieces
}

fn synthesize_separator(header: &str) -> String {
    let cells = header.split('|').count().saturating_sub(2).max(1);
    let mut s = String::from("|");
    for _ in 0..cells {
        s.push_str(" --- |");
    }
    s
}

#[derive(Debug, Clone)]
struct TableRow {
    text: String,
    start: usize,
    end: usize,
}

struct ParsedTable {
    header: String,
    sep: String,
    rows: Vec<TableRow>,
}

fn parse_pipe_table(text: &str) -> ParsedTable {
    let mut header = String::new();
    let mut sep = String::new();
    let mut rows = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let start = offset;
        let end = offset + line.len();
        offset = end;
        let t = line.trim_end_matches('\n').trim_end_matches('\r').trim();
        if t.is_empty() {
            continue;
        }
        if is_separator_line(t) {
            if sep.is_empty() {
                sep = t.to_string();
            }
            continue;
        }
        if !t.starts_with('|') {
            continue;
        }
        if header.is_empty() {
            header = t.to_string();
        } else {
            rows.push(TableRow {
                text: t.to_string(),
                start,
                end,
            });
        }
    }
    ParsedTable { header, sep, rows }
}

fn split_prose_unit(unit: &PackUnit, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
    let recursive = RecursiveCharacterChunking;
    let mut sub = recursive.chunk_with_len(&unit.content, config, count_tokens)?;
    if sub.is_empty() {
        return Ok(vec![prefixed_result(
            unit,
            unit.content.clone(),
            unit.start,
            unit.end,
            true,
        )]);
    }
    let mut out = Vec::new();
    for (i, mut piece) in sub.drain(..).enumerate() {
        if let Some(start) = piece.start_offset.as_mut() {
            *start = start.saturating_add(unit.start);
        }
        if let Some(end) = piece.end_offset.as_mut() {
            *end = end.saturating_add(unit.start);
        }
        if i > 0 {
            piece.content = with_atx_prefix(&piece.content, &unit.stack);
        }
        piece.tokens = count_tokens(&piece.content);
        piece.section = section_of(&unit.stack);
        out.push(piece);
    }
    Ok(out)
}

fn prefixed_result(
    unit: &PackUnit,
    content: String,
    start: usize,
    end: usize,
    apply_prefix: bool,
) -> ChunkResult {
    let content = if apply_prefix {
        with_atx_prefix(&content, &unit.stack)
    } else {
        content
    };
    ChunkResult {
        content: content.clone(),
        tokens: count_tokens(&content),
        chunk_order_index: 0,
        section: section_of(&unit.stack),
        start_offset: Some(start),
        end_offset: Some(end),
        page_start: None,
        page_end: None,
    }
}

/// Legacy heading-hard split (kill switch / Acc rollback).
pub async fn chunk_heading_hard(content: &str, config: &ChunkerConfig) -> Result<Vec<ChunkResult>> {
    use super::types::ChunkingStrategy;
    use crate::markdown_ir::{extract_markdown_blocks, format_breadcrumb};

    let blocks = extract_markdown_blocks(content);
    let recursive = RecursiveCharacterChunking;
    let mut results = Vec::new();
    let mut order = 0usize;

    for block in blocks {
        let section = Some(SectionMetadata::from_block(
            &block.parent_headings,
            &block.heading,
            block.level,
        ));
        let _ = format_breadcrumb(&block.parent_headings, &block.heading);
        let base_offset = block.start_offset;
        let sub_chunks = recursive.chunk(&block.content, config).await?;
        if sub_chunks.is_empty() && !block.content.trim().is_empty() {
            results.push(ChunkResult {
                content: block.content.trim().to_string(),
                tokens: count_tokens(&block.content),
                chunk_order_index: order,
                section: section.clone(),
                start_offset: Some(base_offset),
                end_offset: Some(block.end_offset),
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
                sub.section = section.clone();
                order += 1;
                results.push(sub);
            }
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunker::types::ChunkerConfig;

    fn heading_dense_md() -> &'static str {
        include_str!("../../tests/fixtures/spec125/heading_dense.md")
    }

    fn cfg(size: usize) -> ChunkerConfig {
        ChunkerConfig {
            chunk_size: size,
            chunk_overlap: 10,
            min_chunk_size: 1,
            ..Default::default()
        }
    }

    fn assert_honest_tiktoken(chunks: &[ChunkResult]) {
        for c in chunks {
            assert_eq!(
                c.tokens,
                count_tokens(&c.content),
                "chunk {} tokens must equal tiktoken, got {} vs {}",
                c.chunk_order_index,
                c.tokens,
                count_tokens(&c.content)
            );
        }
    }

    fn nonempty_lines(content: &str) -> Vec<&str> {
        content.lines().filter(|l| !l.trim().is_empty()).collect()
    }

    fn atx_occurrences(content: &str, line: &str) -> usize {
        content.lines().filter(|l| l.trim() == line).count()
    }

    #[tokio::test]
    async fn heading_dense_packs_to_one_chunk_at_product_sizes() {
        for size in [600usize, 800, 1200] {
            let chunks = pack_markdown(heading_dense_md(), &cfg(size)).await.unwrap();
            assert_eq!(
                chunks.len(),
                1,
                "heading-dense fixture must pack to 1 chunk at size={size}, got {}",
                chunks.len()
            );
            assert!(
                !is_atx_heading_only_text(&chunks[0].content),
                "first chunk must not be heading-only"
            );
            assert!(chunks[0].content.contains("First Child"));
            assert!(chunks[0].content.contains("Second Child"));
            assert!(chunks[0].content.contains("Third Child"));
            assert_eq!(chunks[0].tokens, count_tokens(&chunks[0].content));
            assert_honest_tiktoken(&chunks);
        }
    }

    #[tokio::test]
    async fn kill_switch_restores_heading_hard_split() {
        let packed = markdown_chunk(heading_dense_md(), &cfg(1200), true)
            .await
            .unwrap();
        let hard = markdown_chunk(heading_dense_md(), &cfg(1200), false)
            .await
            .unwrap();
        assert_eq!(packed.len(), 1);
        assert!(
            hard.len() >= 4,
            "legacy hard-split must emit ~4 chunks, got {}",
            hard.len()
        );
        assert!(
            is_atx_heading_only_text(&hard[0].content),
            "legacy first chunk should be heading-only, got {:?}",
            hard[0].content
        );
    }

    #[test]
    fn markdown_pack_flag_defaults_on() {
        assert!(markdown_pack_flag(None));
        assert!(markdown_pack_flag(Some("1")));
        assert!(markdown_pack_flag(Some("true")));
        assert!(!markdown_pack_flag(Some("0")));
        assert!(!markdown_pack_flag(Some("false")));
        assert!(!markdown_pack_flag(Some("off")));
    }

    #[tokio::test]
    async fn continuation_repeats_atx_path_exactly_once() {
        let mut body = String::from("## Parent\n\n### Leaf\n\n");
        body.push_str(
            &"A long sentence about context that must overflow the tiny budget. ".repeat(80),
        );
        let mut config = cfg(40);
        config.chunk_overlap = 12;
        let chunks = pack_markdown(&body, &config).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            chunks.len() >= 2,
            "expected overflow split, got {}",
            chunks.len()
        );
        let later = &chunks[1].content;
        let lead = nonempty_lines(later);
        assert!(
            lead.len() >= 2,
            "continuation must start with ATX path, got {later:?}"
        );
        assert_eq!(lead[0], "## Parent", "got {later:?}");
        assert_eq!(lead[1], "### Leaf", "got {later:?}");
        assert_eq!(atx_occurrences(later, "## Parent"), 1);
        assert_eq!(atx_occurrences(later, "### Leaf"), 1);
    }

    #[tokio::test]
    async fn sibling_boundary_overlap_keeps_atx_once() {
        let alpha = format!(
            "### Alpha\n\n{}",
            "Alpha unique sentence ends here. ".repeat(4)
        );
        let beta = format!(
            "### Beta\n\n{}",
            "Beta unique sentence ends here. ".repeat(4)
        );
        let md = format!("## Parent\n\n{alpha}\n{beta}");
        let mut config = cfg(45);
        config.chunk_overlap = 16;
        let chunks = pack_markdown(&md, &config).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            chunks.len() >= 2,
            "siblings under budget that overflow together must split, got {} {:?}",
            chunks.len(),
            chunks.iter().map(|c| c.content.clone()).collect::<Vec<_>>()
        );
        let later = &chunks
            .iter()
            .find(|c| c.content.contains("Beta unique sentence ends here."))
            .expect("beta body must appear in some chunk")
            .content;
        let lead = nonempty_lines(later);
        assert_eq!(lead[0], "## Parent", "got {later:?}");
        assert_eq!(lead[1], "### Beta", "got {later:?}");
        assert_eq!(atx_occurrences(later, "## Parent"), 1);
        assert_eq!(atx_occurrences(later, "### Beta"), 1);
        assert!(
            later.contains("Alpha unique sentence ends here."),
            "boundary overlap must carry last sentence of previous body, got {later:?}"
        );
        assert!(later.contains("Beta unique sentence ends here."));
    }

    #[tokio::test]
    async fn fence_overflow_repeats_opener_and_closer() {
        let mut md = String::from("## Code\n\n```rust\n");
        for i in 0..80 {
            md.push_str(&format!("fn item_{i}() {{ let x = {i}; }}\n"));
        }
        md.push_str("```\n");
        let chunks = pack_markdown(&md, &cfg(40)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            chunks.len() > 1,
            "oversized fence must split, got {}",
            chunks.len()
        );
        for c in &chunks {
            assert!(
                c.content.contains("```rust"),
                "each fence piece must reopen, got {:?}",
                c.content.chars().take(80).collect::<String>()
            );
            assert!(
                c.content.trim_end().ends_with("```"),
                "each fence piece must close, got {:?}",
                c.content.chars().rev().take(40).collect::<String>()
            );
        }
    }

    #[tokio::test]
    async fn vlm_figure_block_stays_atomic_with_following_prose() {
        let md = "# Figure 1 Revenue\n\n**Type:** Chart\n\n**Key values:**\n- Q4: 42\n\nAfter the figure.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("**Type:** Chart"));
        assert!(chunks[0].content.contains("After the figure."));
    }

    #[tokio::test]
    async fn oversized_mm_block_stays_one_chunk() {
        let mut md = String::from("[Chart Name]revenue\n[Image Type]Chart\n\n");
        md.push_str(&"Q4 value 42 with extra commentary. ".repeat(80));
        let chunks = pack_markdown(&md, &cfg(40)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert_eq!(
            chunks.len(),
            1,
            "MM region must stay atomic even over budget, got {}",
            chunks.len()
        );
        assert!(chunks[0].content.contains("[Chart Name]revenue"));
    }

    #[tokio::test]
    async fn table_overflow_repeats_header_and_separator() {
        let mut md = String::from("| ColA | ColB |\n| --- | --- |\n");
        for i in 0..40 {
            md.push_str(&format!("| value{i} | more{i} extra text for tokens |\n"));
        }
        let chunks = pack_markdown(&md, &cfg(30)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            chunks.len() > 1,
            "huge table must split, got {}",
            chunks.len()
        );
        for c in &chunks {
            assert!(
                c.content.contains("| ColA | ColB |"),
                "each table piece must repeat header, got {:?}",
                c.content
            );
            assert!(
                c.content.lines().any(|l| is_separator_line(l.trim())),
                "each table piece must include separator, got {:?}",
                c.content
            );
        }
        let starts: Vec<Option<usize>> = chunks.iter().map(|c| c.start_offset).collect();
        assert!(
            starts.windows(2).any(|w| w[0] != w[1]),
            "table pieces must not share identical start offsets: {starts:?}"
        );
    }

    #[tokio::test]
    async fn atx_inside_fence_is_not_a_split() {
        let md = "Intro paragraph.\n\n```\n# not a heading\n## also not\n```\n\nOutro paragraph that stays nearby.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(
            chunks.len(),
            1,
            "fenced ATX must not split, got {:?}",
            chunks
        );
        assert!(chunks[0].content.contains("# not a heading"));
    }

    #[tokio::test]
    async fn tilde_fence_hides_atx() {
        let md = "Before.\n\n~~~\n# fake\n~~~\n\nAfter text.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("# fake"));
    }

    #[tokio::test]
    async fn html_h2_is_not_a_heading() {
        let md = "<h2>Nope</h2>\n\nParagraph under html.\n\n## Real\n\nBody.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("<h2>Nope</h2>"));
    }

    #[tokio::test]
    async fn blockquote_atx_is_not_a_heading() {
        let md = "> ## Quoted\n\nReal paragraph.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[tokio::test]
    async fn heading_only_document_emits_remainder() {
        let chunks = pack_markdown("## Title Only\n", &cfg(1200)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(is_atx_heading_only_text(&chunks[0].content));
    }

    #[tokio::test]
    async fn crlf_packs_like_lf() {
        let lf = "## A\n\nBody one.\n\n## B\n\nBody two.\n";
        let crlf = lf.replace('\n', "\r\n");
        let a = pack_markdown(lf, &cfg(800)).await.unwrap();
        let b = pack_markdown(&crlf, &cfg(800)).await.unwrap();
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), 1);
    }

    #[tokio::test]
    async fn empty_input() {
        let chunks = pack_markdown("   \n\n  ", &cfg(800)).await.unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn stats_detect_orphans() {
        let stats = ChunkTokenStats::from_pairs([(3, "## Only"), (20, "## H\n\nbody")]);
        assert_eq!(stats.orphan_heading_chunks, 1);
        assert_eq!(stats.token_min, 3);
        assert_eq!(stats.token_max, 20);
    }

    #[tokio::test]
    async fn min_chunk_size_packs_undersize_siblings() {
        let md = "## A\n\nHi.\n\n## B\n\nYo.\n";
        let mut tiny = cfg(8);
        tiny.min_chunk_size = 1;
        let split = pack_markdown(md, &tiny).await.unwrap();
        assert!(
            split.len() >= 2,
            "without floor, tiny budget should emit siblings separately, got {}",
            split.len()
        );

        let mut floored = cfg(8);
        floored.min_chunk_size = 50;
        let packed = pack_markdown(md, &floored).await.unwrap();
        assert_eq!(
            packed.len(),
            1,
            "min_chunk_size must keep packing undersize siblings, got {:?}",
            packed.iter().map(|c| &c.content).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn min_chunk_size_above_budget_clamps() {
        let mut over = cfg(600);
        over.min_chunk_size = 10_000;
        let chunks = pack_markdown(heading_dense_md(), &over).await.unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[tokio::test]
    async fn unclosed_fence_is_atomic_remainder() {
        let md = "Intro paragraph.\n\n```\n# not a heading\nstill in fence\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("# not a heading"));
    }

    #[tokio::test]
    async fn setext_is_not_a_heading_v1() {
        let md = "Title\n=====\n\nA short body that would orphan if setext split.\n";
        let mut tiny = cfg(6);
        tiny.chunk_overlap = 0;
        let chunks = pack_markdown(md, &tiny).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            !chunks.iter().any(|c| is_atx_heading_only_text(&c.content)),
            "setext must not emit a heading-only Title chunk, got {:?}",
            chunks.iter().map(|c| &c.content).collect::<Vec<_>>()
        );
        assert!(chunks.iter().any(|c| c.content.contains("Title")));
        assert!(chunks.iter().any(|c| c.content.contains("short body")));
        assert!(parse_atx_heading("=====").is_none());
        assert!(parse_atx_heading("Title").is_none());
    }

    #[tokio::test]
    async fn heading_level_skip_prefix_uses_hash_then_h3() {
        let mut md = String::from("# Top\n\n### Nested\n\n");
        md.push_str(&"Continuation body sentence for overflow packing tests. ".repeat(80));
        let mut config = cfg(40);
        config.chunk_overlap = 8;
        let chunks = pack_markdown(&md, &config).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(chunks.len() >= 2, "got {}", chunks.len());
        let lead = nonempty_lines(&chunks[1].content);
        assert_eq!(lead[0], "# Top", "got {:?}", chunks[1].content);
        assert_eq!(lead[1], "### Nested", "got {:?}", chunks[1].content);
        assert_eq!(atx_occurrences(&chunks[1].content, "# Top"), 1);
        assert_eq!(atx_occurrences(&chunks[1].content, "### Nested"), 1);
    }

    #[tokio::test]
    async fn table_without_separator_synthesizes_sep() {
        let mut md = String::from("| ColA | ColB |\n");
        for i in 0..40 {
            md.push_str(&format!("| value{i} | more{i} extra text for tokens |\n"));
        }
        let chunks = pack_markdown(&md, &cfg(30)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(chunks.len() > 1, "got {}", chunks.len());
        for c in &chunks {
            assert!(c.content.contains("| ColA | ColB |"));
            assert!(
                c.content.lines().any(|l| is_separator_line(l.trim())),
                "synthesized separator missing: {:?}",
                c.content
            );
        }
    }

    #[tokio::test]
    async fn table_after_list_still_repeats_header() {
        let mut md = String::from("- note\n\n| ColA | ColB |\n| --- | --- |\n");
        for i in 0..40 {
            md.push_str(&format!("| value{i} | more{i} extra text for tokens |\n"));
        }
        let chunks = pack_markdown(&md, &cfg(30)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        let table_pieces: Vec<_> = chunks
            .iter()
            .filter(|c| c.content.contains("| ColA | ColB |"))
            .collect();
        assert!(table_pieces.len() > 1, "list-adjacent table must overflow");
        for c in table_pieces {
            assert!(c.content.lines().any(|l| is_separator_line(l.trim())));
        }
    }

    #[tokio::test]
    async fn fenced_table_is_not_pipe_split() {
        let mut fenced = String::from("```\n| ColA | ColB |\n| --- | --- |\n");
        for i in 0..40 {
            fenced.push_str(&format!("| value{i} | more{i} extra text for tokens |\n"));
        }
        fenced.push_str("```\n");
        let fence_chunks = pack_markdown(&fenced, &cfg(30)).await.unwrap();
        assert_honest_tiktoken(&fence_chunks);
        assert!(
            fence_chunks.iter().any(|c| c.content.contains("```")),
            "fence opener must survive, got {:?}",
            fence_chunks.iter().map(|c| &c.content).collect::<Vec<_>>()
        );
        let pipe = fenced.replace("```\n", "").replace("```", "");
        let pipe_chunks = pack_markdown(&pipe, &cfg(30)).await.unwrap();
        assert!(pipe_chunks.len() > 1);
        for c in &pipe_chunks {
            assert!(c.content.contains("| ColA | ColB |"));
            assert!(c.content.lines().any(|l| is_separator_line(l.trim())));
        }
    }

    #[tokio::test]
    async fn mm_table_name_stays_atomic_under_budget() {
        let md = "[Table Name]sales\n[Image Type]Table\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\nAfter.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].content.contains("[Table Name]sales"));
        assert!(chunks[0].content.contains("After."));
    }

    #[tokio::test]
    async fn structure_induce_faq_packs_without_orphan() {
        let prose = "About BCC. What is basal cell skin cancer? It is common. \
How is basal cell skin cancer treated? Surgery usually.";
        let induced = crate::structure_induce::induce_faq_markdown(prose);
        assert!(
            induced.contains("## What is basal cell skin cancer?"),
            "induce must emit ATX, got {induced}"
        );
        let chunks = pack_markdown(&induced, &cfg(800)).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert_eq!(chunks.len(), 1);
        assert!(!is_atx_heading_only_text(&chunks[0].content));
    }

    #[tokio::test]
    async fn cjk_uses_tiktoken_not_whitespace_words() {
        let md =
            "## 研究\n\n这是一段没有空格的中文内容用于证明分词。\n\n### 方法\n\n继续中文正文。\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_honest_tiktoken(&chunks);
        let words = chunks[0].content.split_whitespace().count();
        assert_ne!(
            chunks[0].tokens, words,
            "CJK tiktoken must not equal whitespace word count ({words})"
        );
    }

    #[tokio::test]
    async fn long_heading_prefix_is_not_truncated() {
        let long = "Alpha".repeat(30);
        let mut md = format!("## {long}\n\n### Leaf\n\n");
        md.push_str(&"Body sentence for overflow of a long heading path. ".repeat(80));
        let mut config = cfg(40);
        config.chunk_overlap = 8;
        let chunks = pack_markdown(&md, &config).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(chunks.len() >= 2);
        let expected = format!("## {long}");
        assert!(
            chunks[1].content.contains(&expected),
            "full heading must appear, got {:?}",
            chunks[1].content.chars().take(80).collect::<String>()
        );
        assert_eq!(atx_occurrences(&chunks[1].content, expected.as_str()), 1);
    }

    #[tokio::test]
    async fn budget_smaller_than_atx_prefix_still_keeps_body() {
        let heading = "H".repeat(80);
        let mut md = format!("## {heading}\n\n### Leaf\n\n");
        md.push_str(&"body word ".repeat(40));
        let prefix = format!("## {heading}\n### Leaf");
        let prefix_tok = count_tokens(&prefix);
        let mut config = cfg(prefix_tok.saturating_sub(1).max(8));
        config.chunk_overlap = 0;
        let chunks = pack_markdown(&md, &config).await.unwrap();
        assert_honest_tiktoken(&chunks);
        assert!(
            chunks
                .iter()
                .any(|c| c.content.contains("body word") && !is_atx_heading_only_text(&c.content)),
            "must not emit prefix-only while body exists: {:?}",
            chunks.iter().map(|c| &c.content).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn three_space_indent_is_atx_four_is_not() {
        let md = "   ## Indented\n\nBody under three-space ATX.\n\n    ## Codeish\n\nStill body.\n";
        let chunks = pack_markdown(md, &cfg(800)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0]
            .section
            .as_ref()
            .is_some_and(|s| s.heading_path.iter().any(|h| h == "Indented")));
        assert!(parse_atx_heading("   ## Indented").is_some());
        assert!(parse_atx_heading("    ## Codeish").is_none());
    }

    #[tokio::test]
    async fn heading_dense_observation_json_has_no_orphans_or_body() {
        let chunks = pack_markdown(heading_dense_md(), &cfg(600)).await.unwrap();
        assert_eq!(chunks.len(), 1);
        let (input, output, dist) = ingest_chunking_observation(
            heading_dense_md().len(),
            chunks.iter().map(|c| (c.tokens, c.content.as_str())),
        );
        assert!(input.contains(&heading_dense_md().len().to_string()));
        assert_eq!(dist.orphan_heading_chunks, 0);
        assert_eq!(dist.token_min, chunks[0].tokens);
        assert_eq!(dist.token_max, chunks[0].tokens);
        assert_eq!(output, dist.observation_output_json(1));
        assert!(!output.contains("PACKPROBE"));
        assert!(!output.contains("## "));
        assert!(output.contains("\"orphan_heading_chunks\":0"));
    }

    #[tokio::test]
    async fn pdf_filename_still_selects_pdf_strategy() {
        use crate::chunker::registry::ChunkStrategy;
        assert_eq!(
            ChunkStrategy::resolve_for_upload(None, None, "scan.pdf"),
            ChunkStrategy::Pdf
        );
    }
}
