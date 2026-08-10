//! SPEC-015V — Vision extract toggles + prompt overrides.
//!
//! Single resolve type for Images / Charts / Figures On/Off and optional
//! system-prompt replacements (Pass A page + Pass B image/chart/figure).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Maximum UTF-8 bytes for a single vision system-prompt override (EC-015V-6).
pub const VISION_PROMPT_MAX_BYTES: usize = 32 * 1024;

/// SPEC-015V — which asset writers run for a resolved extract policy.
///
/// SSOT for vision.rs branching (DRY + unit-testable without a full VLM convert).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisionAssetWritePlan {
    /// Full-page viewer PNGs (`page-NNNN.png`).
    pub write_page_pngs: bool,
    /// Chart ink crops (`page-NNNN-chart.png`).
    pub write_charts: bool,
    /// Embedded + caption figure crops (`page-NNNN-fig-MM.png`, tables).
    pub write_figures: bool,
    /// Fig→chart promotion only when both charts and figures are enabled.
    pub promote_fig_as_chart: bool,
}

impl VisionAssetWritePlan {
    pub fn from_flags(extract_images: bool, extract_charts: bool, extract_figures: bool) -> Self {
        Self {
            write_page_pngs: extract_images,
            write_charts: extract_charts,
            write_figures: extract_figures,
            promote_fig_as_chart: extract_charts && extract_figures,
        }
    }

    pub fn from_config(cfg: &VisionExtractConfig) -> Self {
        Self::from_flags(cfg.extract_images, cfg.extract_charts, cfg.extract_figures)
    }

    pub fn any_writer(self) -> bool {
        self.write_page_pngs || self.write_charts || self.write_figures
    }
}

/// Metadata / API keys (SSOT for FE + BE).
pub const META_EXTRACT_IMAGES: &str = "vision_extract_images";
pub const META_EXTRACT_CHARTS: &str = "vision_extract_charts";
pub const META_EXTRACT_FIGURES: &str = "vision_extract_figures";
pub const META_PAGE_SYSTEM_PROMPT: &str = "vision_page_system_prompt";
pub const META_IMAGE_SYSTEM_PROMPT: &str = "vision_image_system_prompt";
pub const META_CHART_SYSTEM_PROMPT: &str = "vision_chart_system_prompt";
pub const META_FIGURE_SYSTEM_PROMPT: &str = "vision_figure_system_prompt";

/// Document metadata snapshot key (LAW-015V-6).
pub const DOC_META_VISION_EXTRACT: &str = "vision_extract";

/// Resolved Vision extract policy for one conversion / analyze job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisionExtractConfig {
    /// Page PNGs + Pass-B image/drawing modality.
    pub extract_images: bool,
    /// Chart ink crops + Pass-B chart modality.
    pub extract_charts: bool,
    /// Figure crops + Pass-B figure modality.
    pub extract_figures: bool,
    /// Pass A page OCR system prompt override (None → SSOT).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_system_prompt: Option<String>,
    /// Pass B image system prompt override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_system_prompt: Option<String>,
    /// Pass B chart system prompt override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chart_system_prompt: Option<String>,
    /// Pass B figure system prompt override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figure_system_prompt: Option<String>,
}

impl Default for VisionExtractConfig {
    fn default() -> Self {
        Self {
            extract_images: true,
            extract_charts: true,
            extract_figures: true,
            page_system_prompt: None,
            image_system_prompt: None,
            chart_system_prompt: None,
            figure_system_prompt: None,
        }
    }
}

/// Sparse overlay from upload multipart or API (None = inherit).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VisionExtractOverlay {
    pub extract_images: Option<bool>,
    pub extract_charts: Option<bool>,
    pub extract_figures: Option<bool>,
    /// `Some("")` clears to SSOT; `None` inherits.
    pub page_system_prompt: Option<String>,
    pub image_system_prompt: Option<String>,
    pub chart_system_prompt: Option<String>,
    pub figure_system_prompt: Option<String>,
}

impl VisionExtractConfig {
    /// Normalize a prompt field: trim; empty → None; enforce max length.
    pub fn normalize_prompt(raw: Option<String>) -> Result<Option<String>, String> {
        let Some(s) = raw else {
            return Ok(None);
        };
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        if trimmed.len() > VISION_PROMPT_MAX_BYTES {
            return Err(format!(
                "Vision system prompt exceeds max length of {VISION_PROMPT_MAX_BYTES} bytes"
            ));
        }
        Ok(Some(trimmed.to_string()))
    }

    /// Load from workspace (or document) metadata map. Absent bools → true.
    pub fn from_metadata(meta: &HashMap<String, serde_json::Value>) -> Self {
        let mut cfg = Self::default();
        cfg.extract_images = meta_bool(meta, META_EXTRACT_IMAGES).unwrap_or(true);
        cfg.extract_charts = meta_bool(meta, META_EXTRACT_CHARTS).unwrap_or(true);
        cfg.extract_figures = meta_bool(meta, META_EXTRACT_FIGURES).unwrap_or(true);
        cfg.page_system_prompt = meta_prompt(meta, META_PAGE_SYSTEM_PROMPT);
        cfg.image_system_prompt = meta_prompt(meta, META_IMAGE_SYSTEM_PROMPT);
        cfg.chart_system_prompt = meta_prompt(meta, META_CHART_SYSTEM_PROMPT);
        cfg.figure_system_prompt = meta_prompt(meta, META_FIGURE_SYSTEM_PROMPT);
        cfg
    }

    /// Apply sparse overlay (upload wins). Prompt `Some("")` clears to None.
    pub fn apply_overlay(&self, overlay: &VisionExtractOverlay) -> Result<Self, String> {
        let mut out = self.clone();
        if let Some(v) = overlay.extract_images {
            out.extract_images = v;
        }
        if let Some(v) = overlay.extract_charts {
            out.extract_charts = v;
        }
        if let Some(v) = overlay.extract_figures {
            out.extract_figures = v;
        }
        if overlay.page_system_prompt.is_some() {
            out.page_system_prompt =
                Self::normalize_prompt(overlay.page_system_prompt.clone())?;
        }
        if overlay.image_system_prompt.is_some() {
            out.image_system_prompt =
                Self::normalize_prompt(overlay.image_system_prompt.clone())?;
        }
        if overlay.chart_system_prompt.is_some() {
            out.chart_system_prompt =
                Self::normalize_prompt(overlay.chart_system_prompt.clone())?;
        }
        if overlay.figure_system_prompt.is_some() {
            out.figure_system_prompt =
                Self::normalize_prompt(overlay.figure_system_prompt.clone())?;
        }
        Ok(out)
    }

    /// Resolve: workspace metadata + upload overlay (LAW-015V-2).
    pub fn resolve(
        workspace_meta: &HashMap<String, serde_json::Value>,
        upload: &VisionExtractOverlay,
    ) -> Result<Self, String> {
        Self::from_metadata(workspace_meta).apply_overlay(upload)
    }

    /// Write bools/prompts into a metadata map (workspace apply).
    pub fn apply_to_metadata(
        meta: &mut HashMap<String, serde_json::Value>,
        overlay: &VisionExtractOverlay,
    ) -> Result<(), String> {
        if let Some(v) = overlay.extract_images {
            meta.insert(META_EXTRACT_IMAGES.to_string(), serde_json::json!(v));
        }
        if let Some(v) = overlay.extract_charts {
            meta.insert(META_EXTRACT_CHARTS.to_string(), serde_json::json!(v));
        }
        if let Some(v) = overlay.extract_figures {
            meta.insert(META_EXTRACT_FIGURES.to_string(), serde_json::json!(v));
        }
        apply_prompt_key(meta, META_PAGE_SYSTEM_PROMPT, &overlay.page_system_prompt)?;
        apply_prompt_key(meta, META_IMAGE_SYSTEM_PROMPT, &overlay.image_system_prompt)?;
        apply_prompt_key(meta, META_CHART_SYSTEM_PROMPT, &overlay.chart_system_prompt)?;
        apply_prompt_key(meta, META_FIGURE_SYSTEM_PROMPT, &overlay.figure_system_prompt)?;
        Ok(())
    }

    /// JSON object for document ingest snapshot.
    pub fn to_snapshot_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
    }

    /// Any visual asset extraction enabled.
    pub fn any_extract_enabled(&self) -> bool {
        self.extract_images || self.extract_charts || self.extract_figures
    }
}

fn meta_bool(meta: &HashMap<String, serde_json::Value>, key: &str) -> Option<bool> {
    meta.get(key).and_then(|v| v.as_bool())
}

fn meta_prompt(meta: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    meta.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn apply_prompt_key(
    meta: &mut HashMap<String, serde_json::Value>,
    key: &str,
    raw: &Option<String>,
) -> Result<(), String> {
    let Some(s) = raw else {
        return Ok(());
    };
    match VisionExtractConfig::normalize_prompt(Some(s.clone()))? {
        None => {
            meta.remove(key);
        }
        Some(prompt) => {
            meta.insert(key.to_string(), serde_json::json!(prompt));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_on_no_prompts() {
        let cfg = VisionExtractConfig::default();
        assert!(cfg.extract_images && cfg.extract_charts && cfg.extract_figures);
        assert!(cfg.page_system_prompt.is_none());
    }

    #[test]
    fn absent_metadata_defaults_true() {
        let cfg = VisionExtractConfig::from_metadata(&HashMap::new());
        assert!(cfg.extract_images && cfg.extract_charts && cfg.extract_figures);
    }

    #[test]
    fn upload_overrides_workspace() {
        let mut meta = HashMap::new();
        meta.insert(META_EXTRACT_FIGURES.to_string(), serde_json::json!(true));
        meta.insert(
            META_CHART_SYSTEM_PROMPT.to_string(),
            serde_json::json!("WS chart"),
        );
        let overlay = VisionExtractOverlay {
            extract_figures: Some(false),
            chart_system_prompt: Some("Upload chart".into()),
            ..Default::default()
        };
        let cfg = VisionExtractConfig::resolve(&meta, &overlay).unwrap();
        assert!(!cfg.extract_figures);
        assert!(cfg.extract_images);
        assert_eq!(cfg.chart_system_prompt.as_deref(), Some("Upload chart"));
    }

    #[test]
    fn empty_prompt_clears_to_ssot() {
        let base = VisionExtractConfig {
            page_system_prompt: Some("keep".into()),
            ..Default::default()
        };
        let overlay = VisionExtractOverlay {
            page_system_prompt: Some("  ".into()),
            ..Default::default()
        };
        let cfg = base.apply_overlay(&overlay).unwrap();
        assert!(cfg.page_system_prompt.is_none());
    }

    #[test]
    fn prompt_too_long_errors() {
        let huge = "x".repeat(VISION_PROMPT_MAX_BYTES + 1);
        let err = VisionExtractConfig::normalize_prompt(Some(huge)).unwrap_err();
        assert!(err.contains("max length"));
    }

    #[test]
    fn apply_to_metadata_round_trip() {
        let mut meta = HashMap::new();
        let overlay = VisionExtractOverlay {
            extract_images: Some(false),
            page_system_prompt: Some("PAGE".into()),
            ..Default::default()
        };
        VisionExtractConfig::apply_to_metadata(&mut meta, &overlay).unwrap();
        let cfg = VisionExtractConfig::from_metadata(&meta);
        assert!(!cfg.extract_images);
        assert!(cfg.extract_charts);
        assert_eq!(cfg.page_system_prompt.as_deref(), Some("PAGE"));
    }

    #[test]
    fn write_plan_gates_and_promotion() {
        let all = VisionAssetWritePlan::from_flags(true, true, true);
        assert!(all.any_writer());
        assert!(all.promote_fig_as_chart);

        let charts_only = VisionAssetWritePlan::from_flags(false, true, false);
        assert!(charts_only.write_charts);
        assert!(!charts_only.write_page_pngs);
        assert!(!charts_only.write_figures);
        assert!(!charts_only.promote_fig_as_chart);

        let none = VisionAssetWritePlan::from_flags(false, false, false);
        assert!(!none.any_writer());
    }
}
