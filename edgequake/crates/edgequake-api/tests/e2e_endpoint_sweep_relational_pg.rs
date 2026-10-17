//! Endpoint-sweep regressions (2026-10-16) under the relational data families.
//!
//! - F1: `workspace_metrics_history` must live in `public` (migration 170).
//! - F2: `/documents/{id}/lineage` must read the typed `document_artifacts` row.
//! - F4: injection writes must land where relational reads look.
//!
//! Run:
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! SQLX_OFFLINE=true cargo test -p edgequake-api --features postgres \
//!   --test e2e_endpoint_sweep_relational_pg
//! # F4 drives the full worker path; debug builds need RUST_MIN_STACK=16777216.
//! ```

#![cfg(feature = "postgres")]

mod common;

use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_core::{MetricsTriggerType, WorkspaceService, WorkspaceServiceImpl};
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use common::spec013_postgres::{create_postgres_mock_app_or_skip, try_database_url};

const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE: &str = "00000000-0000-0000-0000-000000000003";

fn pin_relational_families() {
    std::env::set_var("EDGEQUAKE_KV_FAMILY_ARTIFACT", "relational");
    std::env::set_var("EDGEQUAKE_KV_FAMILY_INJECTION", "relational");
}

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
    let body = match body {
        Some(b) => {
            req = req.header("Content-Type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
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

/// F1: snapshot + history round-trip (500 "relation does not exist" before 170).
#[tokio::test]
#[serial]
async fn f1_metrics_snapshot_and_history_use_public_table() {
    let Some(url) = try_database_url() else {
        eprintln!("SKIP f1: no DATABASE_URL");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    let schema: Option<String> = sqlx::query_scalar(
        "SELECT schemaname::text FROM pg_tables WHERE tablename = 'workspace_metrics_history' AND schemaname = 'public'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(
        schema.as_deref(),
        Some("public"),
        "migration 170 must place the table in public"
    );

    let ws = Uuid::parse_str(DEFAULT_WORKSPACE).unwrap();
    let svc = WorkspaceServiceImpl::new(pool.clone());
    let snap = svc
        .record_metrics_snapshot(ws, MetricsTriggerType::Event)
        .await
        .expect("record_metrics_snapshot");
    let history = svc.get_metrics_history(ws, 50, 0).await;

    sqlx::query("DELETE FROM public.workspace_metrics_history WHERE id = $1")
        .bind(snap.id)
        .execute(&pool)
        .await
        .unwrap();

    let history = history.expect("get_metrics_history");
    assert!(
        history.iter().any(|s| s.id == snap.id),
        "snapshot must be readable back"
    );
}

/// F2: lineage served from the typed artifact row (404 before the fix).
#[tokio::test]
#[serial]
async fn f2_document_lineage_reads_typed_artifact() {
    pin_relational_families();
    let Some(url) = try_database_url() else {
        eprintln!("SKIP f2: no DATABASE_URL");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    let Some(app) = create_postgres_mock_app_or_skip().await else {
        return;
    };

    let doc = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.documents (id, tenant_id, workspace_id, title, content, status, metadata) \
         VALUES ($1, $2, $3, 'F2 lineage', 'x', 'completed', $4)",
    )
    .bind(doc)
    .bind(Uuid::parse_str(DEFAULT_TENANT).unwrap())
    .bind(Uuid::parse_str(DEFAULT_WORKSPACE).unwrap())
    // Shell metadata (`{doc}-metadata`) backs the handler's access check.
    .bind(json!({
        "id": doc.to_string(),
        "title": "F2 lineage",
        "status": "completed",
        "tenant_id": DEFAULT_TENANT,
        "workspace_id": DEFAULT_WORKSPACE,
    }))
    .execute(&pool)
    .await
    .expect("seed document");
    sqlx::query("INSERT INTO public.document_artifacts (document_id, kind, payload) VALUES ($1, 'lineage', $2)")
        .bind(doc)
        .bind(json!({"document_id": doc.to_string(), "chunks": [], "entities": [], "relationships": []}))
        .execute(&pool)
        .await
        .expect("seed lineage artifact");

    let (lineage, lineage_body) = call(
        &app,
        "GET",
        &format!("/api/v1/documents/{doc}/lineage"),
        None,
    )
    .await;
    let (export, _) = call(
        &app,
        "GET",
        &format!("/api/v1/documents/{doc}/lineage/export?format=json"),
        None,
    )
    .await;

    // CASCADE removes the artifact row too.
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(doc)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(lineage, StatusCode::OK, "lineage: {lineage_body}");
    assert_eq!(export, StatusCode::OK, "lineage export");
}

/// F4: PUT → list/get → PATCH → DELETE all see the same typed row.
#[tokio::test]
#[serial]
async fn f4_injection_crud_round_trip_relational() {
    pin_relational_families();
    let Some(url) = try_database_url() else {
        eprintln!("SKIP f4: no DATABASE_URL");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    let Some(app) = create_postgres_mock_app_or_skip().await else {
        return;
    };
    let base = format!("/api/v1/workspaces/{DEFAULT_WORKSPACE}");

    let (status, put) = call(
        &app,
        "PUT",
        &format!("{base}/injection"),
        Some(json!({"name": "F4 glossary", "content": "Term Alpha relates to Term Beta."})),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{put}");
    let id = put["injection_id"]
        .as_str()
        .expect("injection_id")
        .to_string();
    let inj = Uuid::parse_str(&id).unwrap();
    let outcome = futures::FutureExt::catch_unwind(std::panic::AssertUnwindSafe(async {
        // N2: the parent row carries its tenant, or the pipeline's RLS-guarded upsert fails.
        let tenant: Option<Uuid> =
            sqlx::query_scalar("SELECT tenant_id FROM public.documents WHERE id = $1")
                .bind(inj)
                .fetch_one(&pool)
                .await
                .expect("injection row");
        assert!(
            tenant.is_some(),
            "injection document row must carry tenant_id"
        );
        // Let the worker reach a terminal status (its status write must land too).
        let mut detail = Value::Null;
        for _ in 0..120 {
            let (s, d) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
            assert_eq!(s, StatusCode::OK, "GET by id right after PUT: {d}");
            if matches!(d["status"].as_str(), Some("completed") | Some("failed")) {
                detail = d;
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        assert!(
            !detail.is_null(),
            "injection {id} never reached a terminal status"
        );

        let (s, list) = call(&app, "GET", &format!("{base}/injections?limit=200"), None).await;
        assert_eq!(s, StatusCode::OK);
        assert!(
            list["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["injection_id"] == id.as_str()),
            "list must include {id}: {list}"
        );

        let (s, patched) = call(
            &app,
            "PATCH",
            &format!("{base}/injections/{id}"),
            Some(json!({"name": "F4 glossary renamed"})),
        )
        .await;
        assert_eq!(s, StatusCode::OK, "{patched}");
        let (_, after_patch) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
        assert_eq!(after_patch["name"], "F4 glossary renamed");

        let (s, _) = call(&app, "DELETE", &format!("{base}/injections/{id}"), None).await;
        assert_eq!(s, StatusCode::OK);
        let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM public.documents WHERE id = $1")
            .bind(inj)
            .fetch_one(&pool)
            .await
            .unwrap();
        let (s, _) = call(&app, "GET", &format!("{base}/injections/{id}"), None).await;
        assert_eq!(rows, 0, "DELETE must remove the typed injection row");
        assert_eq!(s, StatusCode::NOT_FOUND);
    }))
    .await;

    // Cleanup regardless of outcome (task row + any leftover document row).
    let _ = sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(inj)
        .execute(&pool)
        .await;
    let _ =
        sqlx::query("DELETE FROM public.tasks WHERE payload->'task_data'->>'injection_id' = $1")
            .bind(&id)
            .execute(&pool)
            .await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}
