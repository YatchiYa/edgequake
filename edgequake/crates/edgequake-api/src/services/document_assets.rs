//! Document-scoped multimodal asset directories (SPEC-047 Phase C MV-21).

use std::path::PathBuf;

use edgequake_pdf::{PageDrawingAssetsConfig, VisionExtractConfig};

/// Root directory for all document mm-assets (override via `EDGEQUAKE_MM_ASSETS_DIR`).
pub fn mm_assets_base_dir() -> PathBuf {
    std::env::var("EDGEQUAKE_MM_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("edgequake-mm-assets"))
}

/// Per-document assets root passed to `resolve_image_asset` as `base_dir`.
pub fn document_mm_assets_root(document_id: &str) -> PathBuf {
    mm_assets_base_dir().join(document_id)
}

/// Whether multimodal image analyze is requested (`process_options` contains `i`).
pub fn multimodal_images_requested(process_options: Option<&str>) -> bool {
    process_options
        .map(super::vision_content::MultimodalProcessOptions::from_option_str)
        .is_some_and(|opts| opts.images)
}

/// Build pdf2md page-drawing config when image analyze is enabled.
pub fn page_drawing_assets_config(
    document_id: &str,
    process_options: Option<&str>,
) -> Option<PageDrawingAssetsConfig> {
    if !multimodal_images_requested(process_options) {
        return None;
    }
    if !super::multimodal::vlm_process_enabled() {
        tracing::warn!(
            document_id,
            "process_options includes 'i' but VLM_PROCESS_ENABLE is false — skipping drawing tags"
        );
        return None;
    }
    let mut cfg = PageDrawingAssetsConfig::with_defaults(
        document_mm_assets_root(document_id),
        Some(document_id.to_string()),
    );
    cfg.emit_analyze_tags = true;
    Some(cfg)
}

/// Page PNG + viewer images for Vision PDF conversion.
///
/// Full-page `page-NNNN.png` rasters are dual-pane PDF context only.
/// Analyze `<drawing/>` tags stay gated on the `i` process option **and**
/// `VLM_PROCESS_ENABLE` — never emit placeholders that Pass B cannot replace.
///
/// SPEC-015V: `extract` gates writers; when all extract flags are false, analyze
/// tags are also suppressed (EC-015V-9).
pub fn page_drawing_assets_config_for_vision(
    document_id: &str,
    process_options: Option<&str>,
    extract: &VisionExtractConfig,
) -> PageDrawingAssetsConfig {
    let images = multimodal_images_requested(process_options);
    let vlm_on = super::multimodal::vlm_process_enabled();
    let mut cfg = PageDrawingAssetsConfig::with_defaults(
        document_mm_assets_root(document_id),
        Some(document_id.to_string()),
    );
    cfg.apply_vision_extract(extract);
    cfg.emit_analyze_tags = images && vlm_on && extract.any_extract_enabled();
    cfg
}

/// Asset base dir for multimodal analyze stage (same root as page PNG writes).
pub fn multimodal_asset_base_dir(
    document_id: &str,
    process_options: Option<&str>,
) -> Option<PathBuf> {
    multimodal_images_requested(process_options).then(|| document_mm_assets_root(document_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_flag_controls_page_assets() {
        assert!(multimodal_images_requested(Some("ite")));
        assert!(multimodal_images_requested(Some("i")));
        assert!(!multimodal_images_requested(Some("te")));
        assert!(!multimodal_images_requested(None));
    }

    #[test]
    fn vision_always_emits_viewer_assets_by_default() {
        let cfg =
            page_drawing_assets_config_for_vision("abc", None, &VisionExtractConfig::default());
        assert!(cfg.assets_root.ends_with("abc"));
        assert!(!cfg.emit_analyze_tags);
        assert!(cfg.extract_images && cfg.extract_charts && cfg.extract_figures);
    }

    #[test]
    fn vision_respects_extract_flags() {
        let extract = VisionExtractConfig {
            extract_images: false,
            extract_charts: true,
            extract_figures: false,
            ..Default::default()
        };
        let cfg = page_drawing_assets_config_for_vision("abc", Some("i"), &extract);
        assert!(!cfg.extract_images);
        assert!(cfg.extract_charts);
        assert!(!cfg.extract_figures);
    }

    #[test]
    #[serial_test::serial]
    fn vision_analyze_tags_require_vlm_enable_and_i_flag() {
        std::env::remove_var("VLM_PROCESS_ENABLE");
        let default_on = page_drawing_assets_config_for_vision(
            "abc",
            Some("i"),
            &VisionExtractConfig::default(),
        );
        assert!(
            default_on.emit_analyze_tags,
            "VLM_PROCESS_ENABLE defaults on — emit tags when process_options=i"
        );
        std::env::set_var("VLM_PROCESS_ENABLE", "false");
        let off = page_drawing_assets_config_for_vision(
            "abc",
            Some("i"),
            &VisionExtractConfig::default(),
        );
        assert!(
            !off.emit_analyze_tags,
            "must not emit orphan <drawing/> when VLM_PROCESS_ENABLE=false"
        );
        std::env::remove_var("VLM_PROCESS_ENABLE");
    }

    #[test]
    fn extract_all_false_suppresses_analyze_even_with_i() {
        let extract = VisionExtractConfig {
            extract_images: false,
            extract_charts: false,
            extract_figures: false,
            ..Default::default()
        };
        let cfg = page_drawing_assets_config_for_vision("abc", Some("i"), &extract);
        assert!(!cfg.emit_analyze_tags);
    }

    #[test]
    fn document_root_is_under_base() {
        let root = document_mm_assets_root("abc-123");
        assert!(root.ends_with("abc-123"));
        assert!(root.starts_with(mm_assets_base_dir()));
    }

    #[test]
    #[serial_test::serial]
    fn attach_figure_filter_honors_extract_flag() {
        std::env::remove_var("EDGEQUAKE_FIGURE_FILTER");
        let mock = std::sync::Arc::new(edgequake_llm::MockProvider::new());
        let provider: std::sync::Arc<dyn edgequake_llm::LLMProvider> = mock;
        let mut cfg = PageDrawingAssetsConfig::with_defaults(
            std::path::PathBuf::from("/tmp/x"),
            Some("abc".into()),
        );
        cfg.extract_figures = false;
        cfg.extract_charts = false;
        cfg.attach_figure_filter_if_enabled(Some(std::sync::Arc::clone(&provider)));
        assert!(cfg.figure_filter_provider.is_none());
        cfg.extract_charts = true;
        cfg.attach_figure_filter_if_enabled(Some(std::sync::Arc::clone(&provider)));
        assert!(
            cfg.figure_filter_provider.is_some(),
            "charts-only extract still attaches the filter"
        );
        cfg.figure_filter_provider = None;
        cfg.extract_charts = false;
        cfg.extract_figures = true;
        cfg.attach_figure_filter_if_enabled(Some(provider));
        assert!(cfg.figure_filter_provider.is_some());
    }

    #[test]
    #[serial_test::serial]
    fn attach_figure_filter_default_on_when_env_unset() {
        std::env::remove_var("EDGEQUAKE_FIGURE_FILTER");
        let mock = std::sync::Arc::new(edgequake_llm::MockProvider::new());
        let provider: std::sync::Arc<dyn edgequake_llm::LLMProvider> = mock;
        let mut cfg = PageDrawingAssetsConfig::with_defaults(
            std::path::PathBuf::from("/tmp/x"),
            Some("abc".into()),
        );
        cfg.extract_figures = true;
        cfg.attach_figure_filter_if_enabled(Some(provider));
        assert!(
            cfg.figure_filter_provider.is_some(),
            "LAW-128-14: filter default-on when env unset"
        );
    }

    #[test]
    #[serial_test::serial]
    fn attach_figure_filter_honors_env_off() {
        std::env::set_var("EDGEQUAKE_FIGURE_FILTER", "0");
        let mock = std::sync::Arc::new(edgequake_llm::MockProvider::new());
        let provider: std::sync::Arc<dyn edgequake_llm::LLMProvider> = mock;
        let mut cfg = PageDrawingAssetsConfig::with_defaults(
            std::path::PathBuf::from("/tmp/x"),
            Some("abc".into()),
        );
        cfg.extract_figures = true;
        cfg.attach_figure_filter_if_enabled(Some(provider));
        assert!(cfg.figure_filter_provider.is_none());
        std::env::remove_var("EDGEQUAKE_FIGURE_FILTER");
    }
}
