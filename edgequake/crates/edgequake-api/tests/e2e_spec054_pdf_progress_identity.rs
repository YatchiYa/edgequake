//! SPEC-054 / GitHub #300 — PDF progress identity contract (E2E).
//!
//! Invariant: admitted PDF jobs seed and expose progress under server
//! `task_id` only. Client `track_id` is batch correlation and must not create
//! a pending progress skeleton.
//!
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! cargo test -p edgequake-api --features postgres --test e2e_spec054_pdf_progress_identity -- --nocapture
//! ```

#![cfg(feature = "postgres")]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::extract_json;
use common::spec013_postgres;
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;

const PDF_FIXTURE_A: &[u8] =
    include_bytes!("../../../../legacy/edgequake-pdf/test-data/001_simple_text.pdf");
const PDF_FIXTURE_B: &[u8] =
    include_bytes!("../../../../legacy/edgequake-pdf/test-data/025_rotated_text.pdf");

fn multipart_pdf_upload_body(
    filename: &str,
    pdf_bytes: &[u8],
    fields: &[(&str, &str)],
) -> (String, Vec<u8>) {
    let boundary = format!("----EdgeQuakeSpec054-{}", Uuid::new_v4().simple());
    let mut body: Vec<u8> = Vec::with_capacity(pdf_bytes.len() + 1024);

    for (k, v) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes(),
        );
    }

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

    (boundary, body)
}

async fn create_tenant_workspace(app: &axum::Router) -> (String, String) {
    let suffix = Uuid::new_v4();
    let tenant = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tenants")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "name": format!("spec054 tenant {suffix}") }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tenant.status(), StatusCode::CREATED);
    let tenant_json = extract_json(tenant).await;
    let tenant_id = tenant_json["id"].as_str().unwrap().to_string();

    let ws = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/tenants/{tenant_id}/workspaces"))
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", &tenant_id)
                .body(Body::from(json!({ "name": "spec054 ws" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ws.status(), StatusCode::CREATED);
    let ws_json = extract_json(ws).await;
    let workspace_id = ws_json["id"].as_str().unwrap().to_string();
    (tenant_id, workspace_id)
}

async fn upload_pdf(
    app: &axum::Router,
    tenant_id: &str,
    workspace_id: &str,
    filename: &str,
    pdf_bytes: &[u8],
    fields: &[(&str, &str)],
) -> serde_json::Value {
    let (boundary, body) = multipart_pdf_upload_body(filename, pdf_bytes, fields);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/pdf")
                .header(
                    "Content-Type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .header("X-Tenant-ID", tenant_id)
                .header("X-User-ID", common::TEST_USER_ID)
                .header("X-Workspace-ID", workspace_id)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "pdf upload failed: {:?}",
        extract_json(response).await
    );
    extract_json(response).await
}

async fn progress_status(app: &axum::Router, track_id: &str) -> StatusCode {
    app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/documents/pdf/progress/{track_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

#[tokio::test]
#[serial]
async fn spec054_progress_keyed_by_task_id_not_client_batch() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let (tenant_id, workspace_id) = create_tenant_workspace(&app).await;
    let client_batch = format!("upload_batch_{}", Uuid::new_v4().simple());

    let body = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-a.pdf",
        PDF_FIXTURE_A,
        &[
            ("title", "spec054-identity"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
            ("track_id", &client_batch),
        ],
    )
    .await;

    let task_id = body["task_id"].as_str().expect("task_id").to_string();
    assert!(!task_id.is_empty(), "admitted PDF requires task_id");
    assert!(
        task_id.starts_with("pdf-"),
        "task_id should be server pdf-* id, got {task_id}"
    );
    assert_ne!(task_id, client_batch);
    assert_eq!(body["track_id"].as_str(), Some(client_batch.as_str()));

    assert_eq!(
        progress_status(&app, &task_id).await,
        StatusCode::OK,
        "progress must exist under server task_id"
    );
    assert_eq!(
        progress_status(&app, &client_batch).await,
        StatusCode::NOT_FOUND,
        "client batch id must not seed progress (GitHub #300)"
    );
}

#[tokio::test]
#[serial]
async fn spec054_upload_without_client_track_id_still_seeds_task_progress() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let (tenant_id, workspace_id) = create_tenant_workspace(&app).await;

    let body = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-no-batch.pdf",
        PDF_FIXTURE_A,
        &[
            ("title", "spec054-no-batch"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
        ],
    )
    .await;

    let task_id = body["task_id"].as_str().expect("task_id").to_string();
    assert!(!task_id.is_empty());
    assert_eq!(progress_status(&app, &task_id).await, StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn spec054_same_batch_id_two_pdfs_get_isolated_task_progress() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let (tenant_id, workspace_id) = create_tenant_workspace(&app).await;
    let client_batch = format!("shared_batch_{}", Uuid::new_v4().simple());

    let a = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-batch-a.pdf",
        PDF_FIXTURE_A,
        &[
            ("title", "spec054-batch-a"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
            ("track_id", &client_batch),
        ],
    )
    .await;
    let b = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-batch-b.pdf",
        PDF_FIXTURE_B,
        &[
            ("title", "spec054-batch-b"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
            ("track_id", &client_batch),
        ],
    )
    .await;

    let task_a = a["task_id"].as_str().expect("task_a").to_string();
    let task_b = b["task_id"].as_str().expect("task_b").to_string();
    assert_ne!(task_a, task_b, "each PDF job must have its own task_id");
    assert_eq!(a["track_id"].as_str(), Some(client_batch.as_str()));
    assert_eq!(b["track_id"].as_str(), Some(client_batch.as_str()));

    assert_eq!(progress_status(&app, &task_a).await, StatusCode::OK);
    assert_eq!(progress_status(&app, &task_b).await, StatusCode::OK);
    assert_eq!(
        progress_status(&app, &client_batch).await,
        StatusCode::NOT_FOUND,
        "shared batch id must not own either job's progress"
    );
}

#[tokio::test]
#[serial]
async fn spec054_duplicate_without_reindex_does_not_seed_progress() {
    let Some(app) = spec013_postgres::create_postgres_mock_app_or_skip().await else {
        eprintln!("SKIP: no PostgreSQL DATABASE_URL configured");
        return;
    };
    let (tenant_id, workspace_id) = create_tenant_workspace(&app).await;
    let phantom_batch = format!("dup_batch_{}", Uuid::new_v4().simple());

    let first = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-dup.pdf",
        PDF_FIXTURE_A,
        &[
            ("title", "spec054-dup-first"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
        ],
    )
    .await;
    let first_task = first["task_id"].as_str().unwrap().to_string();
    assert_eq!(progress_status(&app, &first_task).await, StatusCode::OK);

    let dup = upload_pdf(
        &app,
        &tenant_id,
        &workspace_id,
        "spec054-dup.pdf",
        PDF_FIXTURE_A,
        &[
            ("title", "spec054-dup-second"),
            ("enable_vision", "false"),
            ("pdf_parser_backend", "text"),
            ("track_id", &phantom_batch),
        ],
    )
    .await;

    assert_eq!(dup["status"].as_str(), Some("duplicate"));
    assert_eq!(dup["task_id"].as_str().unwrap_or(""), "");
    assert_eq!(
        progress_status(&app, &phantom_batch).await,
        StatusCode::NOT_FOUND,
        "duplicate without new task must not seed progress under client id"
    );
}
