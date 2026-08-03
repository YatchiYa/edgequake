//! SPEC-098 F-098-14: batch delete admit sets per-doc KV `deleting`.

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
async fn e2e_spec098_batch_delete_admit_deleting() {
    let state = AppState::test_state();
    let kv = state.storage.kv_storage.clone();

    let mut delete_ids = Vec::new();
    let keep_id = "spec098-batch-keep";
    kv.upsert(&[(
        format!("{keep_id}-metadata"),
        with_tenant_scope(json!({
            "id": keep_id,
            "title": "Keep",
            "status": "completed",
            "created_at": "2026-01-01T00:00:00Z",
        })),
    )])
    .await
    .expect("seed keep");

    for i in 0..3 {
        let id = format!("spec098-batch-del-{i}");
        kv.upsert(&[(
            format!("{id}-metadata"),
            with_tenant_scope(json!({
                "id": id,
                "title": format!("Del {i}"),
                "status": "completed",
                "created_at": "2026-01-02T00:00:00Z",
            })),
        )])
        .await
        .expect("seed del");
        delete_ids.push(id);
    }

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
                .method("POST")
                .uri("/api/v1/documents/batch-delete")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::from(
                    serde_json::to_string(&json!({ "document_ids": delete_ids })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    for id in &delete_ids {
        let meta = kv
            .get_by_id(&format!("{id}-metadata"))
            .await
            .ok()
            .flatten()
            .expect("metadata present after admit");
        assert_eq!(
            meta.get("status").and_then(|v| v.as_str()),
            Some("deleting"),
            "batch admit must set KV deleting for {id}"
        );
    }

    let keep = kv
        .get_by_id(&format!("{keep_id}-metadata"))
        .await
        .ok()
        .flatten()
        .expect("keep present");
    assert_eq!(
        keep.get("status").and_then(|v| v.as_str()),
        Some("completed"),
        "unselected must remain completed"
    );
}
