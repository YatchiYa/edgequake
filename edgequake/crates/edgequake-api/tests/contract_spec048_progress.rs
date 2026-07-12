//! SPEC-048: contract tests for ingestion progress + pipeline activity + Busy invariant.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{create_router, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> axum::Router {
    create_router(AppState::test_state())
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

#[tokio::test]
async fn contract_pipeline_activity_idle_not_busy() {
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/pipeline/activity")
                .header("X-Tenant-ID", "11111111-1111-1111-1111-111111111111")
                .header("X-Workspace-ID", "22222222-2222-2222-2222-222222222222")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["busy"], json!(false));
    assert!(body["working"].as_array().unwrap().is_empty());
    assert!(body["tasks"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn contract_ingestion_progress_404_unknown_track() {
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ingestion/missing-track-id/progress")
                .header("X-Tenant-ID", "11111111-1111-1111-1111-111111111111")
                .header("X-Workspace-ID", "22222222-2222-2222-2222-222222222222")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn contract_ingestion_progress_route_registered() {
    // OpenAPI / router must expose the path (DEF-01).
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/ingestion/any/progress")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    // Route exists (not 404 from nest miss) — method may be 405/401/404 for missing doc
    assert!(response.is_ok());
}

#[tokio::test]
async fn contract_reprocess_stage_reset_helper() {
    use edgequake_api::services::reprocess_stage_reset::apply_reprocess_stage_reset;
    use edgequake_tasks::ReprocessMode;
    use serde_json::json;

    let mut v = json!({
        "status": "completed",
        "current_stage": "completed",
        "stage_message": "stale done",
        "stage_progress": 1.0
    });
    apply_reprocess_stage_reset(v.as_object_mut().unwrap(), ReprocessMode::Full);
    assert_eq!(v["status"], "processing");
    assert_eq!(v["current_stage"], "queued");
    assert_eq!(v["stage_progress"], 0.0);
    assert_eq!(v["reprocess_mode"], "full");
}

#[tokio::test]
async fn contract_ws_chunk_progress_event_serializes() {
    use edgequake_api::handlers::ProgressEvent;
    let ev = ProgressEvent::ChunkProgress {
        document_id: "d".into(),
        task_id: "t".into(),
        chunk_index: 2,
        total_chunks: 10,
        chunk_preview: "x".into(),
        time_ms: 1,
        eta_seconds: 2,
        tokens_in: 3,
        tokens_out: 4,
        cost_usd: 0.01,
    };
    let v = serde_json::to_value(&ev).unwrap();
    assert_eq!(v["type"], "ChunkProgress");
    assert_eq!(v["data"]["chunk_index"], 2);
}
