//! Endpoint sweep F9 residual — never advertise a job/task id that 404s.
//!
//! Batch labels (`recover_…`, `reprocess_…`, `rebuild_…`) have no task row.
//! v1 RPCs must point `Location` at a real task (or return 200 without one);
//! v2 `POST .../jobs` must return a `job_id` that `GET`/`DELETE` resolve.
//!
//! Memory-mode (`AppState::test_state()`), CI-safe.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_core::types::CreateWorkspaceRequest;
use edgequake_core::Tenant;
use edgequake_storage::kv_keys;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    }
}

async fn create_tenant_workspace(state: &AppState) -> (Uuid, Uuid) {
    let tenant = Tenant::new("f9", format!("f9-{}", Uuid::new_v4()));
    let created = state
        .workspace_service
        .create_tenant(tenant)
        .await
        .expect("tenant");
    let ws = state
        .workspace_service
        .create_workspace(
            created.tenant_id,
            CreateWorkspaceRequest {
                name: "f9 ws".to_string(),
                slug: Some(format!("f9-{}", Uuid::new_v4())),
                ..Default::default()
            },
        )
        .await
        .expect("workspace");
    (created.tenant_id, ws.workspace_id)
}

async fn seed_doc(state: &AppState, doc_id: &str, status: &str, tenant: Uuid, ws: Uuid) {
    edgequake_api::services::upsert_metadata_kv_with_index(
        state.storage.kv_storage.as_ref(),
        &kv_keys::doc_metadata(doc_id),
        json!({
            "id": doc_id,
            "title": "f9 doc",
            "status": status,
            "updated_at": "2020-01-01T00:00:00Z",
            "tenant_id": tenant.to_string(),
            "workspace_id": ws.to_string(),
            "source_type": "text",
            "chunk_count": 1,
        }),
    )
    .await
    .expect("seed metadata");
    state
        .storage
        .kv_storage
        .upsert(&[(
            format!("{doc_id}-content"),
            json!({ "content": "Body for F9." }),
        )])
        .await
        .unwrap();
}

/// (status, Location, body)
async fn call(
    state: &AppState,
    method: &str,
    uri: &str,
    tenant: Uuid,
    ws: Uuid,
    body: Option<Value>,
) -> (StatusCode, Option<String>, Value) {
    let app = Server::new(test_config(), state.clone()).build_router();
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-ID", tenant.to_string())
        .header("X-Workspace-ID", ws.to_string());
    if body.is_some() {
        req = req.header("Content-Type", "application/json");
    }
    let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));
    let resp = app.oneshot(req.body(body).unwrap()).await.unwrap();
    let status = resp.status();
    let location = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        location,
        serde_json::from_slice(&bytes).unwrap_or(json!({})),
    )
}

/// 202 Location must be GET-able; it must be the first real task id.
async fn assert_location_resolves(
    state: &AppState,
    tenant: Uuid,
    ws: Uuid,
    location: Option<String>,
    expected_task: Option<&str>,
    body: &Value,
) {
    let location = location.unwrap_or_else(|| panic!("202 needs Location: {body}"));
    let expected = expected_task.unwrap_or_else(|| panic!("task id in body: {body}"));
    assert_eq!(location, format!("/api/v1/tasks/{expected}"), "{body}");
    let (st, _, task) = call(state, "GET", &location, tenant, ws, None).await;
    assert_eq!(st, StatusCode::OK, "Location must resolve: {task}");
}

#[tokio::test]
async fn f9_workspace_reprocess_documents_location_is_real_task() {
    let state = AppState::test_state();
    let (tenant, ws) = create_tenant_workspace(&state).await;
    let uri = format!("/api/v1/workspaces/{ws}/reprocess-documents");

    // Nothing to queue → 200, no Location.
    let (st, loc, body) = call(&state, "POST", &uri, tenant, ws, Some(json!({}))).await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(loc.is_none(), "no task → no Location: {body}");
    assert!(body.get("first_task_id").is_none(), "{body}");
    assert!(body["track_id"].is_string(), "batch label kept: {body}");

    seed_doc(&state, "f9-reprocess-doc", "completed", tenant, ws).await;
    let (st, loc, body) = call(&state, "POST", &uri, tenant, ws, Some(json!({}))).await;
    assert_eq!(st, StatusCode::ACCEPTED, "{body}");
    assert_location_resolves(
        &state,
        tenant,
        ws,
        loc,
        body["first_task_id"].as_str(),
        &body,
    )
    .await;
}

#[tokio::test]
async fn f9_recover_stuck_location_is_real_task() {
    let state = AppState::test_state();
    let (tenant, ws) = create_tenant_workspace(&state).await;
    let req = json!({ "stuck_threshold_minutes": 1 });

    let (st, loc, body) = call(
        &state,
        "POST",
        "/api/v1/documents/recover-stuck",
        tenant,
        ws,
        Some(req.clone()),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{body}");
    assert!(loc.is_none(), "nothing recovered → no Location: {body}");

    seed_doc(&state, "f9-stuck-doc", "processing", tenant, ws).await;
    let (st, loc, body) = call(
        &state,
        "POST",
        "/api/v1/documents/recover-stuck",
        tenant,
        ws,
        Some(req),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{body}");
    let task_id = body["document_task_ids"][0]["task_id"].as_str();
    assert_location_resolves(&state, tenant, ws, loc, task_id, &body).await;
}

#[tokio::test]
async fn f9_v2_noop_jobs_resolve_on_get_and_delete() {
    let state = AppState::test_state();
    let (tenant, ws) = create_tenant_workspace(&state).await;
    let jobs = format!("/api/v2/workspaces/{ws}/jobs");

    for (job_type, payload) in [
        ("reprocess_failed", json!({})),
        ("recover_stuck", json!({})),
        ("reprocess_all", json!({})),
        // Empty workspace: used to 500 ("returned no job_id").
        ("rebuild_embeddings", json!({ "force": true })),
    ] {
        let (st, loc, body) = call(
            &state,
            "POST",
            &jobs,
            tenant,
            ws,
            Some(json!({ "job_type": job_type, "payload": payload })),
        )
        .await;
        assert_eq!(st, StatusCode::ACCEPTED, "{job_type}: {body}");
        let job_id = body["job_id"].as_str().expect("job_id").to_string();
        assert_eq!(loc, Some(format!("{jobs}/{job_id}")), "{job_type}");

        let (st, _, got) = call(&state, "GET", &format!("{jobs}/{job_id}"), tenant, ws, None).await;
        assert_eq!(
            st,
            StatusCode::OK,
            "{job_type}: GET job must resolve: {got}"
        );
        assert_eq!(got["job_type"], job_type, "{got}");
        assert_eq!(got["status"], "indexed", "nothing queued → terminal: {got}");

        let (st, _, del) = call(
            &state,
            "DELETE",
            &format!("{jobs}/{job_id}"),
            tenant,
            ws,
            None,
        )
        .await;
        assert_ne!(
            st,
            StatusCode::NOT_FOUND,
            "{job_type}: DELETE must not 404: {del}"
        );
    }
}

#[tokio::test]
async fn f9_v2_reprocess_all_job_id_is_real_task_when_queued() {
    let state = AppState::test_state();
    let (tenant, ws) = create_tenant_workspace(&state).await;
    seed_doc(&state, "f9-v2-doc", "completed", tenant, ws).await;
    let jobs = format!("/api/v2/workspaces/{ws}/jobs");
    let (st, _, body) = call(
        &state,
        "POST",
        &jobs,
        tenant,
        ws,
        Some(json!({ "job_type": "reprocess_all", "payload": {} })),
    )
    .await;
    assert_eq!(st, StatusCode::ACCEPTED, "{body}");
    let job_id = body["job_id"].as_str().unwrap();
    assert!(
        !job_id.starts_with("reprocess_"),
        "batch label leaked: {body}"
    );
    let task = state.tasks.storage.get_task(job_id).await.unwrap();
    assert!(task.is_some_and(|t| t.status.is_inflight()), "{body}");
}
