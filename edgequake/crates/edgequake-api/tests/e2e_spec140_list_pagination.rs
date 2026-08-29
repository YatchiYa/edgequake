//! SPEC-140 — Honest list pagination (`total` = COUNT, not page length).
//!
//! Run: `cargo test -p edgequake-api --test e2e_spec140_list_pagination`

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn test_app() -> axum::Router {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    };
    Server::new(config, AppState::test_state()).build_router()
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(json!({}))
}

async fn get_json(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    (resp.status(), json_body(resp).await)
}

async fn post_json(app: &axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    (resp.status(), json_body(resp).await)
}

async fn create_pro_tenant(app: &axum::Router, name: &str) -> String {
    let slug = format!("spec140-{}", Uuid::new_v4());
    let (status, body) = post_json(
        app,
        "/api/v1/tenants",
        json!({
            "name": name,
            "slug": slug,
            "plan": "pro"
        }),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "create tenant {name}: {status} {body}"
    );
    body["id"].as_str().expect("tenant id").to_string()
}

#[tokio::test]
async fn e2e_140_01_workspace_list_total_is_count_not_page_length() {
    let app = test_app();
    let tenant_id = create_pro_tenant(&app, "SPEC-140 Workspace Tenant").await;

    let mut created_names = Vec::new();
    for i in 0..25 {
        let name = format!("spec140-ws-{i:02}");
        let slug = format!("spec140-ws-{i:02}-{}", Uuid::new_v4().simple());
        let (status, body) = post_json(
            &app,
            &format!("/api/v1/tenants/{tenant_id}/workspaces"),
            json!({ "name": name, "slug": slug }),
        )
        .await;
        assert!(
            status == StatusCode::CREATED || status == StatusCode::OK,
            "create {name}: {status} {body}"
        );
        created_names.push(name);
        assert_eq!(
            body["tenant_id"].as_str().unwrap_or_default(),
            tenant_id,
            "created workspace must belong to the path tenant"
        );
    }

    let (status, default_page) =
        get_json(&app, &format!("/api/v1/tenants/{tenant_id}/workspaces")).await;
    assert_eq!(status, StatusCode::OK);
    let items = default_page["items"].as_array().expect("items");
    let total = default_page["total"].as_u64().expect("total") as usize;
    assert_eq!(items.len(), 20, "default page size is 20");
    assert!(
        total >= 26,
        "total must include Default Workspace + 25 named, got {total}"
    );
    assert!(
        total > items.len(),
        "SPEC-140: total must not equal page length when N>20 (got total={total})"
    );
    for item in items {
        assert_eq!(
            item["tenant_id"].as_str().unwrap_or_default(),
            tenant_id,
            "page must not leak another tenant"
        );
    }

    let (status, full) = get_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces?limit=100"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let full_items = full["items"].as_array().expect("full items");
    let full_total = full["total"].as_u64().expect("full total") as usize;
    assert_eq!(full_total, total);
    assert_eq!(full_items.len(), total);
    let names: Vec<&str> = full_items
        .iter()
        .filter_map(|w| w["name"].as_str())
        .collect();
    for name in &created_names {
        assert!(
            names.contains(&name.as_str()),
            "limit=100 must include {name}, got {names:?}"
        );
    }

    let (status, past_end) = get_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces?offset=1000"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(past_end["items"].as_array().unwrap().is_empty());
    assert_eq!(past_end["total"].as_u64().unwrap() as usize, total);

    let (status, capped) = get_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces?limit=1000"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(capped["limit"].as_u64().unwrap(), 100);
    assert_eq!(capped["total"].as_u64().unwrap() as usize, total);
}

#[tokio::test]
async fn e2e_140_02_tenant_list_total_is_count_not_page_length() {
    let app = test_app();
    for i in 0..25 {
        let _ = create_pro_tenant(&app, &format!("SPEC-140 Org {i:02}")).await;
    }

    let (status, default_page) = get_json(&app, "/api/v1/tenants").await;
    assert_eq!(status, StatusCode::OK);
    let items = default_page["items"].as_array().expect("items");
    let total = default_page["total"].as_u64().expect("total") as usize;
    assert_eq!(items.len(), 20);
    assert!(total >= 25, "total must count all tenants, got {total}");
    assert!(
        total > items.len(),
        "SPEC-140: tenant total must not equal page length when N>20"
    );

    let (status, full) = get_json(&app, "/api/v1/tenants?limit=100").await;
    assert_eq!(status, StatusCode::OK);
    let full_items = full["items"].as_array().expect("full items");
    assert_eq!(full["total"].as_u64().unwrap() as usize, total);
    assert_eq!(full_items.len(), total);
}
