//! SPEC-098 F-098-12 / LAW-098-9: single delete admit sets KV `deleting`.
//!
//! Merge-unit coverage lives in `document_read_model` tests
//! (`merge_keeps_kv_deleting_over_stale_relational_completed`).

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::{json, Value};
use tower::ServiceExt;

const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
const TEST_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000002";

fn with_tenant_scope(mut base: Value) -> Value {
    if let Some(obj) = base.as_object_mut() {
        obj.insert("tenant_id".into(), json!(TEST_TENANT_ID));
        obj.insert("workspace_id".into(), json!(TEST_WORKSPACE_ID));
    }
    base
}

#[tokio::test]
async fn e2e_spec098_single_delete_admit_kv_deleting() {
    let state = AppState::test_state();
    let kv = state.storage.kv_storage.clone();
    let doc_id = "spec098-del-admit-1";
    kv.upsert(&[(
        format!("{doc_id}-metadata"),
        with_tenant_scope(json!({
            "id": doc_id,
            "title": "admit.pdf",
            "status": "completed",
            "created_at": "2026-01-01T00:00:00Z",
        })),
    )])
    .await
    .expect("seed");

    let app = Server::new(
        ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            enable_cors: false,
            enable_compression: false,
            enable_swagger: true,
        },
        state,
    )
    .build_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let meta = kv
        .get_by_id(&format!("{doc_id}-metadata"))
        .await
        .ok()
        .flatten()
        .expect("metadata still present mid-delete");
    assert_eq!(
        meta.get("status").and_then(|v| v.as_str()),
        Some("deleting"),
        "single delete admit must dual-write KV deleting"
    );
}
