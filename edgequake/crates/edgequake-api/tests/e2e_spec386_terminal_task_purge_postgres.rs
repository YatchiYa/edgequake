//! Issue #386 — Postgres e2e: purge must not DELETE a Failed row from `tasks`.
//!
//! Memory tests already cover the status gate. This binary proves the same
//! contract against `PostgresTaskStorage` and a SQL read of the parent `tasks`
//! table (every partition, including `tasks_history` if the row lived there).
//!
//! `tasks_history` is a RANGE partition for old `created_at`, not an archive
//! API. A row created now lands in the current month child. The proof is:
//! `DELETE FROM tasks WHERE track_id = $1` was not issued — the parent still
//! returns `status=failed` and the original `error_message`.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-api --features postgres \
//!     --test e2e_spec386_terminal_task_purge_postgres -- --nocapture
//!
//! Skips when DATABASE_URL / POSTGRES_PASSWORD is unset or Postgres is down.

#![cfg(feature = "postgres")]

use std::env;
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_api::services::document_task_cleanup::purge_persisted_tasks_for_document;
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_storage::kv_keys;
use edgequake_tasks::postgres::PostgresTaskStorage;
use edgequake_tasks::{Task, TaskType};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "common/test_db.rs"]
mod test_db;

const COLLISION_ERROR: &str =
    r#"duplicate key value violates unique constraint "idx_entity_embeddings_legacy_vector_id""#;

fn database_url() -> Option<String> {
    let base = env::var("DATABASE_URL").ok().or_else(|| {
        let password = env::var("POSTGRES_PASSWORD").ok()?;
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "edgequake".to_string());
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "edgequake".to_string());
        Some(format!("postgresql://{user}:{password}@{host}:{port}/{db}"))
    })?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

async fn connect_pool() -> Option<PgPool> {
    let url = database_url()?;
    match PgPoolOptions::new().max_connections(5).connect(&url).await {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!("SKIP spec386 postgres: connect failed: {e}");
            None
        }
    }
}

async fn ensure_tenant_workspace(
    pool: &PgPool,
    tenant_id: Uuid,
    workspace_id: Uuid,
) -> Result<(), sqlx::Error> {
    let tenant_slug = format!("spec386_t_{}", &tenant_id.to_string()[..8]);
    sqlx::query(
        r#"
        INSERT INTO tenants (tenant_id, name, slug, is_active, metadata, settings, created_at, updated_at)
        VALUES ($1, $2, $3, TRUE, '{}'::jsonb, '{}'::jsonb, NOW(), NOW())
        ON CONFLICT (tenant_id) DO NOTHING
        "#,
    )
    .bind(tenant_id)
    .bind(format!("spec386 tenant {tenant_id}"))
    .bind(&tenant_slug)
    .execute(pool)
    .await?;

    let ws_slug = format!("spec386_w_{}", &workspace_id.to_string()[..8]);
    sqlx::query(
        r#"
        INSERT INTO workspaces (
            workspace_id, tenant_id, name, slug, is_active, metadata, settings, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, TRUE, '{}'::jsonb, '{}'::jsonb, NOW(), NOW())
        ON CONFLICT (workspace_id) DO NOTHING
        "#,
    )
    .bind(workspace_id)
    .bind(tenant_id)
    .bind(format!("spec386 workspace {workspace_id}"))
    .bind(&ws_slug)
    .execute(pool)
    .await?;
    Ok(())
}

fn ingest_task(tenant: Uuid, workspace: Uuid, doc_id: &str) -> Task {
    Task::new(
        tenant,
        workspace,
        TaskType::Insert,
        json!({ "document_id": doc_id }),
    )
}

fn postgres_task_state(pool: PgPool) -> AppState {
    let mut state = AppState::test_state();
    // test_state_with_pg_pool keeps MemoryTaskStorage — that would not prove #386.
    state.tasks.storage = Arc::new(PostgresTaskStorage::new(pool.clone()));
    state.pg_pool = Some(pool);
    state
}

async fn sql_task_status_error(pool: &PgPool, track_id: &str) -> Option<(String, Option<String>)> {
    sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT status::text, error_message FROM tasks WHERE track_id = $1",
    )
    .bind(track_id)
    .fetch_optional(pool)
    .await
    .expect("select tasks")
}

async fn cleanup_tracks(pool: &PgPool, track_ids: &[String]) {
    let _ = sqlx::query("DELETE FROM tasks WHERE track_id = ANY($1::text[])")
        .bind(track_ids)
        .execute(pool)
        .await;
}

async fn seed_failed_document(
    state: &AppState,
    tenant: Uuid,
    workspace: Uuid,
    doc_id: &str,
    track_id: &str,
) {
    let metadata = json!({
        "id": doc_id,
        "title": "spec386 postgres collision doc",
        "status": "failed",
        "error_message": COLLISION_ERROR,
        "track_id": track_id,
        "tenant_id": tenant.to_string(),
        "workspace_id": workspace.to_string(),
    });
    edgequake_api::services::upsert_metadata_kv_with_index(
        state.storage.kv_storage.as_ref(),
        &kv_keys::doc_metadata(doc_id),
        metadata,
    )
    .await
    .expect("seed metadata + wsdoc index");

    state
        .storage
        .kv_storage
        .upsert(&[(
            format!("{doc_id}-content"),
            json!({ "content": "Body that survived the failed persist." }),
        )])
        .await
        .unwrap();
}

fn test_config() -> ServerConfig {
    ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        enable_cors: false,
        enable_compression: false,
        enable_swagger: false,
    }
}

async fn json_body(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(json!({}))
}

/// Direct purge against PostgresTaskStorage + SQL parent-table read.
#[tokio::test]
async fn e2e_postgres_purge_keeps_failed_row_in_tasks_table() {
    let Some(pool) = connect_pool().await else {
        eprintln!("SKIP e2e_postgres_purge_keeps_failed_row_in_tasks_table: no DATABASE_URL");
        return;
    };

    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    if let Err(e) = ensure_tenant_workspace(&pool, tenant, workspace).await {
        eprintln!("SKIP spec386 postgres: tenant/workspace seed failed: {e}");
        return;
    }

    let state = postgres_task_state(pool.clone());
    let doc_id = format!("spec386-pg-purge-{}", Uuid::new_v4());

    let mut failed = ingest_task(tenant, workspace, &doc_id);
    failed.mark_failed(COLLISION_ERROR.to_string());
    let failed_id = failed.track_id.clone();
    state
        .tasks
        .storage
        .create_task(&failed)
        .await
        .expect("insert Failed");

    let mut inflight = ingest_task(tenant, workspace, &doc_id);
    inflight.mark_processing();
    let inflight_id = inflight.track_id.clone();
    state
        .tasks
        .storage
        .create_task(&inflight)
        .await
        .expect("insert Processing");

    let before = sql_task_status_error(&pool, &failed_id)
        .await
        .expect("Failed row must exist in tasks before purge");
    assert_eq!(before.0, "failed");
    assert_eq!(before.1.as_deref(), Some(COLLISION_ERROR));

    let purged =
        purge_persisted_tasks_for_document(&state, &doc_id, None, Some(&workspace.to_string()))
            .await;
    assert_eq!(purged, 1, "only the Processing sibling is in-flight");

    let after = sql_task_status_error(&pool, &failed_id)
        .await
        .expect("Failed row must remain in tasks after purge (#386)");
    assert_eq!(after.0, "failed", "must not rewrite Failed → cancelled");
    assert_eq!(after.1.as_deref(), Some(COLLISION_ERROR));

    assert!(
        sql_task_status_error(&pool, &inflight_id).await.is_none(),
        "Processing sibling must be deleted from Postgres tasks"
    );

    cleanup_tracks(&pool, &[failed_id, inflight_id]).await;
}

/// HTTP reprocess admit on Postgres-backed task storage.
#[tokio::test]
async fn e2e_postgres_http_reprocess_keeps_failed_row_in_tasks_table() {
    let Some(pool) = connect_pool().await else {
        eprintln!(
            "SKIP e2e_postgres_http_reprocess_keeps_failed_row_in_tasks_table: no DATABASE_URL"
        );
        return;
    };

    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    if let Err(e) = ensure_tenant_workspace(&pool, tenant, workspace).await {
        eprintln!("SKIP spec386 postgres: tenant/workspace seed failed: {e}");
        return;
    }

    let state = postgres_task_state(pool.clone());
    let doc_id = format!("spec386-pg-http-{}", Uuid::new_v4());

    let mut failed = ingest_task(tenant, workspace, &doc_id);
    failed.mark_failed(COLLISION_ERROR.to_string());
    let failed_id = failed.track_id.clone();
    state
        .tasks
        .storage
        .create_task(&failed)
        .await
        .expect("insert Failed");

    let mut inflight = ingest_task(tenant, workspace, &doc_id);
    inflight.mark_processing();
    let inflight_id = inflight.track_id.clone();
    state
        .tasks
        .storage
        .create_task(&inflight)
        .await
        .expect("insert Processing");

    seed_failed_document(&state, tenant, workspace, &doc_id, &failed_id).await;

    let app = Server::new(test_config(), state.clone()).build_router();
    let request = json!({
        "document_id": doc_id,
        "force": true,
        "max_documents": 1
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents/reprocess")
                .header("Content-Type", "application/json")
                .header("X-Tenant-ID", tenant.to_string())
                .header("X-Workspace-ID", workspace.to_string())
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = json_body(response).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::ACCEPTED,
        "reprocess should succeed, got {status}: {body}"
    );
    let new_id = body["document_task_ids"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v["task_id"].as_str())
        .map(str::to_string);
    assert!(
        body["requeued"].as_u64().unwrap_or(0) >= 1 || new_id.is_some(),
        "document must be requeued: {body}"
    );

    let after = sql_task_status_error(&pool, &failed_id)
        .await
        .expect("HTTP reprocess must not DELETE the Failed row from Postgres");
    assert_eq!(after.0, "failed");
    assert_eq!(after.1.as_deref(), Some(COLLISION_ERROR));
    assert!(
        sql_task_status_error(&pool, &inflight_id).await.is_none(),
        "reprocess purge must have deleted the Processing sibling from Postgres"
    );

    let new_id = new_id.expect("replacement task_id");
    assert_ne!(new_id, failed_id);
    let replacement = sql_task_status_error(&pool, &new_id)
        .await
        .expect("replacement task must be persisted in Postgres");
    assert!(
        replacement.0 == "pending" || replacement.0 == "processing",
        "new attempt status={}, expected inflight",
        replacement.0
    );

    cleanup_tracks(&pool, &[failed_id, inflight_id, new_id]).await;
}
