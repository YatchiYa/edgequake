//! Endpoint sweep N3 — a deleted injection must not come back.
//!
//! PATCH (re-enqueue) then DELETE used to return 200, after which the still
//! running `knowledge_injection` task upserted its final status and re-created
//! the injection. Two fences:
//! - DELETE cancels the injection's pending/processing tasks;
//! - the worker's status write is update-only (never creates the row).
//!
//! Memory test is CI-safe. PG tests need `DATABASE_URL`:
//! ```bash
//! RUST_MIN_STACK=16777216 SQLX_OFFLINE=true DATABASE_URL=... \
//!   cargo test -p edgequake-api --features postgres --test e2e_n3_injection_delete_race
//! ```

#[cfg(feature = "postgres")]
mod common;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::injection_process::{injection_meta_key, write_injection_status};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_storage::traits::KVStorage;
use edgequake_tasks::storage::{Pagination, TaskFilter};
use edgequake_tasks::{TaskStatus, TaskType};
use serde_json::{json, Value};
use tower::ServiceExt;

const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE: &str = "00000000-0000-0000-0000-000000000003";

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Tenant-ID", DEFAULT_TENANT)
        .header("X-Workspace-ID", DEFAULT_WORKSPACE);
    if body.is_some() {
        req = req.header("Content-Type", "application/json");
    }
    let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));
    let resp = app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Terminal status write for a fixed injection (mirrors the worker call).
async fn write_status(
    kv: &Arc<dyn KVStorage>,
    pool: edgequake_api::services::OptionalPgPool<'_>,
    injection_id: &str,
) -> bool {
    let doc_id = format!("injection::{DEFAULT_WORKSPACE}::{injection_id}");
    write_injection_status(
        kv,
        pool,
        &injection_meta_key(DEFAULT_WORKSPACE, injection_id),
        injection_id,
        "n3",
        "n3 content",
        DEFAULT_WORKSPACE,
        "text",
        None,
        "completed",
        2,
        1,
        Some(&[]),
        &doc_id,
        "2026-10-16T00:00:00Z",
        None,
    )
    .await
}

/// DELETE cancels the queued processing task (no worker in test_state).
#[tokio::test]
async fn n3_delete_cancels_pending_injection_task_and_status_write_is_fenced() {
    let state = AppState::test_state();
    let app = Server::new(
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            enable_cors: false,
            enable_compression: false,
            enable_swagger: false,
        },
        state.clone(),
    )
    .build_router();
    let base = format!("/api/v1/workspaces/{DEFAULT_WORKSPACE}");

    let (s, put) = call(
        &app,
        "PUT",
        &format!("{base}/injection"),
        Some(json!({"name": "n3", "content": "Alpha relates to Beta."})),
    )
    .await;
    assert_eq!(s, StatusCode::ACCEPTED, "{put}");
    let id = put["injection_id"].as_str().unwrap().to_string();

    let (s, del) = call(&app, "DELETE", &format!("{base}/injections/{id}"), None).await;
    assert_eq!(s, StatusCode::OK, "{del}");

    let tasks = state
        .tasks
        .storage
        .list_tasks(
            TaskFilter {
                task_type: Some(TaskType::KnowledgeInjection),
                ..Default::default()
            },
            Pagination {
                page: 1,
                page_size: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .tasks;
    let mine: Vec<_> = tasks
        .iter()
        .filter(|t| t.task_data["injection_id"] == id.as_str())
        .collect();
    assert!(!mine.is_empty(), "PUT must have enqueued a task");
    assert!(
        mine.iter().all(|t| t.status == TaskStatus::Cancelled),
        "DELETE must cancel the injection task: {:?}",
        mine.iter().map(|t| t.status).collect::<Vec<_>>()
    );

    // Worker finishing late: KV-only path must not re-create the record.
    let kv = Arc::clone(&state.storage.kv_storage);
    assert!(!write_status(&kv, None, &id).await, "write must be skipped");
    let meta = kv
        .get_by_id(&injection_meta_key(DEFAULT_WORKSPACE, &id))
        .await
        .unwrap();
    assert!(
        meta.is_none(),
        "deleted injection resurrected in KV: {meta:?}"
    );
    let (s, _) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

/// Typed row: update-only — existing row updated, deleted row not re-created.
#[cfg(feature = "postgres")]
#[tokio::test]
async fn n3_pg_status_write_updates_but_never_creates_typed_row() {
    use edgequake_api::services::injection_relational::{
        typed_injection_delete, typed_injection_upsert,
    };
    let Some(url) = common::spec013_postgres::try_database_url() else {
        eprintln!("SKIP n3 pg: no DATABASE_URL");
        return;
    };
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let kv: Arc<dyn KVStorage> = Arc::new(edgequake_storage::MemoryKVStorage::new("n3-pg"));
    let id = uuid::Uuid::new_v4();
    let ids = id.to_string();
    let row_status = |pool: sqlx::PgPool| async move {
        sqlx::query_scalar::<_, String>("SELECT status FROM public.documents WHERE id = $1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .unwrap()
    };

    typed_injection_upsert(
        &json!({"id": ids, "workspace_id": DEFAULT_WORKSPACE, "name": "n3",
                "content": "x", "status": "processing"}),
        Some(&pool),
    )
    .await;
    assert_eq!(
        row_status(pool.clone()).await.as_deref(),
        Some("processing")
    );

    assert!(
        write_status(&kv, Some(&pool), &ids).await,
        "live row must be written"
    );
    let outcome = row_status(pool.clone()).await;

    typed_injection_delete(&ids, Some(&pool)).await;
    let late = write_status(&kv, Some(&pool), &ids).await;
    let after = row_status(pool.clone()).await;
    // Cleanup before asserting (the late write must not have created anything).
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(outcome.as_deref(), Some("indexed"));
    assert!(!late, "status write after delete must be skipped");
    assert_eq!(
        after, None,
        "deleted injection resurrected in public.documents"
    );
}

/// Full repro: PATCH (re-process) then immediate DELETE stays deleted.
#[cfg(feature = "postgres")]
#[tokio::test]
async fn n3_pg_patch_then_delete_stays_deleted() {
    std::env::set_var("EDGEQUAKE_KV_FAMILY_INJECTION", "relational");
    let Some(url) = common::spec013_postgres::try_database_url() else {
        eprintln!("SKIP n3 pg: no DATABASE_URL");
        return;
    };
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let Some(app) = common::spec013_postgres::create_postgres_mock_app_or_skip().await else {
        return;
    };
    let base = format!("/api/v1/workspaces/{DEFAULT_WORKSPACE}");
    let (s, put) = call(
        &app,
        "PUT",
        &format!("{base}/injection"),
        Some(json!({"name": "n3 race", "content": "Term Alpha relates to Term Beta."})),
    )
    .await;
    assert_eq!(s, StatusCode::ACCEPTED, "{put}");
    let id = put["injection_id"].as_str().unwrap().to_string();
    let inj = uuid::Uuid::parse_str(&id).unwrap();

    // Let the first processing settle so PATCH re-enqueues a fresh task.
    for _ in 0..120 {
        let (_, d) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
        if matches!(d["status"].as_str(), Some("completed") | Some("failed")) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    let (s, _) = call(
        &app,
        "PATCH",
        &format!("{base}/injections/{id}"),
        Some(json!({"content": "Term Gamma relates to Term Delta."})),
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    let (s, _) = call(&app, "DELETE", &format!("{base}/injections/{id}"), None).await;
    assert_eq!(s, StatusCode::OK);

    // Give any in-flight task time to finish (sweep saw ~0.4 s).
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let (get_status, _) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM public.documents WHERE id = $1 AND metadata->>'source_type' = 'injection'",
    )
    .bind(inj)
    .fetch_one(&pool)
    .await
    .unwrap();
    // Cleanup any residue (placeholder parent rows, chunks cascade).
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(inj)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        get_status,
        StatusCode::NOT_FOUND,
        "deleted injection came back"
    );
    assert_eq!(rows, 0, "deleted injection row re-created");
}
