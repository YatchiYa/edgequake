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
//! ```
//!
//! ## SOLID
//!
//! - **S**: Filter only — does not write PNG files (that is `region_assets`).
//! - **O**: Opt-in via `Option<Arc<dyn LLMProvider>>`; absent → no-op passthrough.
//! - **L**: Any `LLMProvider` implementation works identically.
//! - **I**: Depends on `LLMProvider` trait only, not on the full VisionConversionConfig.
//! - **D**: `FigureFilter` is injected with a provider; does not create one.

use std::path::Path;
use std::sync::Arc;

use base64::Engine as _;
use edgequake_llm::{ChatMessage, CompletionOptions, ImageData, LLMProvider};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

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
}

impl FigureFilter {
    /// Create a new filter backed by `provider`.
    pub fn new(provider: Arc<dyn LLMProvider>) -> Self {
        Self { provider }
    }

    /// Run both passes on every candidate.  Candidates whose Pass-1 kind is
    /// noise are excluded from Pass 2 but still appear in the result list with
    /// `is_figure = false`.
    pub async fn run(
        &self,
        candidates: &[FigureCandidate],
    ) -> Result<Vec<FigureFilterResult>, PdfConversionError> {
        let mut results = Vec::with_capacity(candidates.len());
        for c in candidates {
            let result = self.process_one(c).await;
            results.push(result);
        }
        Ok(results)
    }

    async fn process_one(&self, c: &FigureCandidate) -> FigureFilterResult {
        // ── Pass 1: filter ────────────────────────────────────────────────────
        let kind = match self.pass1_classify(&c.full_path).await {
            Ok(k) => k,
            Err(e) => {
                warn!(rel_path = %c.rel_path, error = %e, "Pass-1 filter failed; keeping crop");
                FigureKind::Other
            }
        };
        let is_figure = kind.is_figure();
        debug!(rel_path = %c.rel_path, kind = ?kind, is_figure, "Pass-1 result");

        if !is_figure {
            return FigureFilterResult {
                rel_path: c.rel_path.clone(),
                page_num: c.page_num,
                label: c.label.clone(),
                kind,
                is_figure: false,
                description: String::new(),
            };
        }

        // ── Pass 2: specialize ────────────────────────────────────────────────
        let description = match self.pass2_describe(&c.full_path, &kind).await {
            Ok(d) => d,
            Err(e) => {
                warn!(rel_path = %c.rel_path, error = %e, "Pass-2 specialize failed");
                String::new()
            }
        };
        debug!(rel_path = %c.rel_path, desc_chars = description.len(), "Pass-2 result");

        FigureFilterResult {
            rel_path: c.rel_path.clone(),
            page_num: c.page_num,
            label: c.label.clone(),
            kind,
            is_figure: true,
            description,
        }
    }

    // ── Pass-1 implementation ─────────────────────────────────────────────────

    async fn pass1_classify(&self, png_path: &Path) -> Result<FigureKind, PdfConversionError> {
        let image = load_image_data(png_path)?;
        let opts = CompletionOptions {
            max_tokens: Some(80),
            temperature: Some(0.0),
            ..Default::default()
        };
        let messages = vec![
            ChatMessage::system(FIGURE_FILTER_PASS1_SYSTEM),
            ChatMessage::user_with_images(figure_filter_pass1_prompt(), vec![image]),
        ];
        let response = self
            .provider
            .chat(&messages, Some(&opts))
            .await
            .map_err(|e| PdfConversionError::Backend(format!("Pass-1 LLM call failed: {e}")))?;

        let raw = response.content.trim().to_string();
        parse_pass1_json(&raw)
    }

    // ── Pass-2 implementation ─────────────────────────────────────────────────

    async fn pass2_describe(
        &self,
        png_path: &Path,
        kind: &FigureKind,
    ) -> Result<String, PdfConversionError> {
        let image = load_image_data(png_path)?;
        let opts = CompletionOptions {
            max_tokens: Some(600),
            temperature: Some(0.0),
            ..Default::default()
        };
        let messages = vec![
            ChatMessage::system(FIGURE_FILTER_PASS2_SYSTEM),
            ChatMessage::user_with_images(figure_filter_pass2_prompt(kind), vec![image]),
        ];
        let response = self
            .provider
            .chat(&messages, Some(&opts))
            .await
            .map_err(|e| PdfConversionError::Backend(format!("Pass-2 LLM call failed: {e}")))?;

        Ok(response.content.trim().to_string())
    }
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
fn parse_pass1_json(raw: &str) -> Result<FigureKind, PdfConversionError> {
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

    let kind_str = v
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("other");

    Ok(FigureKind::from_str_fuzzy(kind_str))
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
        // Other is kept (conservative)
        assert!(FigureKind::Other.is_figure());
    }

    #[test]
    fn parse_pass1_json_ok() {
        let raw = r#"{"kind": "bar_chart", "is_figure": true, "confidence": 0.99}"#;
        let k = parse_pass1_json(raw).unwrap();
        assert_eq!(k, FigureKind::BarChart);
    }

    #[test]
    fn parse_pass1_json_strips_fences() {
        let raw = "```json\n{\"kind\": \"text_block\", \"is_figure\": false}\n```";
        let k = parse_pass1_json(raw).unwrap();
        assert_eq!(k, FigureKind::TextBlock);
    }

    #[test]
    fn parse_pass1_json_unknown_kind() {
        let raw = r#"{"kind": "unknown_thing"}"#;
        let k = parse_pass1_json(raw).unwrap();
        assert_eq!(k, FigureKind::Other);
    }

    #[test]
    fn kind_fuzzy_parsing() {
        assert_eq!(FigureKind::from_str_fuzzy("architecture"), FigureKind::ArchitectureDiagram);
        assert_eq!(FigureKind::from_str_fuzzy("flow chart"), FigureKind::Flowchart);
        assert_eq!(FigureKind::from_str_fuzzy("LOGO"), FigureKind::Logo);
    }
}
