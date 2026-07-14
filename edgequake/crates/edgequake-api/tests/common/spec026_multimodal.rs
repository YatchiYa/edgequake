//! SPEC-026 Phase 4 multimodal E2E fixtures and helpers (DRY SSOT).
//!
//! LightRAG parity: mock VLM JSON matches `prompt_multimodal.py` image_analysis schema.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use edgequake_pdf::{
    assemble_vision_markdown_with_figures, page_asset_rel_path, page_figure_asset_rel_path,
    page_table_asset_rel_path, VisionPageSlice, WrittenFigureAsset, WrittenTableAsset,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Minimal valid 1×1 PNG (67 bytes).
pub const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// Mock VLM response (LightRAG `{name,type,description}` schema).
pub const MOCK_VLM_SARAH_JSON: &str = r#"{"name":"sarah_chen_profile","type":"Photo","description":"Dr. Sarah Chen leads EdgeQuake research in Zurich."}"#;

/// Mock VLM classify response routing to chart specialize (SPEC-047 Phase B/C).
pub const MOCK_VLM_CHART_CLASSIFY_JSON: &str =
    r#"{"name":"rev","type":"Chart","description":"generic chart"}"#;

/// Mock chart specialize JSON with a searchable key value (SPEC-047 G-C probe).
pub const MOCK_VLM_CHART_SPECIALIZE_JSON: &str = r#"{"name":"rev_q4","chart_kind":"bar","title":"Q4 Revenue","x_axis":"Quarter","y_axis":"USD M","key_values":[{"label":"Q4","value_raw":"42"}],"series":[{"name":"Revenue","values":[{"x":"Q4","y_raw":"42"}]}],"data_table_md":"| Quarter | Value |\n|---|---|\n| Q4 | 42 |","description":"Revenue rose."}"#;

/// Classify as Illustration; caption context must still route to chart specialize (015).
pub const MOCK_VLM_FIGURE_MISROUTE_CLASSIFY_JSON: &str =
    r#"{"name":"rev_fig","type":"Illustration","description":"generic figure"}"#;

/// Multi-panel line-chart grid specialize (research-paper Figure 1 layout).
pub const MOCK_VLM_MULTI_PANEL_CHART_SPECIALIZE_JSON: &str = r#"{"name":"capability_grid","chart_kind":"line","title":"Model performance across dimensions","x_axis":"tokens(B)","y_axis":"score","key_values":[{"label":"Average | full data | 10B","value_raw":"52"},{"label":"Average | w/o code | 10B","value_raw":"41"},{"label":"Mathematical Ability | w/o math | 10B","value_raw":"38"}],"series":[{"name":"full data","values":[{"x":"0","y_raw":"20"},{"x":"10","y_raw":"52"}]}],"data_table_md":"| Panel | Series | tokens(B) | score |\n|---|---|---|---|\n| Average | full data | 10 | 52 |\n| Average | w/o code | 10 | 41 |","description":"Six-panel grid of line charts."}"#;

/// Register classify-as-Illustration + multi-panel chart specialize responses.
pub async fn mock_multi_panel_figure_chart_vlm_responses(mock: &edgequake_llm::MockProvider) {
    mock.add_response(MOCK_VLM_FIGURE_MISROUTE_CLASSIFY_JSON)
        .await;
    mock.add_response(MOCK_VLM_MULTI_PANEL_CHART_SPECIALIZE_JSON)
        .await;
}

/// Write a page PNG asset at `{assets_root}/assets/page-NNNN.png` (viewer).
/// Also writes `page-NNNN-fig-01.png` so analyze tests that use
/// [`vision_page_markdown`] resolve figure-bounded drawings.
pub fn write_page_png_asset(assets_root: &Path, page_num: usize) {
    let rel = page_asset_rel_path(page_num);
    let full = assets_root.join(&rel);
    std::fs::create_dir_all(full.parent().expect("asset parent")).unwrap_or_else(|e| {
        panic!("create asset dir for page {page_num}: {e}");
    });
    std::fs::write(&full, TINY_PNG).unwrap_or_else(|e| {
        panic!("write page png asset {full:?}: {e}");
    });
    write_figure_png_asset(assets_root, page_num);
}

/// Write a figure-bounded PNG (`assets/page-NNNN-fig-01.png`) for VLM analyze tests.
pub fn write_figure_png_asset(assets_root: &Path, page_num: usize) {
    let rel = page_figure_asset_rel_path(page_num, 1);
    let full = assets_root.join(&rel);
    std::fs::create_dir_all(full.parent().expect("asset parent")).unwrap_or_else(|e| {
        panic!("create figure asset dir for page {page_num}: {e}");
    });
    std::fs::write(&full, TINY_PNG).unwrap_or_else(|e| {
        panic!("write figure png asset {full:?}: {e}");
    });
}

/// Write a table-crop PNG (`assets/page-NNNN-table-01.png`) for VLM / viewer tests.
pub fn write_table_png_asset(assets_root: &Path, page_num: usize) {
    let rel = page_table_asset_rel_path(page_num, 1);
    let full = assets_root.join(&rel);
    std::fs::create_dir_all(full.parent().expect("asset parent")).unwrap_or_else(|e| {
        panic!("create table asset dir for page {page_num}: {e}");
    });
    std::fs::write(&full, TINY_PNG).unwrap_or_else(|e| {
        panic!("write table png asset {full:?}: {e}");
    });
}

/// Vision markdown with table-crop Drawing (no fig invent).
pub fn vision_table_page_markdown(document_id: &str, page_num: usize, body: &str) -> String {
    let pages = vec![VisionPageSlice {
        page_num,
        markdown: body.to_string(),
    }];
    let mut tables = HashMap::new();
    tables.insert(
        page_num,
        vec![WrittenTableAsset {
            page_num,
            index: 1,
            rel_path: page_table_asset_rel_path(page_num, 1),
            width: 80,
            height: 40,
            label: "Table 1".into(),
        }],
    );
    assemble_vision_markdown_with_figures(
        &pages,
        true,
        true,
        Some(document_id),
        None,
        None,
        Some(&tables),
    )
}

/// Vision markdown with page markers + figure-bounded `<drawing path="…-fig-01.png"/>`.
pub fn vision_page_markdown(document_id: &str, pages: &[(usize, &str)]) -> String {
    let slices: Vec<VisionPageSlice> = pages
        .iter()
        .map(|(page_num, body)| VisionPageSlice {
            page_num: *page_num,
            markdown: (*body).to_string(),
        })
        .collect();
    let mut figs: HashMap<usize, Vec<WrittenFigureAsset>> = HashMap::new();
    for (page_num, _) in pages {
        figs.insert(
            *page_num,
            vec![WrittenFigureAsset {
                page_num: *page_num,
                index: 1,
                rel_path: page_figure_asset_rel_path(*page_num, 1),
                width: 40,
                height: 30,
                bbox: None,
            }],
        );
    }
    assemble_vision_markdown_with_figures(
        &slices,
        true,
        true,
        Some(document_id),
        None,
        Some(&figs),
        None,
    )
}

/// Register mock VLM responses for chart classify → specialize (two-step).
pub async fn mock_chart_vlm_responses(mock: &edgequake_llm::MockProvider) {
    mock.add_response(MOCK_VLM_CHART_CLASSIFY_JSON).await;
    mock.add_response(MOCK_VLM_CHART_SPECIALIZE_JSON).await;
}

/// Illustration classify + chart specialize (context/caption chart routing).
pub async fn mock_figure_caption_chart_vlm_responses(mock: &edgequake_llm::MockProvider) {
    mock.add_response(MOCK_VLM_FIGURE_MISROUTE_CLASSIFY_JSON)
        .await;
    mock.add_response(MOCK_VLM_CHART_SPECIALIZE_JSON).await;
}

/// Base64 of [`TINY_PNG`] for data-URI markdown fixtures.
pub const TINY_PNG_B64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

/// Enable inline VLM analyze (explicit `VLM_PROCESS_ENABLE=true`; default is on).
pub fn enable_vlm_process_in_tests() {
    std::env::set_var("VLM_PROCESS_ENABLE", "true");
}

/// Disable inline VLM analyze for tests that assert Pass B skip behavior.
pub fn disable_vlm_process_in_tests() {
    std::env::set_var("VLM_PROCESS_ENABLE", "false");
}

/// Lower pixel gate so 1×1 fixture PNG can reach mock VLM in E2E.
pub fn allow_tiny_images_in_tests() {
    enable_vlm_process_in_tests();
    std::env::set_var("VLM_MIN_IMAGE_PIXEL", "1");
}

pub fn restore_vlm_image_limits() {
    std::env::remove_var("VLM_MIN_IMAGE_PIXEL");
    std::env::remove_var("VLM_PROCESS_ENABLE");
}

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spec026")
        .join(name)
}

pub fn load_fixture_utf8(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("missing fixture {name}: {e}"))
}

/// Build multipart body for PNG upload to `/documents/upload`.
pub fn build_png_multipart(boundary: &str, filename: &str) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(TINY_PNG);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

/// POST multipart PNG to `/documents/upload`.
pub fn png_upload_request(boundary: &str, filename: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/documents/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(build_png_multipart(boundary, filename)))
        .unwrap()
}

/// POST JSON text document to `/api/v1/documents`.
pub fn text_upload_request(title: &str, content: &str) -> Request<Body> {
    let body = serde_json::json!({
        "content": content,
        "title": title,
    });
    Request::builder()
        .method("POST")
        .uri("/api/v1/documents")
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// Markdown simulating post-convert PDF with embedded data-URI (LightRAG drawing surrogate).
pub fn markdown_with_data_uri_image() -> String {
    format!(
        "# Report\n\nSee chart:\n\n![inline chart](data:image/png;base64,{TINY_PNG_B64})\n\nEnd.\n"
    )
}

/// Markdown with LightRAG native `<drawing/>` placeholder (no sidecar asset yet).
pub fn markdown_with_drawing_tag() -> String {
    "# Report\n\n<drawing id=\"im-spec026-0001\" format=\"png\" caption=\"Chart\" />\n\nEnd.\n"
        .to_string()
}

pub async fn response_body_bytes(response: Response) -> axum::body::Bytes {
    axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap()
}

/// Parse a 202 Accepted upload response.
pub async fn parse_accepted_upload(response: Response) -> (String, String) {
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let parsed: Value = serde_json::from_slice(&response_body_bytes(response).await).unwrap();
    (
        parsed["document_id"].as_str().unwrap().to_string(),
        parsed["track_id"].as_str().unwrap().to_string(),
    )
}
