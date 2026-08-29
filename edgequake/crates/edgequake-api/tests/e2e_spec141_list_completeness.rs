//! SPEC-141 — List completeness (page-2 honesty, injections, conversation cursor).
//!
//! Run: `cargo test -p edgequake-api --test e2e_spec141_list_completeness`

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{
    create_test_app, extract_json, get_with_tenant, post_json_with_tenant, TEST_TENANT_ID,
    TEST_USER_ID, TEST_WORKSPACE_ID,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

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
    (resp.status(), extract_json(resp).await)
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
    (resp.status(), extract_json(resp).await)
}

async fn get_json_ws(app: &axum::Router, uri: &str, workspace_id: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("x-workspace-id", workspace_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    (resp.status(), extract_json(resp).await)
}

async fn create_pro_tenant(app: &axum::Router, name: &str) -> String {
    let slug = format!("spec141-{}", Uuid::new_v4());
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

async fn put_injection(app: &axum::Router, workspace_id: &str, name: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{workspace_id}/injection"))
                .header("content-type", "application/json")
                .header("x-workspace-id", workspace_id)
                .body(Body::from(
                    json!({ "name": name, "content": format!("{name} glossary") }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    (response.status(), extract_json(response).await)
}

#[tokio::test]
async fn e2e_141_01_workspace_list_page_two_is_nonempty() {
    let app = create_test_app();
    let tenant_id = create_pro_tenant(&app, "SPEC-141 Page2 Tenant").await;

    for i in 0..25 {
        let name = format!("spec141-ws-{i:02}");
        let slug = format!("spec141-ws-{i:02}-{}", Uuid::new_v4().simple());
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
    }

    let (status, page1) = get_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces?limit=10&offset=0"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let total = page1["total"].as_u64().expect("total") as usize;
    assert_eq!(page1["items"].as_array().expect("items").len(), 10);
    assert!(total >= 26, "Default Workspace + 25 named, got {total}");

    let (status, page2) = get_json(
        &app,
        &format!("/api/v1/tenants/{tenant_id}/workspaces?limit=10&offset=20"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let page2_items = page2["items"].as_array().expect("page2 items");
    assert!(
        !page2_items.is_empty(),
        "SPEC-141: offset=20 on 25+ workspaces must be nonempty: {page2}"
    );
    assert_eq!(page2["total"].as_u64().unwrap() as usize, total);
}

#[tokio::test]
async fn e2e_141_02_injection_list_default_page_hides_51st_but_total_is_honest() {
    let app = create_test_app();
    let ws = "00000000-0000-0000-0000-000000000141";
    let mut names = Vec::new();
    for i in 0..51 {
        let name = format!("spec141-inj-{i:02}");
        let (status, body) = put_injection(&app, ws, &name).await;
        assert_eq!(status, StatusCode::ACCEPTED, "put {name}: {status} {body}");
        names.push(name);
    }

    let (status, default_page) =
        get_json_ws(&app, &format!("/api/v1/workspaces/{ws}/injections"), ws).await;
    assert_eq!(status, StatusCode::OK);
    let items = default_page["items"].as_array().expect("items");
    let total = default_page["total"].as_u64().expect("total") as usize;
    assert_eq!(items.len(), 50, "default injection page size is 50");
    assert!(
        total >= 51,
        "total must count all injections, got {total}: {default_page}"
    );

    let (status, full) = get_json_ws(
        &app,
        &format!("/api/v1/workspaces/{ws}/injections?limit=200"),
        ws,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let full_items = full["items"].as_array().expect("full items");
    let full_names: Vec<&str> = full_items
        .iter()
        .filter_map(|i| i["name"].as_str())
        .collect();
    assert_eq!(full["total"].as_u64().unwrap() as usize, total);
    for name in &names {
        assert!(
            full_names.contains(&name.as_str()),
            "limit=200 must include {name}"
        );
    }
}

#[tokio::test]
async fn e2e_141_03_conversation_cursor_returns_second_page() {
    let app = create_test_app();

    for i in 0..25 {
        let (status, body) = post_json_with_tenant(
            &app,
            "/api/v1/conversations",
            &json!({ "title": format!("spec141-conv-{i:02}") }),
            TEST_TENANT_ID,
            TEST_USER_ID,
            TEST_WORKSPACE_ID,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "create conv {i}: {body}");
    }

    let (status, page1) = get_with_tenant(
        &app,
        "/api/v1/conversations",
        TEST_TENANT_ID,
        TEST_USER_ID,
        TEST_WORKSPACE_ID,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page1}");
    let items = page1["items"].as_array().expect("items");
    assert_eq!(items.len(), 20, "default conversation page size is 20");
    let has_more = page1["pagination"]["has_more"].as_bool().expect("has_more");
    assert!(
        has_more,
        "25 conversations must has_more on page 1: {page1}"
    );
    let cursor = page1["pagination"]["next_cursor"]
        .as_str()
        .expect("SPEC-141: next_cursor must be set while has_more");

    let (status, page2) = get_with_tenant(
        &app,
        &format!("/api/v1/conversations?cursor={cursor}"),
        TEST_TENANT_ID,
        TEST_USER_ID,
        TEST_WORKSPACE_ID,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{page2}");
    let page2_items = page2["items"].as_array().expect("page2 items");
    assert!(
        !page2_items.is_empty(),
        "second request with cursor must return remaining conversations: {page2}"
    );
    assert_eq!(
        page2_items.len() + items.len(),
        page1["pagination"]["total"].as_u64().unwrap_or(25) as usize
    );
}
