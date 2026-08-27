//! SSE must not be gzip-buffered (tower-http #420 / PR #389).
//!
//! Run: `cargo test -p edgequake-api --test e2e_sse_compression`

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::json;
use tower::ServiceExt;

fn compressed_app() -> axum::Router {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: true,
        enable_swagger: false,
    };
    Server::new(config, AppState::test_state()).build_router()
}

#[tokio::test]
async fn query_stream_skips_gzip_and_sets_proxy_headers() {
    let app = compressed_app();
    let request = json!({ "query": "What is machine learning?" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/query/stream")
                .header("Content-Type", "application/json")
                .header(header::ACCEPT_ENCODING, "gzip")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let headers = response.headers();
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("text/event-stream"),
        "content-type={content_type}"
    );
    assert!(
        headers.get(header::CONTENT_ENCODING).is_none(),
        "SSE must not be gzip-encoded: {:?}",
        headers.get(header::CONTENT_ENCODING)
    );
    assert_eq!(
        headers
            .get("x-accel-buffering")
            .and_then(|v| v.to_str().ok()),
        Some("no")
    );
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-cache")
    );

    let body = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    let data_events = text.matches("data:").count();
    assert!(
        data_events > 1,
        "expected multiple SSE data events, got {data_events}: {text}"
    );
}
