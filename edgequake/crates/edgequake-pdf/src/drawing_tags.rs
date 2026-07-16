//! LightRAG `<drawing/>` placeholder SSOT (SPEC-047 Phase C MV-21 / MV-26/28).
//!
//! Single source for:
//! - viewer-visible markdown images (`![alt](assets/…)`)
//! - VLM-scannable `<drawing …/>` tags ([`crate::inline_images::scan_inline_image_refs`])

use std::path::Path;

/// Relative assets subdirectory under the document mm-assets root.
pub const ASSETS_SUBDIR: &str = "assets";

/// Max chars of Pass A body embedded into drawing caption (MV-26 routing hints).
const CAPTION_BODY_BUDGET: usize = 160;

/// Parse 1-indexed page number from an mm-asset relative path (`assets/page-0003-fig-01.png`).
///
/// DRY with `edgequake_storage::classify_mm_asset_path` — same `page-NNNN` stem convention.
pub fn page_num_from_asset_rel_path(asset_path: &str) -> Option<u32> {
    let name = asset_path.rsplit('/').next().unwrap_or(asset_path).trim();
    let rest = name.strip_prefix("page-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

/// Filename pattern for full-page vision renders (`page-0001.png`).
pub fn page_asset_filename(page_num: usize) -> String {
    format!("page-{page_num:04}.png")
}

/// Filename for MV-24 hi-res chart crop (`page-0001-chart.png`).
pub fn page_chart_crop_filename(page_num: usize) -> String {
    format!("page-{page_num:04}-chart.png")
}

/// Filename for an embedded ImageXObject figure (`page-0001-fig-01.png`).
///
/// First principle: VLM analyze paths use figure objects, not full-page rasters.
pub fn page_figure_asset_filename(page_num: usize, fig_index: usize) -> String {
    format!("page-{page_num:04}-fig-{fig_index:02}.png")
}

/// Filename for a caption-anchored table crop (`page-0001-table-01.png`).
pub fn page_table_asset_filename(page_num: usize, table_index: usize) -> String {
    format!("page-{page_num:04}-table-{table_index:02}.png")
}

/// Relative path for table crop (`assets/page-0001-table-01.png`).
pub fn page_table_asset_rel_path(page_num: usize, table_index: usize) -> String {
    format!(
        "{}/{}",
        ASSETS_SUBDIR,
        page_table_asset_filename(page_num, table_index)
    )
}

/// Relative path for embedded figure (`assets/page-0001-fig-01.png`).
pub fn page_figure_asset_rel_path(page_num: usize, fig_index: usize) -> String {
    format!(
        "{}/{}",
        ASSETS_SUBDIR,
        page_figure_asset_filename(page_num, fig_index)
    )
}

/// Stable drawing item id for an embedded figure on a page.
pub fn page_figure_drawing_item_id(
    page_num: usize,
    fig_index: usize,
    id_prefix: Option<&str>,
) -> String {
    match id_prefix.filter(|p| !p.is_empty()) {
        Some(prefix) => {
            let slug: String = prefix
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect();
            let slug = slug.trim_matches('-');
            if slug.is_empty() {
                format!("im-page-{page_num:04}-fig-{fig_index:02}")
            } else {
                format!("im-{slug}-page-{page_num:04}-fig-{fig_index:02}")
            }
        }
        None => format!("im-page-{page_num:04}-fig-{fig_index:02}"),
    }
}

/// Relative path from document mm-assets root (`assets/page-0001.png`).
pub fn page_asset_rel_path(page_num: usize) -> String {
    format!("{}/{}", ASSETS_SUBDIR, page_asset_filename(page_num))
}

/// Stable REST asset id from a relative path or filename (`page-0001-fig-01`).
///
/// First principle: id is the durable identity of one binary under a document —
/// independent of storage layout (`assets/…`) and of drawing tag prefixes.
pub fn asset_id_from_rel_path(asset_rel_path: &str) -> String {
    let name = asset_rel_path
        .rsplit('/')
        .next()
        .unwrap_or(asset_rel_path)
        .trim();
    Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name)
        .to_string()
}

/// Relative path for chart crop asset (`assets/page-0001-chart.png`).
pub fn page_chart_crop_rel_path(page_num: usize) -> String {
    format!("{}/{}", ASSETS_SUBDIR, page_chart_crop_filename(page_num))
}

/// True when `path` is a full-page raster (`assets/page-NNNN.png`), not a figure/chart crop.
///
/// Full-page rasters are dual-pane viewer context only — never Drawing / VLM targets.
pub fn is_full_page_asset_rel_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).trim();
    if name.contains("-fig-") || name.contains("-chart") || name.contains("-table-") {
        return false;
    }
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !stem.starts_with("page-") || stem.len() != 9 {
        return false;
    }
    stem[5..].bytes().all(|b| b.is_ascii_digit())
}

/// Asset eligible as a Drawing / markdown figure: embedded fig, chart crop, or table crop.
pub fn is_drawing_eligible_asset_rel_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).trim();
    name.contains("-fig-") || name.contains("-chart") || name.contains("-table-")
}

/// Stable drawing item id for a vision page render.
pub fn page_drawing_item_id(page_num: usize, id_prefix: Option<&str>) -> String {
    match id_prefix.filter(|p| !p.is_empty()) {
        Some(prefix) => {
            let slug: String = prefix
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect();
            let slug = slug.trim_matches('-');
            if slug.is_empty() {
                format!("im-page-{page_num:04}")
            } else {
                format!("im-{slug}-page-{page_num:04}")
            }
        }
        None => format!("im-page-{page_num:04}"),
    }
}

/// Stable drawing item id for an MV-24 residual chart crop (`page-NNNN-chart.png`).
///
/// Distinct from [`page_drawing_item_id`] / fig ids so chart specialize can run
/// **alongside** embedded figures (026 W1-coexist) without colliding.
pub fn page_chart_drawing_item_id(page_num: usize, id_prefix: Option<&str>) -> String {
    match id_prefix.filter(|p| !p.is_empty()) {
        Some(prefix) => {
            let slug: String = prefix
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect();
            let slug = slug.trim_matches('-');
            if slug.is_empty() {
                format!("im-page-{page_num:04}-chart")
            } else {
                format!("im-{slug}-page-{page_num:04}-chart")
            }
        }
        None => format!("im-page-{page_num:04}-chart"),
    }
}

/// Standard markdown image for the markdown viewer (`![alt](assets/…)`).
pub fn format_inline_asset_image(alt: &str, asset_rel_path: &str) -> String {
    let alt = alt.replace(['[', ']'], "");
    format!("![{alt}]({asset_rel_path})")
}

/// True when `url` already points at our durable mm-asset path SSOT.
pub fn is_durable_mm_asset_url(url: &str) -> bool {
    let u = url.trim();
    u.starts_with("assets/")
        || u.contains("/mm-assets/")
        || u.contains("/assets/") && u.contains("/documents/")
}

/// Bind VLM-hallucinated / empty figure image links to the durable page asset.
///
/// First principle: every visible figure in markdown must resolve to a real
/// stored asset under the document — never a model-invented relative path.
pub fn bind_figure_images_to_page_asset(markdown: &str, asset_rel_path: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let bytes = markdown.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((end, alt, url)) = parse_markdown_image_at(markdown, i) {
                if is_durable_mm_asset_url(url) {
                    out.push_str(&markdown[i..end]);
                } else {
                    let alt_clean = if alt.trim().is_empty() {
                        "Page image"
                    } else {
                        alt
                    };
                    out.push_str(&format_inline_asset_image(alt_clean, asset_rel_path));
                }
                i = end;
                continue;
            }
        }
        out.push(markdown[i..].chars().next().unwrap());
        i += markdown[i..].chars().next().unwrap().len_utf8();
    }
    out
}

/// Parse `![alt](url)` starting at `start` (index of `!`). Returns end index (exclusive).
pub(crate) fn parse_markdown_image_at(s: &str, start: usize) -> Option<(usize, &str, &str)> {
    let rest = &s[start..];
    if !rest.starts_with("![") {
        return None;
    }
    let alt_start = start + 2;
    let alt_end = s[alt_start..].find(']')? + alt_start;
    let after_alt = &s[alt_end..];
    if !after_alt.starts_with("](") {
        return None;
    }
    let url_start = alt_end + 2;
    let url_end = s[url_start..].find(')')? + url_start;
    let alt = &s[alt_start..alt_end];
    let url = s[url_start..url_end].trim();
    Some((url_end + 1, alt, url))
}

/// True when a line looks like a figure heading (`## Figure 1`, `**Figure 1:**`, …).
fn is_figure_heading_line(line: &str) -> bool {
    is_numbered_caption_heading(line, "figure ")
}

/// True when a line looks like a table caption (`## Table 1`, `**Table 1:**`, …).
fn is_table_heading_line(line: &str) -> bool {
    is_numbered_caption_heading(line, "table ")
}

fn is_numbered_caption_heading(line: &str, prefix: &str) -> bool {
    let t = line.trim();
    let stripped = t
        .trim_start_matches('#')
        .trim_start()
        .trim_start_matches(['*', '_'])
        .trim_start();
    let lower = stripped.to_ascii_lowercase();
    lower.starts_with(prefix)
        && lower
            .chars()
            .nth(prefix.len())
            .is_some_and(|c| c.is_ascii_digit())
}

/// Extract a short alt label from a figure/table heading line.
fn caption_heading_alt(line: &str, fallback: &str) -> String {
    let t = line.trim();
    let stripped = t
        .trim_start_matches('#')
        .trim_start()
        .trim_matches('*')
        .trim()
        .trim_matches('*')
        .trim();
    let alt: String = stripped.chars().take(120).collect();
    if alt.is_empty() {
        fallback.into()
    } else {
        alt.replace(['[', ']'], "")
    }
}

/// True when `url` references the same mm-asset as `asset_rel_path`.
pub fn asset_url_matches_rel(url: &str, asset_rel_path: &str) -> bool {
    let u = url.trim();
    u == asset_rel_path || u.ends_with(asset_rel_path)
}

/// Count markdown image refs pointing at `asset_rel_path`.
pub fn count_markdown_images_for_asset(markdown: &str, asset_rel_path: &str) -> usize {
    let mut count = 0;
    let mut i = 0;
    let bytes = markdown.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((end, _, url)) = parse_markdown_image_at(markdown, i) {
                if asset_url_matches_rel(url, asset_rel_path) {
                    count += 1;
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    count
}

fn line_is_asset_image_line(line: &str, asset_rel_path: &str) -> bool {
    let t = line.trim();
    if !t.starts_with("![") {
        return false;
    }
    t.contains(&format!("({asset_rel_path})"))
}

/// Prefer the image immediately after a Figure/Table heading; else keep first occurrence.
fn preferred_asset_image_line(lines: &[&str], image_line_idxs: &[usize]) -> usize {
    let mut best = image_line_idxs[0];
    let mut best_score = 0_i32;
    for &idx in image_line_idxs {
        let mut score = 0_i32;
        for lookback in 1..=3 {
            if idx >= lookback {
                let prev = lines[idx - lookback].trim();
                if is_figure_heading_line(prev) || is_table_heading_line(prev) {
                    score = 2;
                    break;
                }
            }
        }
        if score > best_score {
            best_score = score;
            best = idx;
        }
    }
    best
}

/// Keep at most one `![…](asset_rel_path)` per page body (MV-28 / SPEC-047 dedupe).
///
/// First principle: one asset identity → one viewer image. Pass A may place an
/// image above the caption while inject adds another after the Figure heading.
pub fn dedupe_markdown_asset_images(markdown: &str, asset_rel_path: &str) -> String {
    if asset_rel_path.trim().is_empty() {
        return markdown.to_string();
    }
    let lines: Vec<&str> = markdown.split('\n').collect();
    let image_line_idxs: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line_is_asset_image_line(line, asset_rel_path))
        .map(|(i, _)| i)
        .collect();
    if image_line_idxs.len() <= 1 {
        return markdown.to_string();
    }
    let keep = preferred_asset_image_line(&lines, &image_line_idxs);
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        if image_line_idxs.contains(&i) && i != keep {
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

/// Dedupe each asset path in `asset_rels` (order preserved, idempotent).
pub fn finalize_page_asset_images(body: &str, asset_rels: &[&str]) -> String {
    let mut out = body.to_string();
    for rel in asset_rels {
        if !rel.is_empty() {
            out = dedupe_markdown_asset_images(&out, rel);
        }
    }
    out
}

///
/// First principle: the visible crop belongs next to its caption — not a full-page
/// PNG and not only at the top of the page — so the side-by-side viewer matches the PDF.
///
/// - `-table-` assets bind only to Table headings
/// - `-fig-` / `-chart` assets bind only to Figure headings
/// - other eligible paths bind to either (legacy chart crops)
pub fn inject_figure_local_images(markdown: &str, asset_rel_path: &str) -> String {
    if markdown.trim().is_empty() {
        return markdown.to_string();
    }
    // Page-wide guard: inject only when this asset is not already referenced anywhere.
    if count_markdown_images_for_asset(markdown, asset_rel_path) > 0 {
        return markdown.to_string();
    }
    let name = asset_rel_path.rsplit('/').next().unwrap_or(asset_rel_path);
    let for_table = name.contains("-table-");
    let for_figure = name.contains("-fig-") || name.contains("-chart");
    let for_either = !for_table && !for_figure;

    let lines: Vec<&str> = markdown.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len().saturating_mul(2));
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        out.push(line.to_string());
        let match_figure = is_figure_heading_line(line) && (for_figure || for_either);
        let match_table = is_table_heading_line(line) && (for_table || for_either);
        if match_figure || match_table {
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            let has_nearby_image = j < lines.len()
                && (lines[j].trim_start().starts_with("![")
                    || lines[j].contains("](assets/")
                    || lines[j].trim_start().starts_with("<drawing"));
            if !has_nearby_image {
                let alt = caption_heading_alt(line, if match_table { "Table" } else { "Figure" });
                out.push(String::new());
                out.push(format_inline_asset_image(&alt, asset_rel_path));
            }
        }
        i += 1;
    }
    out.join("\n")
}

/// Insert a `<drawing/>` tag immediately after the first durable markdown image.
///
/// Returns `None` when no durable image is present (caller should use [`format_drawing_block`]).
pub fn insert_drawing_tag_after_first_image(
    markdown: &str,
    item_id: &str,
    asset_rel_path: &str,
    caption: Option<&str>,
) -> Option<String> {
    let bytes = markdown.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((end, _, url)) = parse_markdown_image_at(markdown, i) {
                if is_durable_mm_asset_url(url) {
                    let tag = format_drawing_tag(item_id, asset_rel_path, caption);
                    let mut out = String::with_capacity(markdown.len() + tag.len() + 2);
                    out.push_str(&markdown[..end]);
                    out.push('\n');
                    out.push_str(&tag);
                    out.push_str(&markdown[end..]);
                    return Some(out);
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    None
}

/// Whether markdown already contains a durable `![…](assets/…)` image.
pub fn markdown_has_durable_asset_image(markdown: &str) -> bool {
    let mut i = 0;
    let bytes = markdown.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((end, _, url)) = parse_markdown_image_at(markdown, i) {
                if is_durable_mm_asset_url(url) {
                    return true;
                }
                i = end;
                continue;
            }
        }
        i += 1;
    }
    false
}

/// Strip HTML tags / angle brackets from caption text so `<drawing caption="…"/>`
/// stays a well-formed self-closing tag (and markdown/HTML renderers do not leak
/// fragments like `1 Junpeng Fang…" />` into the viewer).
pub fn sanitize_drawing_caption(caption: &str) -> String {
    let mut out = String::with_capacity(caption.len());
    let mut in_tag = false;
    for ch in caption.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Format a LightRAG-native self-closing drawing tag scannable by `scan_inline_image_refs`.
pub fn format_drawing_tag(item_id: &str, asset_rel_path: &str, caption: Option<&str>) -> String {
    let caption_attr = caption
        .map(sanitize_drawing_caption)
        .filter(|c| !c.is_empty())
        .map(|c| {
            let escaped = c.replace('"', "'");
            format!(r#" caption="{escaped}""#)
        })
        .unwrap_or_default();
    format!(r#"<drawing id="{item_id}" format="png" path="{asset_rel_path}"{caption_attr} />"#)
}

/// Viewer image + VLM drawing tag (MV-28 / markdown viewer).
///
/// Order: visible `![…](…)` first, then `<drawing/>` for analyze scan.
pub fn format_drawing_block(item_id: &str, asset_rel_path: &str, caption: Option<&str>) -> String {
    let alt = caption.unwrap_or("Page image");
    format!(
        "{}\n{}",
        format_inline_asset_image(alt, asset_rel_path),
        format_drawing_tag(item_id, asset_rel_path, caption)
    )
}

/// Build a caption that carries Pass A chart hints for specialize routing (MV-26).
pub fn caption_with_page_context(page_num: usize, page_body: &str, is_chart_crop: bool) -> String {
    let prefix = if is_chart_crop {
        format!("Page {page_num} chart")
    } else {
        format!("Page {page_num}")
    };
    // Strip markdown images so captions don't re-embed `](assets/…)` (viewer/count noise).
    let body_for_caption = strip_markdown_images(page_body);
    let snippet = body_for_caption
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("<!--"))
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
    let snippet: String = snippet.chars().take(CAPTION_BODY_BUDGET).collect();
    let snippet = snippet.trim();
    if snippet.is_empty() {
        prefix
    } else {
        format!("{prefix} | {snippet}")
    }
}

/// Remove `![alt](url)` spans (used for caption text / dedupe).
pub fn strip_markdown_images(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let bytes = markdown.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((end, _, _)) = parse_markdown_image_at(markdown, i) {
                i = end;
                continue;
            }
        }
        let ch = markdown[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Placeholder body when vision returns no markdown for a page (MV-20).
pub const EMPTY_VISION_PAGE_PLACEHOLDER: &str =
    "*[No text extracted for this page; see page image below.]*";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_asset_paths_are_stable() {
        assert_eq!(
            page_num_from_asset_rel_path("assets/page-0003-fig-01.png"),
            Some(3)
        );
        assert_eq!(
            page_num_from_asset_rel_path("assets/page-0012-chart.png"),
            Some(12)
        );
        assert_eq!(
            page_num_from_asset_rel_path("assets/page-0006-table-01.png"),
            Some(6)
        );
        assert_eq!(page_num_from_asset_rel_path("not-a-page.png"), None);
        assert_eq!(page_asset_rel_path(1), "assets/page-0001.png");
        assert_eq!(page_asset_rel_path(42), "assets/page-0042.png");
        assert_eq!(page_chart_crop_rel_path(1), "assets/page-0001-chart.png");
        assert!(is_full_page_asset_rel_path("assets/page-0001.png"));
        assert!(!is_full_page_asset_rel_path("assets/page-0001-fig-01.png"));
        assert!(!is_full_page_asset_rel_path("assets/page-0001-chart.png"));
        assert!(is_drawing_eligible_asset_rel_path(
            "assets/page-0002-fig-03.png"
        ));
        assert!(is_drawing_eligible_asset_rel_path(
            "assets/page-0002-chart.png"
        ));
        assert!(is_drawing_eligible_asset_rel_path(
            "assets/page-0006-table-01.png"
        ));
        assert!(!is_drawing_eligible_asset_rel_path("assets/page-0002.png"));
        assert_eq!(
            page_table_asset_rel_path(6, 1),
            "assets/page-0006-table-01.png"
        );
        assert!(!is_full_page_asset_rel_path(
            "assets/page-0006-table-01.png"
        ));
    }

    #[test]
    fn inject_table_heading_image() {
        let md = "## Table 1: Pass rates\n\nDiscussion.";
        let out = inject_figure_local_images(md, "assets/page-0006-table-01.png");
        assert!(out.contains("![Table 1: Pass rates](assets/page-0006-table-01.png)"));
        let fig_only = inject_figure_local_images(md, "assets/page-0006-fig-01.png");
        assert!(
            !fig_only.contains("![Table"),
            "fig asset must not bind to table heading: {fig_only}"
        );
    }

    #[test]
    fn inject_skips_when_asset_already_referenced_above_caption() {
        let md = "# Overview\n\n![Pipeline diagram](assets/page-0003-fig-01.png)\n\nFigure 1: System architecture.\n";
        let out = inject_figure_local_images(md, "assets/page-0003-fig-01.png");
        assert_eq!(
            out.matches("](assets/page-0003-fig-01.png)").count(),
            1,
            "must not inject second image when Pass A already bound asset above caption"
        );
    }

    #[test]
    fn dedupe_keeps_caption_adjacent_image_over_pass_a_top_placement() {
        let md = "# Overview\n\n![Pipeline](assets/page-0003-fig-01.png)\n\nFigure 1: System architecture.\n\n![Figure 1: System architecture](assets/page-0003-fig-01.png)\n\nCaption body.";
        let out = dedupe_markdown_asset_images(md, "assets/page-0003-fig-01.png");
        assert_eq!(out.matches("](assets/page-0003-fig-01.png)").count(), 1);
        let keep_pos = out
            .find("![Figure 1: System architecture]")
            .expect("caption image kept");
        let drop_pos = out.find("![Pipeline]");
        assert!(drop_pos.is_none(), "remote duplicate removed: {out}");
        assert!(out[keep_pos..].contains("Caption body."));
    }

    #[test]
    fn dedupe_is_idempotent() {
        let md =
            "![A](assets/page-0001-fig-01.png)\n\nFigure 1: X\n\n![B](assets/page-0001-fig-01.png)";
        let once = dedupe_markdown_asset_images(md, "assets/page-0001-fig-01.png");
        let twice = dedupe_markdown_asset_images(&once, "assets/page-0001-fig-01.png");
        assert_eq!(once, twice);
        assert_eq!(once.matches("](assets/page-0001-fig-01.png)").count(), 1);
    }

    #[test]
    fn count_asset_images_matches_url_variants() {
        assert_eq!(
            count_markdown_images_for_asset(
                "![x](assets/page-0001-fig-01.png)",
                "assets/page-0001-fig-01.png"
            ),
            1
        );
        assert_eq!(
            count_markdown_images_for_asset(
                "![x](/api/v1/documents/d/assets/page-0001-fig-01.png)",
                "assets/page-0001-fig-01.png"
            ),
            1
        );
    }

    #[test]
    fn inject_figure_does_not_duplicate_nearby_image() {
        let md = "## Figure 1: Overview\n\n![Figure 1: Overview](assets/page-0001-fig-01.png)\n\nCaption.";
        let out = inject_figure_local_images(md, "assets/page-0001-fig-01.png");
        assert_eq!(out.matches("](assets/page-0001-fig-01.png)").count(), 1);
    }

    #[test]
    fn sanitize_drawing_caption_strips_html() {
        let s = sanitize_drawing_caption("Page 1 | **Yuze** <sup>1</sup> Fang");
        assert!(!s.contains('<'));
        assert!(!s.contains('>'));
        assert!(s.contains("Yuze"));
        assert!(s.contains("Fang"));
    }

    #[test]
    fn drawing_tag_matches_scanner_contract() {
        let tag = format_drawing_tag("im-doc-page-0003", "assets/page-0003.png", Some("Page 3"));
        assert!(tag.contains(r#"id="im-doc-page-0003""#));
        assert!(tag.contains(r#"path="assets/page-0003.png""#));
        assert!(tag.contains(r#"format="png""#));
        assert!(tag.contains(r#"caption="Page 3""#));
    }

    #[test]
    fn drawing_block_emits_viewable_markdown_image() {
        let block = format_drawing_block(
            "im-doc-page-0001",
            "assets/page-0001.png",
            Some("Page 1 chart"),
        );
        assert!(block.contains("![Page 1 chart](assets/page-0001.png)"));
        assert!(block.contains("<drawing "));
        assert!(block.contains(r#"path="assets/page-0001.png""#));
    }

    #[test]
    fn caption_embeds_pass_a_chart_hints() {
        let c = caption_with_page_context(
            2,
            "Revenue chart\n\n| Q | V |\n|---|---|\n| Q4 | 12% |",
            true,
        );
        assert!(c.starts_with("Page 2 chart"));
        assert!(c.contains("12%") || c.contains("chart") || c.contains("Revenue"));
    }

    #[test]
    fn asset_id_strips_path_and_extension() {
        assert_eq!(asset_id_from_rel_path("assets/page-0001.png"), "page-0001");
        assert_eq!(
            asset_id_from_rel_path("assets/page-0002-chart.png"),
            "page-0002-chart"
        );
    }

    #[test]
    fn bind_rewrites_hallucinated_figure_urls() {
        let md = "![Figure 1. Overview of the MMMU dataset](media/fig1.png)\n\nCaption";
        let out = bind_figure_images_to_page_asset(md, "assets/page-0001.png");
        assert!(out.contains("![Figure 1. Overview of the MMMU dataset](assets/page-0001.png)"));
        assert!(!out.contains("media/fig1.png"));
    }

    #[test]
    fn bind_keeps_durable_asset_urls() {
        let md = "![Page](assets/page-0001.png)";
        let out = bind_figure_images_to_page_asset(md, "assets/page-0002.png");
        assert_eq!(out, md);
    }

    #[test]
    fn bind_rewrites_empty_href() {
        let md = "![Figure 1]()";
        let out = bind_figure_images_to_page_asset(md, "assets/page-0001.png");
        assert_eq!(out, "![Figure 1](assets/page-0001.png)");
    }

    #[test]
    fn item_id_slugifies_document_prefix() {
        assert_eq!(
            page_drawing_item_id(2, Some("ABC-123")),
            "im-abc-123-page-0002"
        );
        assert_eq!(
            page_chart_drawing_item_id(2, Some("ABC-123")),
            "im-abc-123-page-0002-chart"
        );
        assert_eq!(page_chart_drawing_item_id(7, None), "im-page-0007-chart");
    }
}
