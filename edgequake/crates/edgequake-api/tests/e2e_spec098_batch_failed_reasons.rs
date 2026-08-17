//! SPEC-098 F-098-17 / LAW-098-11: batch cascade fail → `failed[{id,reason}]`
//! and KV `delete_failed`.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_tasks::TaskStatus;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
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
async fn e2e_spec098_batch_failed_reasons() {
    let workers = common::create_test_app_with_workers().await;
    let app = workers.app();
    let kv = &workers.kv_storage;
    let graph = &workers.graph_storage;

    let doc_id = "spec098-batch-fail-reason-1";
    kv.upsert(&[(
        format!("{doc_id}-metadata"),
        with_tenant_scope(json!({
            "id": doc_id,
            "title": "Filter mismatch doc",
            "status": "completed",
            "created_at": "2026-01-01T00:00:00Z",
        })),
    )])
    .await
    .expect("seed meta");
    kv.upsert(&[(
        format!("{doc_id}-content"),
        json!({ "content": "body for cascade" }),
    )])
    .await
    .expect("seed content");

    // Entity owned by this document but scoped to a *different* workspace so
    // scoped discovery is empty while unscoped finds it → fail-closed cascade.
    let mut props = HashMap::new();
    props.insert("entity_type".into(), json!("PERSON"));
    props.insert("source_ids".into(), json!([doc_id]));
    props.insert("tenant_id".into(), json!(TEST_TENANT_ID));
    props.insert(
        "workspace_id".into(),
        json!("00000000-0000-0000-0000-000000000099"),
    );
    graph
        .upsert_node("SPEC098_MISMATCH_ENTITY", props)
        .await
        .expect("seed mismatch entity");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/batch-delete")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .header("X-Workspace-ID", TEST_WORKSPACE_ID)
                .body(Body::from(
                    serde_json::to_string(&json!({ "document_ids": [doc_id] })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let body = {
        let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("body");
        serde_json::from_slice::<Value>(&bytes).expect("json")
    };
    let track = body["batch_track_id"]
        .as_str()
        .expect("batch_track_id")
        .to_string();

    let mut result: Option<Value> = None;
    for _ in 0..80 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Ok(Some(task)) = workers.task_storage.get_task(&track).await {
            if matches!(
                task.status,
                TaskStatus::Indexed | TaskStatus::Failed | TaskStatus::Cancelled
            ) {
                result = task.result;
                break;
            }
        }
    }
    let result = result.expect("batch task must finish");

    let failed_ids = result["failed_ids"].as_array().cloned().unwrap_or_default();
    assert!(
        failed_ids.iter().any(|v| v.as_str() == Some(doc_id)),
        "failed_ids must include {doc_id}: {result}"
    );

    let failed = result["failed"].as_array().expect("failed array");
    let entry = failed
        .iter()
        .find(|f| f.get("document_id").and_then(|v| v.as_str()) == Some(doc_id))
        .expect("per-id failed entry");
    let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("");
    assert!(
        !reason.is_empty(),
        "failed[].reason must be non-empty: {entry}"
    );
    assert!(
        reason.to_lowercase().contains("mismatch")
            || reason.to_lowercase().contains("filter")
            || reason.to_lowercase().contains("graph")
            || reason.to_lowercase().contains("cascade"),
        "reason should describe cascade/filter failure, got: {reason}"
    );

    let meta = kv
        .get_by_id(&format!("{doc_id}-metadata"))
        .await
        .ok()
        .flatten()
        .expect("metadata retained as delete_failed");
    assert_eq!(
        meta.get("status").and_then(|v| v.as_str()),
        Some("delete_failed"),
        "KV must be delete_failed after cascade Err"
    );
}
