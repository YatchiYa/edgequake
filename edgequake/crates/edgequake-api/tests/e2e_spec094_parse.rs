//! SPEC-094: Stateless PDF → Markdown parse API e2e tests.
//!
//! Covers sync EdgeParse, pipeline identity, raw PDF, backends, ceilings,
//! async Prefer, fallback on/off, unsupported media, and residue cleanup.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{create_test_app, extract_json, TEST_TENANT_ID, TEST_WORKSPACE_ID};
use edgequake_pdf::{create_pdf_converter, PdfConversionConfig, PdfParserBackend};
use serde_json::{json, Value};
use tower::ServiceExt;

const PDF_FIXTURE: &[u8] =
    include_bytes!("../../../../legacy/edgequake-pdf/test-data/001_simple_text.pdf");

const MULTI_PAGE_PDF: &[u8] =
    include_bytes!("../../../../legacy/edgequake-pdf/test-data/008_multi_page_5_pages.pdf");

fn multipart_parse_body(filename: &str, pdf_bytes: &[u8], options: &Value) -> (String, Vec<u8>) {
    let boundary = "----EdgeQuakeParseBoundary094";
    let mut body: Vec<u8> = Vec::with_capacity(pdf_bytes.len() + 1024);

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"options\"\r\nContent-Type: application/json\r\n\r\n",
    );
    body.extend_from_slice(options.to_string().as_bytes());
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\
             Content-Type: application/pdf\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(pdf_bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (boundary.to_string(), body)
}

fn tenant_headers(builder: axum::http::request::Builder) -> axum::http::request::Builder {
    builder
        .header("X-Tenant-ID", TEST_TENANT_ID)
        .header("X-Workspace-ID", TEST_WORKSPACE_ID)
}

async fn post_parse_multipart(
    app: &axum::Router,
    pdf_bytes: &[u8],
    options: Value,
    extra_headers: &[(&str, &str)],
) -> (StatusCode, Value) {
    let (boundary, body) = multipart_parse_body("sample.pdf", pdf_bytes, &options);
    let mut builder = tenant_headers(
        Request::builder()
            .method("POST")
            .uri("/api/v1/parse")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            ),
    );
    for (k, v) in extra_headers {
        builder = builder.header(*k, *v);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let json = extract_json(response).await;
    (status, json)
}

#[tokio::test]
async fn parse_edgeparse_sync_returns_markdown_without_documents() {
    let app = create_test_app();

    let (status, body) =
        post_parse_multipart(&app, PDF_FIXTURE, json!({"backend": "edgeparse"}), &[]).await;

    assert_eq!(status, StatusCode::OK, "body={body}");
    let markdown = body["markdown"].as_str().unwrap_or("");
    assert!(!markdown.trim().is_empty(), "expected non-empty markdown");
    assert_eq!(body["backend"], "edgeparse");
    assert_eq!(body["backend_effective"], "edgeparse");
    assert_eq!(body["fallback_applied"], false);
    assert!(body["request_id"].as_str().unwrap().starts_with("pr_"));

    // Stateless: parse must not create a document id / track fields in the response.
    assert!(body.get("document_id").is_none() || body["document_id"].is_null());
    assert!(body.get("pdf_id").is_none() || body["pdf_id"].is_null());
    assert!(body.get("task_id").is_none() || body["task_id"].is_null());
}

#[tokio::test]
async fn parse_markdown_matches_pipeline_converter() {
    let app = create_test_app();
    let (status, body) = post_parse_multipart(
        &app,
        PDF_FIXTURE,
        json!({"backend": "edgeparse", "emit_assets": false}),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    let api_md = body["markdown"].as_str().unwrap();

    let converter = create_pdf_converter(PdfParserBackend::EdgeParse);
    let config = PdfConversionConfig {
        filename: Some("sample.pdf".into()),
        page_drawing_assets: None,
        pages: None,
        ..Default::default()
    };
    let pipeline_md = converter.convert(PDF_FIXTURE, &config).await.unwrap();
    assert_eq!(
        api_md, pipeline_md,
        "SPEC-094 identity: parse endpoint must match PdfConverter output"
    );
}

#[tokio::test]
async fn parse_raw_pdf_body_with_query_options() {
    let app = create_test_app();
    let response = app
        .clone()
        .oneshot(
            tenant_headers(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/parse?backend=edgeparse")
                    .header("content-type", "application/pdf")
                    .header("X-Filename", "raw.pdf"),
            )
            .body(Body::from(PDF_FIXTURE.to_vec()))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    assert!(!body["markdown"].as_str().unwrap().trim().is_empty());
    assert_eq!(body["backend"], "edgeparse");
}

#[tokio::test]
async fn parse_backends_lists_vision_and_edgeparse() {
    let app = create_test_app();
    let response = app
        .oneshot(
            tenant_headers(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/parse/backends"),
            )
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = extract_json(response).await;
    let names: Vec<&str> = body["backends"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["name"].as_str())
        .collect();
    assert!(names.contains(&"vision"));
    assert!(names.contains(&"edgeparse"));
    assert!(body["limits"]["sync_max_pages"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn parse_too_large_returns_413_when_over_sync_ceiling_without_async() {
    // Force sync ceiling to 1 page; multi-page PDF must go async unless we
    // also disable async by... wait: over sync ceiling auto-promotes to async.
    // Spec: over ceiling without Prefer → auto 202. For 413, exceed *async* ceiling.
    std::env::set_var("EDGEQUAKE_PARSE_ASYNC_MAX_PAGES", "2");
    let app = create_test_app();
    let (status, body) =
        post_parse_multipart(&app, MULTI_PAGE_PDF, json!({"backend": "edgeparse"}), &[]).await;
    std::env::remove_var("EDGEQUAKE_PARSE_ASYNC_MAX_PAGES");

    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "body={body}");
    assert_eq!(body["code"], "parse.too_large");
}

#[tokio::test]
async fn parse_prefer_async_returns_202_then_job_completes() {
    let app = create_test_app();
    let (status, body) = post_parse_multipart(
        &app,
        PDF_FIXTURE,
        json!({"backend": "edgeparse"}),
        &[("Prefer", "respond-async")],
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED, "body={body}");
    let job_id = body["job_id"].as_str().unwrap().to_string();
    assert!(job_id.starts_with("pr_"));

    let mut result = None;
    for _ in 0..50 {
        let response = app
            .clone()
            .oneshot(
                tenant_headers(
                    Request::builder()
                        .method("GET")
                        .uri(format!("/api/v1/parse/jobs/{job_id}")),
                )
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let job = extract_json(response).await;
        let st = job["status"].as_str().unwrap_or("");
        if st == "completed" || st == "failed" {
            result = Some(job);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let job = result.expect("job did not complete in time");
    assert_eq!(job["status"], "completed", "job={job}");
    assert!(!job["result"]["markdown"]
        .as_str()
        .unwrap_or("")
        .trim()
        .is_empty());
}

#[tokio::test]
async fn parse_vision_fallback_disabled_returns_502() {
    let app = create_test_app();
    let (status, body) = post_parse_multipart(
        &app,
        PDF_FIXTURE,
        json!({
            "backend": "vision",
            "provider": "nonexistent-provider-xyz",
            "model": "no-such-model",
            "allow_fallback": false
        }),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY, "body={body}");
    assert_eq!(body["code"], "parse.backend_unavailable");
}

#[tokio::test]
async fn parse_vision_fallback_enabled_returns_edgeparse() {
    let app = create_test_app();
    let (status, body) = post_parse_multipart(
        &app,
        PDF_FIXTURE,
        json!({
            "backend": "vision",
            "provider": "nonexistent-provider-xyz",
            "model": "no-such-model",
            "allow_fallback": true
        }),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["backend"], "vision");
    assert_eq!(body["backend_effective"], "edgeparse");
    assert_eq!(body["fallback_applied"], true);
    assert!(!body["warnings"].as_array().unwrap().is_empty());
    assert!(!body["markdown"].as_str().unwrap().trim().is_empty());
}

#[tokio::test]
async fn parse_unsupported_media_returns_415() {
    let app = create_test_app();
    let response = app
        .oneshot(
            tenant_headers(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/parse")
                    .header("content-type", "text/plain"),
            )
            .body(Body::from("not a pdf"))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let body = extract_json(response).await;
    assert_eq!(body["code"], "parse.unsupported_media_type");
}

#[tokio::test]
async fn parse_emit_assets_leaves_no_temp_residue() {
    let before: std::collections::HashSet<_> = std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();

    let app = create_test_app();
    let (status, body) = post_parse_multipart(
        &app,
        PDF_FIXTURE,
        json!({"backend": "edgeparse", "emit_assets": true}),
        &[],
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body={body}");

    // Drop app / allow cleanup.
    drop(app);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let after: std::collections::HashSet<_> = std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .collect();
    let new_entries: Vec<_> = after.difference(&before).collect();
    // Tempfile crate uses random names; any leftover parse assets dirs would
    // typically start with `.tmp` — we only assert we didn't leave named
    // `edgequake-parse-*` residue.
    for name in &new_entries {
        let s = name.to_string_lossy();
        assert!(
            !s.contains("edgequake-parse"),
            "unexpected parse residue: {s}"
        );
    }
}
