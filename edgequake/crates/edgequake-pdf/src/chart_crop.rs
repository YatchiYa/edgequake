//! Chart-region perception for specialize (SPEC-047 EQ-047-MV-24 / SPEC-049 P2).
//!
//! First principles (no flaky chart **proposers**):
//! - **Propose** chart crops from ink residual on pages that already lack fig/table
//!   assets (deterministic geometry + area gates) — never from English keywords.
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

use crate::drawing_tags::{page_asset_filename, page_chart_crop_filename, ASSETS_SUBDIR};
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

/// Pages eligible for chart **residual** crops: no fig and no table assets yet.
///
/// Proposal authority is ink geometry inside [`write_chart_crop_assets`] / page-PNG
/// prefilter — not Pass-A English (SPEC-049 P2).
pub fn chart_residual_candidate_pages(
    page_nums: &[usize],
    figures_by_page: &HashMap<usize, Vec<WrittenFigureAsset>>,
    tables_by_page: &HashMap<usize, Vec<WrittenTableAsset>>,
) -> Vec<usize> {
    let mut out: Vec<usize> = page_nums
        .iter()
        .copied()
        .filter(|p| {
            let no_fig = figures_by_page.get(p).is_none_or(|v| v.is_empty());
            let no_table = tables_by_page.get(p).is_none_or(|v| v.is_empty());
            no_fig && no_table
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// True when an on-disk page PNG has a lawful ink residual crop (area gates).
///
/// Used to skip hi-res chart renders for text-heavy pages where the page raster
/// cannot shrink under [`MAX_CROP_AREA_FRAC`].
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
    fn chart_residual_skips_pages_with_fig_or_table() {
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
            }],
        );
        let pages = chart_residual_candidate_pages(&[1, 2, 3], &figs, &tables);
        assert_eq!(pages, vec![3]);
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
