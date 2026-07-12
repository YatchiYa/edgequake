//! SPEC-047 Phase C — vision `<drawing/>` ref contract (parallel-safe, no shared env).

mod common;

use common::spec026_multimodal::{vision_page_markdown, write_page_png_asset};
use edgequake_api::services::{
    document_mm_assets_root, multimodal_images_requested, page_drawing_assets_config,
};
use edgequake_pdf::{format_drawing_tag, scan_inline_image_refs};

#[test]
fn multimodal_images_flag_parses_ite() {
    assert!(multimodal_images_requested(Some("ite")));
    assert!(!multimodal_images_requested(Some("te")));
}

#[test]
fn page_drawing_config_enabled_only_for_images() {
    assert!(page_drawing_assets_config("doc-1", Some("i")).is_some());
    assert!(page_drawing_assets_config("doc-1", None).is_none());
}

#[test]
fn assembled_markdown_drawing_tags_roundtrip_scan() {
    let md = vision_page_markdown("bench-doc", &[(7, "Quarterly revenue")]);
    let refs = scan_inline_image_refs(&md);
    assert_eq!(refs.len(), 1);
    assert_eq!(
        refs[0].asset_path.as_deref(),
        Some("assets/page-0007-fig-01.png")
    );
    assert!(refs[0].item_id.contains("bench-doc"));
}

#[test]
fn format_drawing_tag_matches_inline_scanner() {
    let tag = format_drawing_tag("im-test-001", "assets/page-0001.png", None);
    let refs = scan_inline_image_refs(&format!("Intro\n{tag}\n"));
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].item_id, "im-test-001");
}

#[test]
fn document_assets_root_is_unique_per_doc() {
    let a = document_mm_assets_root("aaa");
    let b = document_mm_assets_root("bbb");
    assert_ne!(a, b);
}

#[test]
fn page_png_asset_resolves_under_assets_root() {
    let dir = tempfile::tempdir().unwrap();
    write_page_png_asset(dir.path(), 3);
    let path = dir.path().join("assets/page-0003.png");
    assert!(path.is_file());
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]));
}
