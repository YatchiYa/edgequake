//! Chart-region perception for specialize (SPEC-047 EQ-047-MV-24 / SPEC-049 P2 / 026).
//!
//! First principles (no flaky chart **proposers**):
//! - **Propose** chart crops from ink residual on pages that lack **table** assets
//!   (W1-crop-expand: may run **alongside** fig assets; still ink-gated) — never from
//!   English keywords.
//! - Pass-A text (`text_suggests_chart`) may only **route** VLM specialize after a
//!   crop exists on disk.
//! - Re-render gated pages at higher pixel budget; trim near-white margins.
//! - Never invent unreadables; only enlarge what is already on the page.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use edgequake_pdf2md::pipeline::render::render_pages;
use edgequake_pdf2md::ConversionConfig;
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba};
use tracing::{debug, info, warn};

use crate::drawing_tags::{
    page_asset_filename, page_chart_crop_filename, page_chart_crop_rel_path,
    page_figure_asset_rel_path, ASSETS_SUBDIR,
};
use crate::embedded_images::WrittenFigureAsset;
use crate::error::PdfConversionError;
use crate::page_assets::PageAssetRenderConfig;
use crate::region_assets::WrittenTableAsset;

/// Higher-fidelity render for chart-candidate pages (MV-24 / MV-25 budgeted).
pub const CHART_CROP_RENDER: PageAssetRenderConfig = PageAssetRenderConfig {
    dpi: 200,
    max_rendered_pixels: 3600,
};

/// Near-white / near-transparent treated as background for ink bbox.
const BG_LUMA_MIN: u8 = 245;
const BG_ALPHA_MAX: u8 = 16;
/// Padding around ink bbox as fraction of crop width/height.
const PAD_FRAC: f32 = 0.02;
/// Skip crop when ink already fills most of the page (no perception gain).
///
/// Raised from 0.08 → 0.25 so margin-only trims of text pages are rejected
/// (those are near-full-page rasters, not figure/chart crops).
const MIN_AREA_GAIN: f32 = 0.25;
/// Even with enough gain vs the hi-res render, reject crops that still cover
/// most of the page — those are full-page dumps, not chart regions.
const MAX_CROP_AREA_FRAC: f32 = 0.55;

/// Pass A / surrounding text suggests a quantitative chart (**specialize router only**).
///
/// SPEC-049/005: must **not** be the sole source of a chart crop proposal. Use
/// [`chart_residual_candidate_pages`] + ink crop for proposal; call this only to
/// prefer chart specialize when a `-chart.png` already exists.
pub fn text_suggests_chart(blob: &str) -> bool {
    let lower = blob.to_ascii_lowercase();
    const PHRASES: &[&str] = &[
        "bar chart",
        "line chart",
        "pie chart",
        "scatter plot",
        "data table",
        "**key values:**",
        "| category / x |",
        "x-axis",
        "y-axis",
    ];
    if PHRASES.iter().any(|h| lower.contains(h)) {
        return true;
    }
    if figure_caption_suggests_quantitative_chart(&lower) {
        return true;
    }
    // Caption-style: "Figure N … (%)" / axis percent dumps — not bare prose "%".
    if lower.contains("figure") && lower.contains('%') {
        return true;
    }
    if lower.contains("(%)") {
        return true;
    }
    // Word-ish tokens (avoid "geographic"/"photograph" matching "graph").
    const WORDS: &[&str] = &[
        "chart",
        "graph",
        "plot",
        "histogram",
        "scatter",
        "boxplot",
        "heatmap",
    ];
    WORDS.iter().any(|w| contains_ascii_word(&lower, w))
}

/// Figure caption + quantitative evaluation language → chart specialize (Pass B router).
///
/// Catches research-paper layouts like "Figure 1 … model performance … dimensions … tokens"
/// where the VLM may classify a multi-panel line-chart grid as Illustration.
fn figure_caption_suggests_quantitative_chart(lower: &str) -> bool {
    if !contains_ascii_word(lower, "figure") && !lower.contains("fig.") {
        return false;
    }
    const QUANT: &[&str] = &[
        "score",
        "token",
        "plot",
        "chart",
        "graph",
        "axis",
        "series",
        "subplot",
        "performance",
        "dimension",
        "data composition",
        "data point",
        "corpus",
        "ablate",
        "evaluate",
        "tokens(b)",
        "tokens (b)",
    ];
    QUANT.iter().any(|q| {
        if q.contains('(') {
            lower.contains(q)
        } else {
            contains_ascii_word(lower, q)
        }
    })
}

fn contains_ascii_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle = word.as_bytes();
    if needle.is_empty() || bytes.len() < needle.len() {
        return false;
    }
    for i in 0..=(bytes.len() - needle.len()) {
        if &bytes[i..i + needle.len()] != needle {
            continue;
        }
        let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        let after = i + needle.len();
        let after_ok = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Page markdown (Pass A body) is a chart specialize candidate — not a crop proposer.
pub fn page_markdown_suggests_chart(markdown: &str) -> bool {
    text_suggests_chart(markdown)
}

/// Pages eligible for chart **residual** crops (SPEC-047 / 026 W1-crop-expand).
///
/// Policy:
/// - **Always** include pages with neither fig nor table (classic residual).
/// - **Also** include up to [`MAX_ALONGSIDE_FIG_RESIDUAL`] pages that already have
///   figure assets (no table) — fig presence previously blocked residual while Chart
///   digits never entered markdown. Cap bounds hi-res re-render cost on long PDFs
///   (chart-8 includes 117-page docs).
/// - **Skip** pages that already have table assets (table specialize owns that channel).
///
/// Ink area gates in [`crop_png_to_ink_bbox`] / page-PNG prefilter still reject
/// near-full-page junk (SPEC-049 Lever C). Proposal authority remains ink geometry —
/// not Pass-A English (SPEC-049 P2).
pub const MAX_ALONGSIDE_FIG_RESIDUAL: usize = 12;

pub fn chart_residual_candidate_pages(
    page_nums: &[usize],
    figures_by_page: &HashMap<usize, Vec<WrittenFigureAsset>>,
    tables_by_page: &HashMap<usize, Vec<WrittenTableAsset>>,
) -> Vec<usize> {
    let mut bare: Vec<usize> = Vec::new();
    let mut alongside: Vec<usize> = Vec::new();
    for &p in page_nums {
        if tables_by_page.get(&p).is_some_and(|v| !v.is_empty()) {
            continue;
        }
        let has_fig = figures_by_page.get(&p).is_some_and(|v| !v.is_empty());
        if has_fig {
            alongside.push(p);
        } else {
            bare.push(p);
        }
    }
    bare.sort_unstable();
    bare.dedup();
    alongside.sort_unstable();
    alongside.dedup();
    // Prefer earlier pages for alongside (title/KPI charts often front-loaded).
    if alongside.len() > MAX_ALONGSIDE_FIG_RESIDUAL {
        alongside.truncate(MAX_ALONGSIDE_FIG_RESIDUAL);
    }
    let mut out = bare;
    out.extend(alongside);
    out.sort_unstable();
    out.dedup();
    out
}

/// Pages that qualify for residual **because** a fig is present (alongside-fig expansion).
///
/// Diagnostic / telemetry only — candidates still pass through ink gates before write.
pub fn chart_residual_alongside_fig_pages(
    page_nums: &[usize],
    figures_by_page: &HashMap<usize, Vec<WrittenFigureAsset>>,
    tables_by_page: &HashMap<usize, Vec<WrittenTableAsset>>,
) -> Vec<usize> {
    chart_residual_candidate_pages(page_nums, figures_by_page, tables_by_page)
        .into_iter()
        .filter(|p| figures_by_page.get(p).is_some_and(|v| !v.is_empty()))
        .collect()
}

/// SPEC-047 / 026 W1-crop-telemetry (+ W1-crop-expand accounting).
///
/// Pure report for ingest metadata / HTML comment — does not change crop policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CropCoverageReport {
    pub total_pages: usize,
    pub pages_with_fig: usize,
    pub pages_with_table: usize,
    /// Pages eligible for residual propose (no table; fig OK after W1-crop-expand).
    pub residual_candidates: usize,
    /// Subset of candidates that already have a fig asset (alongside-fig expansion).
    pub residual_alongside_fig: usize,
    /// Pages skipped because a table asset exists (residual still blocked on tables).
    ///
    /// Field name kept for HTML comment compatibility; after W1-crop-expand this is
    /// table-only skips (fig pages are candidates).
    pub residual_skipped_due_to_fig_or_table: usize,
    /// After ink prefilter (optional; 0 when not computed).
    pub residual_after_ink_filter: usize,
    /// Successfully written chart crop files (optional; 0 when not computed).
    pub residual_crops_written: usize,
}

impl CropCoverageReport {
    /// Build coverage from page lists + asset maps (SRP: pure accounting).
    pub fn from_pages(
        page_nums: &[usize],
        figures_by_page: &HashMap<usize, Vec<WrittenFigureAsset>>,
        tables_by_page: &HashMap<usize, Vec<WrittenTableAsset>>,
    ) -> Self {
        let total_pages = page_nums.len();
        let pages_with_fig = page_nums
            .iter()
            .filter(|p| figures_by_page.get(p).is_some_and(|v| !v.is_empty()))
            .count();
        let pages_with_table = page_nums
            .iter()
            .filter(|p| tables_by_page.get(p).is_some_and(|v| !v.is_empty()))
            .count();
        let candidates = chart_residual_candidate_pages(page_nums, figures_by_page, tables_by_page);
        let residual_candidates = candidates.len();
        let residual_alongside_fig =
            chart_residual_alongside_fig_pages(page_nums, figures_by_page, tables_by_page).len();
        // After expand: only table pages are hard-skipped from residual propose.
        let residual_skipped_due_to_fig_or_table = total_pages.saturating_sub(residual_candidates);
        Self {
            total_pages,
            pages_with_fig,
            pages_with_table,
            residual_candidates,
            residual_alongside_fig,
            residual_skipped_due_to_fig_or_table,
            residual_after_ink_filter: 0,
            residual_crops_written: 0,
        }
    }

    pub fn with_ink_filter_count(mut self, n: usize) -> Self {
        self.residual_after_ink_filter = n;
        self
    }

    pub fn with_crops_written(mut self, n: usize) -> Self {
        self.residual_crops_written = n;
        self
    }

    /// Machine-readable HTML comment for markdown (bench/fidelity can parse).
    pub fn to_html_comment(&self) -> String {
        format!(
            "<!-- edgequake-crop-coverage: total_pages={} pages_with_fig={} pages_with_table={} residual_candidates={} residual_alongside_fig={} residual_skipped_due_to_fig_or_table={} residual_after_ink_filter={} residual_crops_written={} -->",
            self.total_pages,
            self.pages_with_fig,
            self.pages_with_table,
            self.residual_candidates,
            self.residual_alongside_fig,
            self.residual_skipped_due_to_fig_or_table,
            self.residual_after_ink_filter,
            self.residual_crops_written
        )
    }
}

/// True when an on-disk page PNG has a lawful ink residual crop (area gates).
///
/// Used to skip hi-res chart renders for text-heavy pages where the page raster
/// cannot shrink under `MAX_CROP_AREA_FRAC`.
pub fn page_png_has_ink_residual(page_png: &Path) -> bool {
    let Ok(bytes) = std::fs::read(page_png) else {
        return false;
    };
    crop_png_to_ink_bbox(&bytes).is_some()
}

/// Filter residual candidates: keep pages whose viewer PNG passes ink crop, or
/// pages with no PNG yet (hi-res path may still succeed).
pub fn filter_chart_pages_by_page_png_ink(assets_root: &Path, candidates: &[usize]) -> Vec<usize> {
    candidates
        .iter()
        .copied()
        .filter(|page| {
            let rel = page_asset_filename(*page);
            let path = assets_root.join(ASSETS_SUBDIR).join(&rel);
            if !path.is_file() {
                return true;
            }
            page_png_has_ink_residual(&path)
        })
        .collect()
}

/// When alongside-fig residual ink is empty, the chart **is** the embedded fig.
///
/// First principle (026 W1-fig-as-chart): layout/OCR residual after a full-bleed
/// figure bbox is empty — promoting `page-NNNN-fig-01.png` → `page-NNNN-chart.png`
/// lets W1-coexist emit a chart drawing so specialize can still land numeric text.
///
/// Skips pages that already have a residual chart crop in `already_written`.
/// Copies at most [`MAX_ALONGSIDE_FIG_RESIDUAL`] pages (same cost bound as expand).
pub fn promote_fig_as_chart_when_ink_empty(
    assets_root: &Path,
    alongside_pages: &[usize],
    already_written: &HashMap<usize, String>,
) -> HashMap<usize, String> {
    let mut out = HashMap::new();
    for &page in alongside_pages {
        if out.len() >= MAX_ALONGSIDE_FIG_RESIDUAL {
            break;
        }
        if already_written.contains_key(&page) {
            continue;
        }
        let fig_rel = page_figure_asset_rel_path(page, 1);
        let chart_rel = page_chart_crop_rel_path(page);
        let fig_path = assets_root.join(&fig_rel);
        let chart_path = assets_root.join(&chart_rel);
        if !fig_path.is_file() {
            continue;
        }
        if chart_path.is_file() {
            // Chart already on disk (prior residual) — register path only.
            out.insert(page, chart_rel);
            continue;
        }
        if let Some(parent) = chart_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::copy(&fig_path, &chart_path) {
            Ok(_) => {
                debug!(
                    page_num = page,
                    from = %fig_rel,
                    to = %chart_rel,
                    "W1-fig-as-chart: promoted fig to chart crop (ink residual empty)"
                );
                out.insert(page, chart_rel);
            }
            Err(e) => {
                warn!(
                    page_num = page,
                    error = %e,
                    "W1-fig-as-chart: failed to copy fig to chart crop"
                );
            }
        }
    }
    out
}

/// Axis-aligned ink bounding box (x0, y0, x1, y1) exclusive of x1/y1; `None` if empty.
pub fn ink_content_bbox(img: &DynamicImage) -> Option<(u32, u32, u32, u32)> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        return None;
    }

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut any = false;

    for y in 0..h {
        for x in 0..w {
            let p = rgba.get_pixel(x, y);
            if is_background(*p) {
                continue;
            }
            any = true;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    if !any {
        return None;
    }
    Some((
        min_x,
        min_y,
        max_x.saturating_add(1),
        max_y.saturating_add(1),
    ))
}

fn is_background(p: Rgba<u8>) -> bool {
    let [r, g, b, a] = p.0;
    if a <= BG_ALPHA_MAX {
        return true;
    }
    r >= BG_LUMA_MIN && g >= BG_LUMA_MIN && b >= BG_LUMA_MIN
}

fn apply_pad(bbox: (u32, u32, u32, u32), w: u32, h: u32) -> (u32, u32, u32, u32) {
    let (x0, y0, x1, y1) = bbox;
    let bw = x1.saturating_sub(x0).max(1);
    let bh = y1.saturating_sub(y0).max(1);
    let pad_x = ((bw as f32) * PAD_FRAC).ceil() as u32;
    let pad_y = ((bh as f32) * PAD_FRAC).ceil() as u32;
    (
        x0.saturating_sub(pad_x),
        y0.saturating_sub(pad_y),
        (x1 + pad_x).min(w),
        (y1 + pad_y).min(h),
    )
}

/// Crop PNG bytes to ink content bbox when the crop meaningfully reduces empty margin.
///
/// Returns `None` when crop would not improve perception (full-bleed ink or empty).
pub fn crop_png_to_ink_bbox(png_bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(png_bytes).ok()?;
    let (w, h) = img.dimensions();
    let full_area = (w as f64) * (h as f64);
    if full_area <= 0.0 {
        return None;
    }
    let bbox = ink_content_bbox(&img)?;
    let (x0, y0, x1, y1) = apply_pad(bbox, w, h);
    let cw = x1.saturating_sub(x0);
    let ch = y1.saturating_sub(y0);
    if cw < 8 || ch < 8 {
        return None;
    }
    let crop_area = (cw as f64) * (ch as f64);
    let gain = 1.0 - (crop_area / full_area);
    if gain < MIN_AREA_GAIN as f64 {
        return None;
    }
    // Absolute size gate: a "chart" that still covers most of the page is not a crop.
    if (crop_area / full_area) > MAX_CROP_AREA_FRAC as f64 {
        return None;
    }
    let cropped = img.crop_imm(x0, y0, cw, ch);
    encode_png(&cropped)
}

/// Encode image as PNG bytes.
pub fn encode_png(img: &DynamicImage) -> Option<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png).ok()?;
    Some(buf.into_inner())
}

/// Prefer ink-cropped PNG for chart specialize when crop improves framing.
///
/// Fail-open: returns original bytes when crop is unavailable or not beneficial.
pub fn maybe_chart_specialize_bytes(png_bytes: &[u8]) -> Vec<u8> {
    match crop_png_to_ink_bbox(png_bytes) {
        Some(cropped) => {
            debug!(
                original_len = png_bytes.len(),
                cropped_len = cropped.len(),
                "MV-24 specialize ink-crop applied"
            );
            cropped
        }
        None => png_bytes.to_vec(),
    }
}

/// Re-render chart-candidate pages at higher fidelity, ink-crop, write `page-NNNN-chart.png`.
///
/// Returns map of 1-indexed page → relative asset path (`assets/page-0001-chart.png`).
pub async fn write_chart_crop_assets(
    pdf_bytes: &[u8],
    assets_root: &Path,
    chart_pages: &[usize],
    render: PageAssetRenderConfig,
) -> Result<HashMap<usize, String>, PdfConversionError> {
    if chart_pages.is_empty() {
        return Ok(HashMap::new());
    }

    let assets_dir = assets_root.join(ASSETS_SUBDIR);
    std::fs::create_dir_all(&assets_dir).map_err(|e| {
        PdfConversionError::Backend(format!("failed to create assets dir {assets_dir:?}: {e}"))
    })?;

    let temp_pdf = tempfile::NamedTempFile::new().map_err(|e| {
        PdfConversionError::Backend(format!("failed to create temp pdf for chart crop: {e}"))
    })?;
    let temp_path: PathBuf = temp_pdf.path().to_path_buf();
    std::fs::write(&temp_path, pdf_bytes).map_err(|e| {
        PdfConversionError::Backend(format!("failed to write temp pdf {temp_path:?}: {e}"))
    })?;

    let mut unique: Vec<usize> = chart_pages.to_vec();
    unique.sort_unstable();
    unique.dedup();
    let indices: Vec<usize> = unique.iter().map(|n| n.saturating_sub(1)).collect();

    let render_config = ConversionConfig::builder()
        .dpi(render.dpi)
        .max_rendered_pixels(render.max_rendered_pixels)
        .build()
        .map_err(|e| PdfConversionError::Backend(e.to_string()))?;

    let rendered = render_pages(&temp_path, &render_config, &indices)
        .await
        .map_err(|e| PdfConversionError::Backend(format!("chart crop render failed: {e}")))?;

    let mut written = HashMap::new();
    for (idx0, image) in rendered {
        let page_num = idx0 + 1;
        let filename = page_chart_crop_filename(page_num);
        let full_path = assets_dir.join(&filename);

        let png = match encode_png(&image) {
            Some(b) => b,
            None => {
                warn!(page_num, "Failed to encode hi-res page for chart crop");
                continue;
            }
        };
        // First principle: never promote a full-page raster to a "chart" drawing.
        // If ink-crop cannot meaningfully shrink the frame, skip this page.
        let Some(final_bytes) = crop_png_to_ink_bbox(&png) else {
            debug!(
                page_num,
                "Skipping chart crop: ink bbox is near full-page (not a figure/chart crop)"
            );
            continue;
        };

        if let Err(e) = std::fs::write(&full_path, &final_bytes) {
            warn!(
                page_num,
                path = %full_path.display(),
                error = %e,
                "Failed to write chart crop PNG; skipping (no full-page fallback)"
            );
            continue;
        }
        let rel = format!("{ASSETS_SUBDIR}/{filename}");
        debug!(page_num, path = %full_path.display(), "Wrote MV-24 chart crop asset");
        written.insert(page_num, rel);
    }

    if !written.is_empty() {
        info!(
            pages = written.len(),
            dpi = render.dpi,
            max_pixels = render.max_rendered_pixels,
            "MV-24 chart crop assets written"
        );
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, RgbaImage};

    fn white_with_black_rect() -> DynamicImage {
        let mut img: RgbaImage = ImageBuffer::from_pixel(200, 100, Rgba([255, 255, 255, 255]));
        for y in 20..80 {
            for x in 40..160 {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn chart_residual_allows_fig_pages_skips_table_pages() {
        let mut figs = HashMap::new();
        figs.insert(
            1,
            vec![WrittenFigureAsset {
                page_num: 1,
                index: 1,
                rel_path: "assets/page-0001-fig-01.png".into(),
                width: 10,
                height: 10,
                bbox: None,
            }],
        );
        let mut tables = HashMap::new();
        tables.insert(
            2,
            vec![WrittenTableAsset {
                page_num: 2,
                index: 1,
                rel_path: "assets/page-0002-table-01.png".into(),
                width: 10,
                height: 10,
                label: "Table 1".into(),
                bbox: None,
            }],
        );
        // W1-crop-expand: fig page 1 is a candidate; table page 2 is not.
        let pages = chart_residual_candidate_pages(&[1, 2, 3], &figs, &tables);
        assert_eq!(pages, vec![1, 3]);
        assert_eq!(
            chart_residual_alongside_fig_pages(&[1, 2, 3], &figs, &tables),
            vec![1]
        );
    }

    #[test]
    fn chart_residual_alongside_fig_is_capped() {
        let mut figs = HashMap::new();
        let pages: Vec<usize> = (1..=40).collect();
        for p in &pages {
            figs.insert(
                *p,
                vec![WrittenFigureAsset {
                    page_num: *p,
                    index: 1,
                    rel_path: format!("assets/page-{p:04}-fig-01.png"),
                    width: 10,
                    height: 10,
                    bbox: None,
                }],
            );
        }
        let tables = HashMap::new();
        let cand = chart_residual_candidate_pages(&pages, &figs, &tables);
        let along = chart_residual_alongside_fig_pages(&pages, &figs, &tables);
        assert_eq!(along.len(), MAX_ALONGSIDE_FIG_RESIDUAL);
        assert_eq!(cand.len(), MAX_ALONGSIDE_FIG_RESIDUAL);
        // earliest pages preferred
        assert_eq!(along, (1..=MAX_ALONGSIDE_FIG_RESIDUAL).collect::<Vec<_>>());
    }

    #[test]
    fn promote_fig_as_chart_copies_when_ink_empty() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        std::fs::write(assets.join("page-0005-fig-01.png"), b"\x89PNG-fig").unwrap();
        let alongside = vec![5usize];
        let already = HashMap::new();
        let promoted = promote_fig_as_chart_when_ink_empty(dir.path(), &alongside, &already);
        assert_eq!(
            promoted.get(&5).map(String::as_str),
            Some("assets/page-0005-chart.png")
        );
        let chart = assets.join("page-0005-chart.png");
        assert!(chart.is_file());
        assert_eq!(std::fs::read(&chart).unwrap(), b"\x89PNG-fig");
    }

    #[test]
    fn promote_fig_as_chart_skips_already_written_residual() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        std::fs::write(assets.join("page-0003-fig-01.png"), b"\x89PNG-fig").unwrap();
        std::fs::write(assets.join("page-0003-chart.png"), b"\x89PNG-residual").unwrap();
        let mut already = HashMap::new();
        already.insert(3usize, "assets/page-0003-chart.png".into());
        let promoted = promote_fig_as_chart_when_ink_empty(dir.path(), &[3usize], &already);
        assert!(promoted.is_empty());
        // residual bytes untouched
        assert_eq!(
            std::fs::read(assets.join("page-0003-chart.png")).unwrap(),
            b"\x89PNG-residual"
        );
    }

    #[test]
    fn promote_fig_as_chart_skips_missing_fig_and_respects_cap() {
        let dir = tempfile::tempdir().unwrap();
        let assets = dir.path().join("assets");
        std::fs::create_dir_all(&assets).unwrap();
        // pages 1..cap+2: only even pages have fig-01 (cap+1 eligible)
        let alongside: Vec<usize> = (1..=MAX_ALONGSIDE_FIG_RESIDUAL + 2).collect();
        for p in &alongside {
            if p % 2 == 0 {
                std::fs::write(
                    assets.join(format!("page-{p:04}-fig-01.png")),
                    format!("fig-{p}").as_bytes(),
                )
                .unwrap();
            }
        }
        let promoted = promote_fig_as_chart_when_ink_empty(dir.path(), &alongside, &HashMap::new());
        assert!(promoted.len() <= MAX_ALONGSIDE_FIG_RESIDUAL);
        // Odd pages skipped (no fig); even pages promoted until cap.
        assert!(!promoted.contains_key(&1));
        assert!(promoted.contains_key(&2));
        for (p, rel) in &promoted {
            assert_eq!(rel, &format!("assets/page-{p:04}-chart.png"));
            assert!(dir.path().join(rel).is_file());
        }
    }

    #[test]
    fn crop_coverage_report_accounts_for_table_skips_and_alongside_fig() {
        let mut figs = HashMap::new();
        figs.insert(
            1,
            vec![WrittenFigureAsset {
                page_num: 1,
                index: 1,
                rel_path: "assets/page-0001-fig-01.png".into(),
                width: 10,
                height: 10,
                bbox: None,
            }],
        );
        let tables = HashMap::new();
        let report = CropCoverageReport::from_pages(&[1, 2], &figs, &tables)
            .with_ink_filter_count(2)
            .with_crops_written(1);
        assert_eq!(report.total_pages, 2);
        assert_eq!(report.pages_with_fig, 1);
        // Both pages are candidates (no tables); page 1 is alongside-fig.
        assert_eq!(report.residual_candidates, 2);
        assert_eq!(report.residual_alongside_fig, 1);
        assert_eq!(report.residual_skipped_due_to_fig_or_table, 0);
        assert_eq!(report.residual_after_ink_filter, 2);
        assert_eq!(report.residual_crops_written, 1);
        let comment = report.to_html_comment();
        assert!(comment.contains("edgequake-crop-coverage:"));
        assert!(comment.contains("residual_alongside_fig=1"));
        assert!(comment.contains("residual_skipped_due_to_fig_or_table=0"));
    }

    #[test]
    fn text_suggests_chart_from_pass_a_signals() {
        assert!(text_suggests_chart(
            "Quarterly revenue chart with 12% growth"
        ));
        assert!(text_suggests_chart("| Category / X | Value |\n|---|---|"));
        assert!(text_suggests_chart("**Key values:**\n- A: 1"));
        assert!(!text_suggests_chart("Photo of a beach sunset"));
        // Prose with a bare percentage must NOT become a chart candidate.
        assert!(!text_suggests_chart(
            "ARTS outperforms leading algorithms, with over 15.3% relative improvement"
        ));
        // Figure captions with (%) remain chart signals (specialize routing).
        assert!(text_suggests_chart("Figure 3. Revenue by quarter (%)"));
        assert!(text_suggests_chart(
            "Figure 1. The impact of three data compositions on model performance across capability dimensions. 10T-token corpus."
        ));
        assert!(!text_suggests_chart(
            "geographic distribution of the photograph dataset"
        ));
    }

    #[test]
    fn ink_bbox_finds_black_rect() {
        let img = white_with_black_rect();
        let (x0, y0, x1, y1) = ink_content_bbox(&img).expect("bbox");
        assert!(x0 <= 40 && y0 <= 20);
        assert!(x1 >= 160 && y1 >= 80);
    }

    #[test]
    fn crop_png_reduces_margins() {
        let img = white_with_black_rect();
        let png = encode_png(&img).expect("png");
        let cropped = crop_png_to_ink_bbox(&png).expect("crop");
        let out = image::load_from_memory(&cropped).expect("load crop");
        let (w, h) = out.dimensions();
        assert!(w < 200, "expected width crop, got {w}");
        assert!(h < 100, "expected height crop, got {h}");
    }

    #[test]
    fn full_bleed_ink_skips_crop() {
        let img: RgbaImage = ImageBuffer::from_pixel(50, 50, Rgba([10, 10, 10, 255]));
        let png = encode_png(&DynamicImage::ImageRgba8(img)).unwrap();
        assert!(crop_png_to_ink_bbox(&png).is_none());
    }

    #[test]
    fn near_full_crop_rejected_even_with_margin_gain() {
        // 80% ink fill → gain 20%? Wait we need gain>=25% and area>55%.
        // Ink from (15,15)-(90,90) ≈ 75×75 / 100×100 = 0.5625, gain ≈ 0.44.
        let mut img: RgbaImage = ImageBuffer::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        for y in 15..90 {
            for x in 15..90 {
                img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
        let png = encode_png(&DynamicImage::ImageRgba8(img)).unwrap();
        assert!(
            crop_png_to_ink_bbox(&png).is_none(),
            "crop covering >55% of page must be rejected as near-full"
        );
    }

    #[test]
    fn empty_png_skips_crop() {
        assert!(crop_png_to_ink_bbox(&[]).is_none());
    }

    #[test]
    fn maybe_specialize_fail_open_on_invalid() {
        let out = maybe_chart_specialize_bytes(b"not-a-png");
        assert_eq!(out, b"not-a-png");
    }
}
