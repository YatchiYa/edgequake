//! Two-pass VLM figure filter for SPEC-049 (SOLID / DRY).
//!
//! ## Design
//!
//! The geometry layer (L0/L1 pdfium) is a **conservative proposer**:
//! it intentionally keeps more crops than needed so it never misses a real
//! figure.  Geometry alone cannot distinguish a vector bar-chart from a
//! decorated text-box (both are rectangular path fills).
//!
//! This module is the **semantic oracle**: it runs two cheap VLM passes on
//! each proposed PNG to decide whether it is a real figure and, if so, what
//! specialised description to emit:
//!
//! ```text
//! Geometry proposes PNGs
//!        │
//!        ▼
//!  Pass 1 — FILTER  (is_figure: bool, kind: FigureKind)
//!        │           discard: logo, text_block, icon, decorative_rule
//!        ▼ (kept only)
//!  Pass 2 — SPECIALIZE  (kind-aware structured prompt → Markdown description)
//!        │               chart → data-table, diagram → component list, …
//!        ▼
//!  FigureFilterResult  (written to figure_filter_manifest.json)
//!        │
//!        ▼
//!  prune figure_map + chart crops; inject Pass-2 Markdown into the page body
//! ```
//!
//! ## SOLID
//!
//! - **S**: Filter only — does not write PNG files (that is `region_assets`).
//! - **O**: Default-on when a vision LLM is attached; `EDGEQUAKE_FIGURE_FILTER=0` forces off.
//! - **L**: Any `LLMProvider` implementation works identically.
//! - **I**: Depends on `LLMProvider` trait only, not on the full VisionConversionConfig.
//! - **D**: `FigureFilter` is injected with a provider; does not create one.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use base64::Engine as _;
use edgequake_llm::{ChatMessage, CompletionOptions, ImageData, LLMProvider};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::embedded_images::WrittenFigureAsset;

use crate::error::PdfConversionError;
use crate::vision_prompts::{
    figure_filter_pass1_prompt, figure_filter_pass2_prompt, FIGURE_FILTER_PASS1_SYSTEM,
    FIGURE_FILTER_PASS2_SYSTEM,
};

// ── Public types ──────────────────────────────────────────────────────────────

/// Semantic classification of one proposed figure crop (Pass 1 output).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FigureKind {
    BarChart,
    LineChart,
    ScatterPlot,
    Heatmap,
    Histogram,
    PieChart,
    RadarChart,
    ArchitectureDiagram,
    Flowchart,
    Diagram,
    Illustration,
    Photograph,
    SystemDemo,
    TableVisual,
    /// Not a figure — discard.
    Logo,
    IconLogo,
    TextBlock,
    DecorativeRule,
    Empty,
    /// SPEC-128 discard taxonomy.
    Stamp,
    Signature,
    ScanArtefact,
    Watermark,
    Other,
}

impl FigureKind {
    /// True when this kind carries unique visual signal worth keeping.
    pub fn is_figure(&self) -> bool {
        !matches!(
            self,
            Self::Logo
                | Self::IconLogo
                | Self::TextBlock
                | Self::DecorativeRule
                | Self::Empty
                | Self::Stamp
                | Self::Signature
                | Self::ScanArtefact
                | Self::Watermark
        )
    }

    /// Parse from a raw string returned by the VLM.
    fn from_str_fuzzy(s: &str) -> Self {
        match s.to_ascii_lowercase().replace([' ', '-'], "_").as_str() {
            "bar_chart" => Self::BarChart,
            "line_chart" => Self::LineChart,
            "scatter_plot" => Self::ScatterPlot,
            "heatmap" => Self::Heatmap,
            "histogram" => Self::Histogram,
            "pie_chart" => Self::PieChart,
            "radar_chart" => Self::RadarChart,
            "architecture_diagram" | "architecture" => Self::ArchitectureDiagram,
            "flowchart" | "flow_chart" => Self::Flowchart,
            "diagram" => Self::Diagram,
            "illustration" => Self::Illustration,
            "photograph" | "photo" => Self::Photograph,
            "system_demo" => Self::SystemDemo,
            "table_visual" | "table" => Self::TableVisual,
            "logo" => Self::Logo,
            "icon_logo" | "icon" => Self::IconLogo,
            "text_block" | "text" => Self::TextBlock,
            "decorative_rule" | "decorative" => Self::DecorativeRule,
            "empty" => Self::Empty,
            "stamp" => Self::Stamp,
            "signature" => Self::Signature,
            "scan_artefact" | "scan_artifact" => Self::ScanArtefact,
            "watermark" => Self::Watermark,
            _ => Self::Other,
        }
    }
}

/// Input reference to one candidate figure PNG.
#[derive(Debug, Clone)]
pub struct FigureCandidate {
    /// Relative path (e.g. `assets/page-0001-fig-01.png`).
    pub rel_path: String,
    /// Absolute path to the PNG on disk.
    pub full_path: std::path::PathBuf,
    /// 1-indexed page number.
    pub page_num: usize,
    /// Caption label from geometry (`Figure 1`, `Figure`, …).
    pub label: String,
    /// Pixel area for VLM budget (lowest-area dropped first). SPEC-128 WP-4.
    pub area_px: u64,
}

/// Result for one candidate after both passes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureFilterResult {
    pub rel_path: String,
    pub page_num: usize,
    pub label: String,
    /// Pass-1 classification.
    pub kind: FigureKind,
    /// True when kind carries visual signal and Pass 2 ran.
    pub is_figure: bool,
    /// Pass-2 description (empty when `is_figure` is false).
    pub description: String,
}

/// Written sidecar file under `{assets_root}/figure_filter_manifest.json`.
pub const FIGURE_FILTER_MANIFEST: &str = "figure_filter_manifest.json";

// ── FigureFilter ─────────────────────────────────────────────────────────────

/// Two-pass VLM figure filter.
///
/// Construct with [`FigureFilter::new`], then call [`FigureFilter::run`].
pub struct FigureFilter {
    provider: Arc<dyn LLMProvider>,
    concurrency: usize,
    max_per_page: usize,
}

/// Default Pass-1 in-flight cap (SPEC-128 WP-4).
pub const DEFAULT_FIGURE_FILTER_CONCURRENCY: usize = 4;
/// Soft cap of VLM calls per page; lowest-area crops dropped first.
pub const DEFAULT_MAX_FIGURE_VLM_PER_PAGE: usize = 12;

/// `EDGEQUAKE_FIGURE_FILTER=0|false|off|no` disables; default **on**.
pub fn figure_filter_env_enabled() -> bool {
    match std::env::var("EDGEQUAKE_FIGURE_FILTER") {
        Ok(v) => {
            let t = v.trim().to_ascii_lowercase();
            !matches!(t.as_str(), "0" | "false" | "off" | "no")
        }
        Err(_) => true,
    }
}

impl FigureFilter {
    /// Create a new filter backed by `provider` (default concurrency/budget).
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self::with_limits(
            provider,
            DEFAULT_FIGURE_FILTER_CONCURRENCY,
            DEFAULT_MAX_FIGURE_VLM_PER_PAGE,
        )
    }

    pub fn with_limits(
        provider: Arc<dyn LLMProvider>,
        concurrency: usize,
        max_per_page: usize,
    ) -> Self {
        Self {
            provider,
            concurrency: concurrency.max(1),
            max_per_page: max_per_page.max(1),
        }
    }

    /// Run both passes on every (budgeted) candidate.  Noise kinds skip Pass 2
    /// but still appear in the result list with `is_figure = false`.
    pub async fn run(
        &self,
        candidates: &[FigureCandidate],
    ) -> Result<Vec<FigureFilterResult>, PdfConversionError> {
        let selected = budget_candidates(candidates, self.max_per_page);
        let conc = self.concurrency;
        let provider = Arc::clone(&self.provider);
        let mut results: Vec<FigureFilterResult> = stream::iter(selected)
            .map(|c| {
                let provider = Arc::clone(&provider);
                async move { process_one(provider, c).await }
            })
            .buffer_unordered(conc)
            .collect()
            .await;
        results.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
        Ok(results)
    }
}

async fn process_one(provider: Arc<dyn LLMProvider>, c: FigureCandidate) -> FigureFilterResult {
    let (kind, is_figure) = match pass1_classify(provider.as_ref(), &c.full_path).await {
        Ok(d) => (d.kind, d.is_figure),
        Err(e) => {
            warn!(rel_path = %c.rel_path, error = %e, "Pass-1 filter failed; keeping crop");
            (FigureKind::Other, true)
        }
    };
    debug!(rel_path = %c.rel_path, kind = ?kind, is_figure, "Pass-1 result");

    if !is_figure {
        return FigureFilterResult {
            rel_path: c.rel_path,
            page_num: c.page_num,
            label: c.label,
            kind,
            is_figure: false,
            description: String::new(),
        };
    }

    let description = match pass2_describe(provider.as_ref(), &c.full_path, &kind).await {
        Ok(d) => d,
        Err(e) => {
            warn!(rel_path = %c.rel_path, error = %e, "Pass-2 specialize failed");
            String::new()
        }
    };
    debug!(rel_path = %c.rel_path, desc_chars = description.len(), "Pass-2 result");

    FigureFilterResult {
        rel_path: c.rel_path,
        page_num: c.page_num,
        label: c.label,
        kind,
        is_figure: true,
        description,
    }
}

pub(crate) fn budget_candidates(
    candidates: &[FigureCandidate],
    max_per_page: usize,
) -> Vec<FigureCandidate> {
    let mut by_page: HashMap<usize, Vec<FigureCandidate>> = HashMap::new();
    for c in candidates {
        by_page.entry(c.page_num).or_default().push(c.clone());
    }
    let mut out = Vec::new();
    for mut list in by_page.into_values() {
        list.sort_by_key(|b| std::cmp::Reverse(b.area_px));
        list.truncate(max_per_page);
        out.extend(list);
    }
    out
}

struct Pass1Decision {
    kind: FigureKind,
    is_figure: bool,
}

async fn pass1_classify(
    provider: &dyn LLMProvider,
    png_path: &Path,
) -> Result<Pass1Decision, PdfConversionError> {
    let image = load_image_data(png_path)?;
    let opts = CompletionOptions {
        max_tokens: Some(80),
        temperature: Some(0.0),
        ..Default::default()
    }
    .with_role_cache("figure-filter", provider);
    let messages = vec![
        ChatMessage::system(FIGURE_FILTER_PASS1_SYSTEM),
        ChatMessage::user_with_images(figure_filter_pass1_prompt(), vec![image]),
    ];
    let response = provider
        .chat(&messages, Some(&opts))
        .await
        .map_err(|e| PdfConversionError::Backend(format!("Pass-1 LLM call failed: {e}")))?;
    parse_pass1_json(response.content.trim())
}

async fn pass2_describe(
    provider: &dyn LLMProvider,
    png_path: &Path,
    kind: &FigureKind,
) -> Result<String, PdfConversionError> {
    let image = load_image_data(png_path)?;
    let opts = CompletionOptions {
        max_tokens: Some(600),
        temperature: Some(0.0),
        ..Default::default()
    }
    .with_role_cache("figure-filter", provider);
    let messages = vec![
        ChatMessage::system(FIGURE_FILTER_PASS2_SYSTEM),
        ChatMessage::user_with_images(figure_filter_pass2_prompt(kind), vec![image]),
    ];
    let response = provider
        .chat(&messages, Some(&opts))
        .await
        .map_err(|e| PdfConversionError::Backend(format!("Pass-2 LLM call failed: {e}")))?;
    Ok(response.content.trim().to_string())
}

/// Rebuild `figure_map` to kept paths only (G-prune). Paths not in `results` are
/// kept (fail-open for crops the filter never saw). Empty pages are dropped.
pub fn prune_figure_map(
    figure_map: HashMap<usize, Vec<WrittenFigureAsset>>,
    results: &[FigureFilterResult],
) -> HashMap<usize, Vec<WrittenFigureAsset>> {
    let seen: HashSet<&str> = results.iter().map(|r| r.rel_path.as_str()).collect();
    let kept: HashSet<&str> = results
        .iter()
        .filter(|r| r.is_figure)
        .map(|r| r.rel_path.as_str())
        .collect();
    let mut out: HashMap<usize, Vec<WrittenFigureAsset>> = HashMap::new();
    for (page, figs) in figure_map {
        let filtered: Vec<WrittenFigureAsset> = figs
            .into_iter()
            .filter(|f| {
                if !seen.contains(f.rel_path.as_str()) {
                    true
                } else {
                    kept.contains(f.rel_path.as_str())
                }
            })
            .collect();
        if !filtered.is_empty() {
            out.insert(page, filtered);
        }
    }
    out
}

/// Delete discarded PNG files under `assets_root`. Best-effort.
pub fn delete_discarded_pngs(assets_root: &Path, results: &[FigureFilterResult]) {
    for r in results.iter().filter(|r| !r.is_figure) {
        let path = assets_root.join(&r.rel_path);
        if path.is_file() {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!(path = %path.display(), error = %e, "failed to delete discarded figure PNG");
            }
        }
    }
}

/// Apply prune + optional PNG delete. Logs kept/discarded by kind.
pub fn apply_filter_to_figure_map(
    figure_map: HashMap<usize, Vec<WrittenFigureAsset>>,
    results: &[FigureFilterResult],
    assets_root: &Path,
    delete_discarded: bool,
) -> HashMap<usize, Vec<WrittenFigureAsset>> {
    let kept_n = results.iter().filter(|r| r.is_figure).count();
    let mut discarded_by_kind: HashMap<String, usize> = HashMap::new();
    for r in results.iter().filter(|r| !r.is_figure) {
        *discarded_by_kind
            .entry(format!("{:?}", r.kind))
            .or_insert(0) += 1;
    }
    info!(
        figure_filter_kept = kept_n,
        discarded = results.len().saturating_sub(kept_n),
        discarded_by_kind = ?discarded_by_kind,
        total = results.len(),
        "SPEC-128 figure filter prune"
    );
    if delete_discarded {
        delete_discarded_pngs(assets_root, results);
    }
    prune_figure_map(figure_map, results)
}

/// Fail-open: on filter `Err`, return the original map unchanged (LAW-128-13).
pub fn apply_filter_result_or_keep(
    figure_map: HashMap<usize, Vec<WrittenFigureAsset>>,
    run: Result<Vec<FigureFilterResult>, PdfConversionError>,
    assets_root: &Path,
    delete_discarded: bool,
) -> HashMap<usize, Vec<WrittenFigureAsset>> {
    match run {
        Ok(results) => {
            apply_filter_to_figure_map(figure_map, &results, assets_root, delete_discarded)
        }
        Err(e) => {
            warn!(error = %e, "figure filter failed; keeping all crops (fail-open)");
            figure_map
        }
    }
}

/// Apply an on-disk `figure_filter_manifest.json` if present (include-from-pdf).
pub fn prune_figure_map_using_manifest(
    figure_map: HashMap<usize, Vec<WrittenFigureAsset>>,
    assets_root: &Path,
    delete_discarded: bool,
) -> HashMap<usize, Vec<WrittenFigureAsset>> {
    let manifest = load_manifest(assets_root);
    if manifest.is_empty() {
        figure_map
    } else {
        apply_filter_to_figure_map(figure_map, &manifest, assets_root, delete_discarded)
    }
}

/// Collect unique figure + chart crops for Pass-1/Pass-2 (LAW-128-13).
pub fn collect_filter_candidates(
    assets_root: &Path,
    figure_map: &HashMap<usize, Vec<WrittenFigureAsset>>,
    chart_paths: &HashMap<usize, String>,
) -> Vec<FigureCandidate> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for fig in figure_map.values().flatten() {
        if !seen.insert(fig.rel_path.clone()) {
            continue;
        }
        out.push(FigureCandidate {
            rel_path: fig.rel_path.clone(),
            full_path: assets_root.join(&fig.rel_path),
            page_num: fig.page_num,
            label: String::new(),
            area_px: u64::from(fig.width)
                .saturating_mul(u64::from(fig.height))
                .max(1),
        });
    }
    for (page, rel) in chart_paths {
        if !seen.insert(rel.clone()) {
            continue;
        }
        let full_path = assets_root.join(rel);
        out.push(FigureCandidate {
            rel_path: rel.clone(),
            full_path: full_path.clone(),
            page_num: *page,
            label: "Chart".into(),
            area_px: png_area_px(&full_path),
        });
    }
    out
}

/// Drop discarded chart crops from the assemble override map (fail-open for unseen paths).
pub fn prune_chart_crop_paths(
    chart_paths: HashMap<usize, String>,
    results: &[FigureFilterResult],
) -> HashMap<usize, String> {
    if results.is_empty() {
        return chart_paths;
    }
    let seen: HashSet<&str> = results.iter().map(|r| r.rel_path.as_str()).collect();
    let kept: HashSet<&str> = results
        .iter()
        .filter(|r| r.is_figure)
        .map(|r| r.rel_path.as_str())
        .collect();
    chart_paths
        .into_iter()
        .filter(|(_, rel)| {
            if !seen.contains(rel.as_str()) {
                true
            } else {
                kept.contains(rel.as_str())
            }
        })
        .collect()
}

/// Rel-paths Pass-1 classified as artefacts.
pub fn discarded_rel_paths(results: &[FigureFilterResult]) -> HashSet<String> {
    results
        .iter()
        .filter(|r| !r.is_figure)
        .map(|r| r.rel_path.clone())
        .collect()
}

/// Rel-paths discarded by an on-disk manifest (empty when the filter never ran).
pub fn discarded_rel_paths_from_manifest(assets_root: &Path) -> HashSet<String> {
    discarded_rel_paths(&load_manifest(assets_root))
}

/// Inject Pass-2 specialised Markdown after each kept crop's image href.
///
/// Artefacts are never injected. Already-injected paths are skipped (idempotent).
pub fn inject_kept_descriptions(markdown: &str, results: &[FigureFilterResult]) -> String {
    let mut out = markdown.to_string();
    for r in results
        .iter()
        .filter(|r| r.is_figure && !r.description.trim().is_empty())
    {
        let marker = format!("<!-- edgequake-figure-vision:{} -->", r.rel_path);
        if out.contains(&marker) {
            continue;
        }
        let needle = format!("]({})", r.rel_path);
        let Some(idx) = out.find(&needle) else {
            continue;
        };
        let after = idx + needle.len();
        let line_end = out[after..]
            .find('\n')
            .map(|n| after + n)
            .unwrap_or(out.len());
        let block = format!("\n\n{}\n{}\n", marker, r.description.trim());
        out.insert_str(line_end, &block);
    }
    out
}

/// Drop markdown image lines that point at discarded artefact crops.
pub fn strip_discarded_asset_lines(markdown: &str, discarded: &HashSet<String>) -> String {
    if discarded.is_empty() {
        return markdown.to_string();
    }
    let mut out = String::with_capacity(markdown.len());
    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim();
        let drop = discarded.iter().any(|rel| {
            trimmed.contains(&format!("]({rel})")) || trimmed.contains(&format!("path=\"{rel}\""))
        });
        if !drop {
            out.push_str(line);
        }
    }
    out
}

// ── Manifest I/O ─────────────────────────────────────────────────────────────

/// Write `figure_filter_manifest.json` under `assets_root`.
pub fn write_manifest(
    assets_root: &Path,
    results: &[FigureFilterResult],
) -> Result<(), PdfConversionError> {
    let path = assets_root.join(FIGURE_FILTER_MANIFEST);
    let json = serde_json::to_string_pretty(results).map_err(|e| {
        PdfConversionError::Backend(format!("serialize figure filter manifest: {e}"))
    })?;
    std::fs::write(&path, json).map_err(|e| {
        PdfConversionError::Backend(format!("write figure filter manifest {path:?}: {e}"))
    })?;
    Ok(())
}

/// Load the manifest if it exists; return empty vec otherwise.
pub fn load_manifest(assets_root: &Path) -> Vec<FigureFilterResult> {
    let path = assets_root.join(FIGURE_FILTER_MANIFEST);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Encode a PNG file as `ImageData` for the LLM provider.
fn load_image_data(png_path: &Path) -> Result<ImageData, PdfConversionError> {
    let bytes = std::fs::read(png_path).map_err(|e| {
        PdfConversionError::Backend(format!("read PNG {}: {e}", png_path.display()))
    })?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(ImageData::new(b64, "image/png"))
}

/// Parse `{"kind": "...", "is_figure": bool}` from Pass-1 VLM response.
/// Tolerates fenced JSON and missing `is_figure` key (derives from kind).
/// Explicit `is_figure: false` discards even when kind is `other`.
fn parse_pass1_json(raw: &str) -> Result<Pass1Decision, PdfConversionError> {
    let cleaned = if let Some(start) = raw.find('{') {
        &raw[start..]
    } else {
        raw
    };
    let cleaned = if let Some(end) = cleaned.rfind('}') {
        &cleaned[..=end]
    } else {
        cleaned
    };

    let v: serde_json::Value = serde_json::from_str(cleaned)
        .map_err(|e| PdfConversionError::Backend(format!("Pass-1 JSON parse: {e} — raw={raw}")))?;

    let kind_str = v.get("kind").and_then(|k| k.as_str()).unwrap_or("other");
    let kind = FigureKind::from_str_fuzzy(kind_str);
    let json_flag = v.get("is_figure").and_then(|x| x.as_bool());
    let is_figure = kind.is_figure() && json_flag.unwrap_or(true);

    Ok(Pass1Decision { kind, is_figure })
}

fn png_area_px(path: &Path) -> u64 {
    let Ok(bytes) = std::fs::read(path) else {
        return 1;
    };
    if bytes.len() < 24 || &bytes[0..4] != b"\x89PNG" {
        return 1;
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as u64;
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as u64;
    w.saturating_mul(h).max(1)
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_classification() {
        assert!(FigureKind::BarChart.is_figure());
        assert!(FigureKind::ArchitectureDiagram.is_figure());
        assert!(FigureKind::SystemDemo.is_figure());
        assert!(!FigureKind::Logo.is_figure());
        assert!(!FigureKind::TextBlock.is_figure());
        assert!(!FigureKind::DecorativeRule.is_figure());
        assert!(!FigureKind::Stamp.is_figure());
        assert!(!FigureKind::Watermark.is_figure());
        assert!(!FigureKind::Signature.is_figure());
        // Other is kept (conservative)
        assert!(FigureKind::Other.is_figure());
    }

    #[test]
    fn prune_figure_map_drops_discarded_only() {
        let mut map = HashMap::new();
        let fig = |rel: &str, page: usize| WrittenFigureAsset {
            page_num: page,
            index: 1,
            rel_path: rel.into(),
            width: 10,
            height: 10,
            bbox: Some((0.0, 0.0, 10.0, 10.0)),
        };
        map.insert(
            1usize,
            vec![
                fig("assets/a.png", 1),
                fig("assets/b.png", 1),
                fig("assets/c.png", 1),
            ],
        );
        map.insert(2usize, vec![fig("assets/d.png", 2), fig("assets/e.png", 2)]);
        let results = vec![
            FigureFilterResult {
                rel_path: "assets/a.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::BarChart,
                is_figure: true,
                description: "ok".into(),
            },
            FigureFilterResult {
                rel_path: "assets/b.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::Logo,
                is_figure: false,
                description: String::new(),
            },
            FigureFilterResult {
                rel_path: "assets/c.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::Diagram,
                is_figure: true,
                description: "ok".into(),
            },
            FigureFilterResult {
                rel_path: "assets/d.png".into(),
                page_num: 2,
                label: String::new(),
                kind: FigureKind::Stamp,
                is_figure: false,
                description: String::new(),
            },
            FigureFilterResult {
                rel_path: "assets/e.png".into(),
                page_num: 2,
                label: String::new(),
                kind: FigureKind::Photograph,
                is_figure: true,
                description: "ok".into(),
            },
        ];
        let pruned = prune_figure_map(map, &results);
        let paths: Vec<String> = pruned
            .values()
            .flatten()
            .map(|f| f.rel_path.clone())
            .collect();
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"assets/a.png".into()));
        assert!(paths.contains(&"assets/c.png".into()));
        assert!(paths.contains(&"assets/e.png".into()));
        assert!(!paths.contains(&"assets/b.png".into()));
        assert!(!paths.contains(&"assets/d.png".into()));
    }

    #[test]
    #[serial_test::serial]
    fn figure_filter_env_default_on() {
        std::env::remove_var("EDGEQUAKE_FIGURE_FILTER");
        assert!(figure_filter_env_enabled());
        std::env::set_var("EDGEQUAKE_FIGURE_FILTER", "0");
        assert!(!figure_filter_env_enabled());
        std::env::remove_var("EDGEQUAKE_FIGURE_FILTER");
    }

    #[test]
    fn parse_pass1_json_ok() {
        let raw = r#"{"kind": "bar_chart", "is_figure": true, "confidence": 0.99}"#;
        let d = parse_pass1_json(raw).unwrap();
        assert_eq!(d.kind, FigureKind::BarChart);
        assert!(d.is_figure);
    }

    #[test]
    fn parse_pass1_json_strips_fences() {
        let raw = "```json\n{\"kind\": \"text_block\", \"is_figure\": false}\n```";
        let d = parse_pass1_json(raw).unwrap();
        assert_eq!(d.kind, FigureKind::TextBlock);
        assert!(!d.is_figure);
    }

    #[test]
    fn parse_pass1_json_unknown_kind() {
        let raw = r#"{"kind": "unknown_thing"}"#;
        let d = parse_pass1_json(raw).unwrap();
        assert_eq!(d.kind, FigureKind::Other);
        assert!(d.is_figure, "missing is_figure on other → fail-open keep");
    }

    #[test]
    fn parse_pass1_other_explicit_false_discards() {
        let d = parse_pass1_json(r#"{"kind":"other","is_figure":false}"#).unwrap();
        assert_eq!(d.kind, FigureKind::Other);
        assert!(!d.is_figure);
    }

    #[test]
    fn parse_pass1_logo_true_still_discards() {
        let d = parse_pass1_json(r#"{"kind":"logo","is_figure":true}"#).unwrap();
        assert_eq!(d.kind, FigureKind::Logo);
        assert!(!d.is_figure);
    }

    #[test]
    fn kind_fuzzy_parsing() {
        assert_eq!(
            FigureKind::from_str_fuzzy("architecture"),
            FigureKind::ArchitectureDiagram
        );
        assert_eq!(
            FigureKind::from_str_fuzzy("flow chart"),
            FigureKind::Flowchart
        );
        assert_eq!(FigureKind::from_str_fuzzy("LOGO"), FigureKind::Logo);
    }

    #[test]
    fn filter_error_fail_open_keeps_all() {
        let mut map = HashMap::new();
        map.insert(
            1usize,
            vec![WrittenFigureAsset {
                page_num: 1,
                index: 1,
                rel_path: "assets/keep.png".into(),
                width: 10,
                height: 10,
                bbox: Some((0.0, 0.0, 10.0, 10.0)),
            }],
        );
        let dir = tempfile::tempdir().unwrap();
        let out = apply_filter_result_or_keep(
            map.clone(),
            Err(PdfConversionError::Backend("vlm down".into())),
            dir.path(),
            false,
        );
        assert_eq!(out.get(&1).map(|v| v.len()), Some(1));
    }

    #[test]
    fn manifest_on_disk_prunes_include_from_pdf() {
        let dir = tempfile::tempdir().unwrap();
        let results = vec![
            FigureFilterResult {
                rel_path: "assets/keep.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::Diagram,
                is_figure: true,
                description: "ok".into(),
            },
            FigureFilterResult {
                rel_path: "assets/drop.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::Logo,
                is_figure: false,
                description: String::new(),
            },
        ];
        write_manifest(dir.path(), &results).unwrap();
        let mut map = HashMap::new();
        let fig = |rel: &str| WrittenFigureAsset {
            page_num: 1,
            index: 1,
            rel_path: rel.into(),
            width: 10,
            height: 10,
            bbox: Some((0.0, 0.0, 10.0, 10.0)),
        };
        map.insert(1usize, vec![fig("assets/keep.png"), fig("assets/drop.png")]);
        let pruned = prune_figure_map_using_manifest(map, dir.path(), false);
        let paths: Vec<_> = pruned
            .values()
            .flatten()
            .map(|f| f.rel_path.as_str())
            .collect();
        assert_eq!(paths, vec!["assets/keep.png"]);
    }

    #[test]
    fn budget_drops_lowest_area_first() {
        let cands: Vec<FigureCandidate> = (0..3)
            .map(|i| FigureCandidate {
                rel_path: format!("assets/{i}.png"),
                full_path: std::path::PathBuf::from("x"),
                page_num: 1,
                label: String::new(),
                area_px: (i as u64 + 1) * 10,
            })
            .collect();
        let kept = budget_candidates(&cands, 2);
        assert_eq!(kept.len(), 2);
        let areas: Vec<u64> = kept.iter().map(|c| c.area_px).collect();
        assert!(areas.contains(&20));
        assert!(areas.contains(&30));
        assert!(!areas.contains(&10));
    }

    #[test]
    fn prune_chart_crop_paths_drops_artefacts() {
        let mut charts = HashMap::new();
        charts.insert(1usize, "assets/page-0001-chart.png".into());
        charts.insert(2usize, "assets/page-0002-chart.png".into());
        charts.insert(3usize, "assets/page-0003-chart.png".into());
        let results = vec![
            FigureFilterResult {
                rel_path: "assets/page-0001-chart.png".into(),
                page_num: 1,
                label: "Chart".into(),
                kind: FigureKind::BarChart,
                is_figure: true,
                description: "bars".into(),
            },
            FigureFilterResult {
                rel_path: "assets/page-0002-chart.png".into(),
                page_num: 2,
                label: "Chart".into(),
                kind: FigureKind::Logo,
                is_figure: false,
                description: String::new(),
            },
        ];
        let pruned = prune_chart_crop_paths(charts, &results);
        assert_eq!(
            pruned.get(&1).map(String::as_str),
            Some("assets/page-0001-chart.png")
        );
        assert!(!pruned.contains_key(&2));
        assert_eq!(
            pruned.get(&3).map(String::as_str),
            Some("assets/page-0003-chart.png"),
            "unseen chart paths fail-open"
        );
    }

    #[test]
    fn inject_kept_descriptions_promotes_charts_not_logos() {
        let md = "![keep](assets/chart.png)\n\n![logo](assets/logo.png)\n";
        let results = vec![
            FigureFilterResult {
                rel_path: "assets/chart.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::LineChart,
                is_figure: true,
                description: "Revenue rose from 10 to 20.".into(),
            },
            FigureFilterResult {
                rel_path: "assets/logo.png".into(),
                page_num: 1,
                label: String::new(),
                kind: FigureKind::Logo,
                is_figure: false,
                description: "should not appear".into(),
            },
        ];
        let out = inject_kept_descriptions(md, &results);
        assert!(out.contains("Revenue rose from 10 to 20."));
        assert!(out.contains("<!-- edgequake-figure-vision:assets/chart.png -->"));
        assert!(!out.contains("should not appear"));
        let again = inject_kept_descriptions(&out, &results);
        assert_eq!(
            again.matches("Revenue rose from 10 to 20.").count(),
            1,
            "inject is idempotent"
        );
    }

    #[test]
    fn strip_discarded_asset_lines_drops_logo_href() {
        let md = "![keep](assets/fig.png)\n![logo](assets/logo.png)\n";
        let mut discarded = HashSet::new();
        discarded.insert("assets/logo.png".into());
        let out = strip_discarded_asset_lines(md, &discarded);
        assert!(out.contains("assets/fig.png"));
        assert!(!out.contains("assets/logo.png"));
    }
}
