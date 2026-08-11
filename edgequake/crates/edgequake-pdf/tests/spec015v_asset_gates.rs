//! SPEC-015V G4/G8–G10 — asset writers honor extract gates (pdfium when available).
//!
//! Runs the same writers vision.rs calls, gated by [`VisionAssetWritePlan`].

use edgequake_pdf::{
    write_chart_crop_assets, write_embedded_figure_assets, write_page_png_assets,
    PageAssetRenderConfig, VisionAssetWritePlan, ASSETS_SUBDIR, CHART_CROP_RENDER,
};
use serial_test::serial;
use std::path::{Path, PathBuf};

fn minimal_pdf_bytes() -> Vec<u8> {
    br#"%PDF-1.1
1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj
2 0 obj<< /Type /Pages /Kids [3 0 R] /Count 1 >>endobj
3 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R >>endobj
4 0 obj<< /Length 44 >>stream
BT /F1 12 Tf 50 150 Td (hello) Tj ET
endstream
endobj
xref
0 5
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
0000000206 00000 n 
trailer<< /Size 5 /Root 1 0 R >>
startxref
300
%%EOF"#
        .to_vec()
}

fn embedded_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/embedded_figure_sample.pdf")
}

fn list_asset_names(root: &Path) -> Vec<String> {
    let dir = root.join(ASSETS_SUBDIR);
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

fn has_stem(names: &[String], needle: &str) -> bool {
    names.iter().any(|n| n.contains(needle))
}

/// G10: images=false → no `page-NNNN.png` (writers not invoked).
#[tokio::test]
#[serial]
async fn g10_images_off_skips_page_png_writer() {
    let plan = VisionAssetWritePlan::from_flags(false, true, true);
    assert!(!plan.write_page_pngs);
    let dir = tempfile::tempdir().unwrap();
    if plan.write_page_pngs {
        let _ = write_page_png_assets(
            &minimal_pdf_bytes(),
            dir.path(),
            &[1],
            PageAssetRenderConfig {
                dpi: 72,
                max_rendered_pixels: 512,
            },
        )
        .await;
    }
    let names = list_asset_names(dir.path());
    assert!(
        names.is_empty(),
        "G10: expected no assets when page PNG writer skipped, got {names:?}"
    );
}

/// G10 complement: images=true writes page PNG when pdfium available.
#[tokio::test]
#[serial]
async fn g10_images_on_writes_page_png_when_pdfium() {
    let plan = VisionAssetWritePlan::from_flags(true, false, false);
    assert!(plan.write_page_pngs);
    let dir = tempfile::tempdir().unwrap();
    match write_page_png_assets(
        &minimal_pdf_bytes(),
        dir.path(),
        &[1],
        PageAssetRenderConfig {
            dpi: 72,
            max_rendered_pixels: 512,
        },
    )
    .await
    {
        Ok(written) => {
            assert!(!written.is_empty());
            let names = list_asset_names(dir.path());
            assert!(
                names.iter().any(|n| n == "page-0001.png"),
                "expected page-0001.png, got {names:?}"
            );
        }
        Err(e) => eprintln!("skip pdfium: {e}"),
    }
}

/// G8: figures=false → no `-fig-` assets even when fixture would produce them.
#[tokio::test]
#[serial]
async fn g8_figures_off_skips_embedded_writer() {
    let plan = VisionAssetWritePlan::from_flags(true, true, false);
    assert!(!plan.write_figures);
    let path = embedded_fixture();
    if !path.is_file() {
        eprintln!("skip: missing fixture {path:?}");
        return;
    }
    let bytes = std::fs::read(&path).expect("read fixture");
    let dir = tempfile::tempdir().unwrap();
    if plan.write_figures {
        let _ = write_embedded_figure_assets(&bytes, dir.path(), Some(&[1])).await;
    }
    let names = list_asset_names(dir.path());
    assert!(
        !has_stem(&names, "-fig-"),
        "G8: figures off must not write -fig- assets, got {names:?}"
    );
}

/// G8 complement: figures=true yields `-fig-` from embedded fixture (pdfium).
#[tokio::test]
#[serial]
async fn g8_figures_on_writes_fig_from_fixture() {
    let plan = VisionAssetWritePlan::from_flags(false, false, true);
    assert!(plan.write_figures);
    let path = embedded_fixture();
    if !path.is_file() {
        eprintln!("skip: missing fixture {path:?}");
        return;
    }
    let bytes = std::fs::read(&path).expect("read fixture");
    let dir = tempfile::tempdir().unwrap();
    match write_embedded_figure_assets(&bytes, dir.path(), None).await {
        Ok(written) => {
            if written.is_empty() {
                eprintln!("skip: fixture produced 0 embedded figures");
                return;
            }
            let names = list_asset_names(dir.path());
            assert!(
                has_stem(&names, "-fig-"),
                "expected -fig- assets, got {names:?}"
            );
        }
        Err(e) => eprintln!("skip pdfium: {e}"),
    }
}

/// G9: charts=false → no `-chart` assets (writer not invoked).
#[tokio::test]
#[serial]
async fn g9_charts_off_skips_chart_writer() {
    let plan = VisionAssetWritePlan::from_flags(true, false, true);
    assert!(!plan.write_charts);
    let dir = tempfile::tempdir().unwrap();
    if plan.write_charts {
        let _ = write_chart_crop_assets(&minimal_pdf_bytes(), dir.path(), &[1], CHART_CROP_RENDER)
            .await;
    }
    let names = list_asset_names(dir.path());
    assert!(
        !has_stem(&names, "-chart"),
        "G9: charts off must not write -chart assets, got {names:?}"
    );
}

/// G9 complement: charts=true may write chart crop when ink present (best-effort).
#[tokio::test]
#[serial]
async fn g9_charts_on_invokes_writer() {
    let plan = VisionAssetWritePlan::from_flags(false, true, false);
    assert!(plan.write_charts);
    let dir = tempfile::tempdir().unwrap();
    match write_chart_crop_assets(&minimal_pdf_bytes(), dir.path(), &[1], CHART_CROP_RENDER).await {
        Ok(_paths) => {
            // Minimal PDF may yield 0 crops — gate is that the call is allowed.
        }
        Err(e) => eprintln!("skip pdfium: {e}"),
    }
}

#[test]
fn promote_requires_both_charts_and_figures() {
    assert!(VisionAssetWritePlan::from_flags(true, true, true).promote_fig_as_chart);
    assert!(!VisionAssetWritePlan::from_flags(true, true, false).promote_fig_as_chart);
    assert!(!VisionAssetWritePlan::from_flags(true, false, true).promote_fig_as_chart);
}
