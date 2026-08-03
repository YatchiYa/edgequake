//! SPEC-098 F-098-16 / LAW-098-11: admit + reset → SQL+KV `delete_failed`;
//! list wire status is not collapsed to pipeline `failed`.
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::relational_sidecar_store::register_sidecar_pool;
use edgequake_api::services::{admit_document_deleting, reset_deleting_status};
use edgequake_api::{AppState, Server, ServerConfig};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

#[tokio::test]
async fn e2e_spec098_delete_failed_status_persists() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let Ok(pool) = PgPool::connect(&url).await else {
        eprintln!("skip: cannot connect to DATABASE_URL");
        return;
    };

    let apply = include_str!("../../../migrations/support/141/apply.sql");
    sqlx::raw_sql(apply)
        .execute(&pool)
        .await
        .expect("apply support/141");

    register_sidecar_pool(pool.clone());

    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    let doc_id = Uuid::new_v4();

    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(&pool)
    .await
    .expect("workspace");

    sqlx::query(
        r#"
        INSERT INTO documents (id, tenant_id, workspace_id, title, content, status, content_hash)
        VALUES ($1, $2, $3, 'lifecycle-fail', 'body', 'completed', $4)
        "#,
    )
    .bind(doc_id)
    .bind(tenant)
    .bind(workspace)
    .bind(format!("hash-{doc_id}"))
    .execute(&pool)
    .await
    .expect("insert doc");

    let state = AppState::test_state_with_pg_pool(pool.clone());
    let doc_id_str = doc_id.to_string();
    state
        .storage
        .kv_storage
        .upsert(&[(
            format!("{doc_id_str}-metadata"),
            json!({
                "id": doc_id_str,
                "title": "lifecycle-fail",
                "status": "completed",
                "tenant_id": tenant.to_string(),
                "workspace_id": workspace.to_string(),
                "created_at": "2026-01-01T00:00:00Z",
            }),
        )])
        .await
        .expect("seed kv");

    admit_document_deleting(&state, &doc_id_str, &doc_id_str)
        .await
        .expect("admit deleting");

    let sql_deleting: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("read deleting");
    assert_eq!(sql_deleting, "deleting");

    reset_deleting_status(
        &state,
        &doc_id_str,
        &doc_id_str,
        "cascade boom for SPEC-098",
        Some("track-spec098-df"),
    )
    .await;

    let sql_failed: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("read delete_failed");
    assert_eq!(
        sql_failed, "delete_failed",
        "SQL column must persist lifecycle delete_failed (not pipeline failed)"
    );

    let meta = state
        .storage
        .kv_storage
        .get_by_id(&format!("{doc_id_str}-metadata"))
        .await
        .ok()
        .flatten()
        .expect("kv meta");
    assert_eq!(
        meta.get("status").and_then(|v| v.as_str()),
        Some("delete_failed")
    );

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
                .method("GET")
                .uri("/api/v1/documents?page_size=100")
                .header("X-Tenant-ID", tenant.to_string())
                .header("X-Workspace-ID", workspace.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    let docs = body["documents"].as_array().cloned().unwrap_or_default();
    let hit = docs.iter().find(|d| {
        d.get("id")
            .and_then(|v| v.as_str())
            .map(|id| id == doc_id_str)
            .unwrap_or(false)
    });
    assert!(
        hit.is_some(),
        "list must include delete_failed doc; body={body}"
    );
    let hit = hit.unwrap();
    assert_eq!(
        hit.get("status").and_then(|v| v.as_str()),
        Some("delete_failed"),
        "list wire must not collapse delete_failed → failed: {hit}"
    );

    let failed_count = body["status_counts"]["failed"].as_u64().unwrap_or(u64::MAX);
    assert_eq!(
        failed_count, 0,
        "Retry Failed bucket must exclude delete_failed"
    );
}
