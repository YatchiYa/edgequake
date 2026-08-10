//! SPEC-015V — Vision extract resolve + metadata snapshot + upload overlay contract.

use edgequake_pdf::{
    VisionAssetWritePlan, VisionExtractConfig, VisionExtractOverlay, DOC_META_VISION_EXTRACT,
    META_EXTRACT_FIGURES,
};
use std::collections::HashMap;

#[test]
fn resolve_upload_overrides_workspace_figures_off() {
    let mut meta = HashMap::new();
    meta.insert(META_EXTRACT_FIGURES.to_string(), serde_json::json!(true));
    let overlay = VisionExtractOverlay {
        extract_figures: Some(false),
        chart_system_prompt: Some("CUSTOM CHART".into()),
        ..Default::default()
    };
    let cfg = VisionExtractConfig::resolve(&meta, &overlay).unwrap();
    assert!(!cfg.extract_figures);
    assert!(cfg.extract_images);
    assert_eq!(cfg.chart_system_prompt.as_deref(), Some("CUSTOM CHART"));
}

#[test]
fn snapshot_round_trips_through_json() {
    let cfg = VisionExtractConfig {
        extract_images: false,
        extract_charts: true,
        extract_figures: false,
        page_system_prompt: Some("PAGE".into()),
        ..Default::default()
    };
    let snap = cfg.to_snapshot_value();
    let mut meta = serde_json::Map::new();
    meta.insert(DOC_META_VISION_EXTRACT.to_string(), snap.clone());
    let back: VisionExtractConfig = serde_json::from_value(snap).unwrap();
    assert_eq!(back, cfg);
    assert!(meta.contains_key(DOC_META_VISION_EXTRACT));
}

/// G7: multipart-shaped overlay (as parsed into VisionExtractOverlay) resolves into task snapshot.
#[test]
fn g7_multipart_overlay_lands_in_snapshot() {
    let overlay = VisionExtractOverlay {
        extract_images: Some(true),
        extract_charts: Some(false),
        extract_figures: Some(true),
        page_system_prompt: None,
        image_system_prompt: None,
        chart_system_prompt: Some("E2E chart".into()),
        figure_system_prompt: None,
    };
    let cfg = VisionExtractConfig::resolve(&HashMap::new(), &overlay).unwrap();
    assert!(cfg.extract_images);
    assert!(!cfg.extract_charts);
    assert!(cfg.extract_figures);
    assert_eq!(cfg.chart_system_prompt.as_deref(), Some("E2E chart"));

    let plan = VisionAssetWritePlan::from_config(&cfg);
    assert!(plan.write_page_pngs);
    assert!(!plan.write_charts);
    assert!(plan.write_figures);
    assert!(!plan.promote_fig_as_chart);

    let snap = cfg.to_snapshot_value();
    assert_eq!(snap["extract_charts"], serde_json::json!(false));
    assert_eq!(snap["chart_system_prompt"], serde_json::json!("E2E chart"));
}

#[test]
fn g13_openapi_snapshot_includes_vision_extract_fields() {
    let snap = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../edgequake_webui/openapi/openapi.snapshot.json");
    let body = std::fs::read_to_string(&snap).expect("openapi snapshot");
    for key in [
        "vision_extract_images",
        "vision_extract_charts",
        "vision_extract_figures",
        "vision_page_system_prompt",
        "vision_image_system_prompt",
        "vision_chart_system_prompt",
        "vision_figure_system_prompt",
    ] {
        assert!(
            body.contains(key),
            "OpenAPI snapshot missing {key} — run: make codegen-openapi-refresh"
        );
    }
}
