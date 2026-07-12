//! Document-scoped multimodal asset directories (SPEC-047 Phase C MV-21).

use std::path::PathBuf;

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
) -> Option<edgequake_pdf::PageDrawingAssetsConfig> {
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
    Some(edgequake_pdf::PageDrawingAssetsConfig {
        assets_root: document_mm_assets_root(document_id),
        id_prefix: Some(document_id.to_string()),
        emit_analyze_tags: true,
    })
}

/// Page PNG + viewer images for Vision PDF conversion.
///
/// Full-page `page-NNNN.png` rasters are dual-pane PDF context only.
/// Analyze `<drawing/>` tags stay gated on the `i` process option **and**
/// `VLM_PROCESS_ENABLE` — never emit placeholders that Pass B cannot replace.
pub fn page_drawing_assets_config_for_vision(
    document_id: &str,
    process_options: Option<&str>,
) -> edgequake_pdf::PageDrawingAssetsConfig {
    let images = multimodal_images_requested(process_options);
    let vlm_on = super::multimodal::vlm_process_enabled();
    edgequake_pdf::PageDrawingAssetsConfig {
        assets_root: document_mm_assets_root(document_id),
        id_prefix: Some(document_id.to_string()),
        emit_analyze_tags: images && vlm_on,
    }
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
    fn vision_always_emits_viewer_assets() {
        let cfg = page_drawing_assets_config_for_vision("abc", None);
        assert!(cfg.assets_root.ends_with("abc"));
        assert!(!cfg.emit_analyze_tags);
    }

    #[test]
    #[serial_test::serial]
    fn vision_analyze_tags_require_vlm_enable_and_i_flag() {
        std::env::remove_var("VLM_PROCESS_ENABLE");
        let default_on = page_drawing_assets_config_for_vision("abc", Some("i"));
        assert!(
            default_on.emit_analyze_tags,
            "VLM_PROCESS_ENABLE defaults on — emit tags when process_options=i"
        );
        std::env::set_var("VLM_PROCESS_ENABLE", "false");
        let off = page_drawing_assets_config_for_vision("abc", Some("i"));
        assert!(
            !off.emit_analyze_tags,
            "must not emit orphan <drawing/> when VLM_PROCESS_ENABLE=false"
        );
        std::env::remove_var("VLM_PROCESS_ENABLE");
    }

    #[test]
    fn document_root_is_under_base() {
        let root = document_mm_assets_root("abc-123");
        assert!(root.ends_with("abc-123"));
        assert!(root.starts_with(mm_assets_base_dir()));
    }
}
