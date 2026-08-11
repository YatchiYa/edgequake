//! SPEC-118 / GitHub #376 — knowledge injection under relational chunk authority.
//!
//! Closes the CI blind spot where memory worker harnesses pin
//! `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=kv` and never exercise the #376 path.
//!
//! Run:
//! ```bash
//! export DATABASE_URL=postgres://edgequake:edgequake_secret@localhost:5432/edgequake
//! cargo test -p edgequake-api --features postgres --test e2e_spec118_injection_relational_pg
//! ```

#![cfg(feature = "postgres")]

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use edgequake_pipeline::chunker::TextChunk;
use edgequake_pipeline::pipeline::ProcessingResult;
use edgequake_pipeline::{persist_relational_chunks, IngestionPersistContext};
use edgequake_storage::PostgresChunkRepository;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use common::spec013_postgres::{create_postgres_mock_app_or_skip, try_database_url};

const DEFAULT_TENANT: &str = "00000000-0000-0000-0000-000000000002";
const DEFAULT_WORKSPACE: &str = "00000000-0000-0000-0000-000000000003";

fn pin_relational_authority() {
    // SPEC-118: force the product-default authority that surfaces #376.
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "relational");
}

fn require_db() -> Option<String> {
    try_database_url()
}

async fn wait_for_injection_status(
    app: &axum::Router,
    workspace_id: &str,
    injection_id: &str,
) -> serde_json::Value {
    for _ in 0..120 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!(
                        "/api/v1/workspaces/{workspace_id}/injections/{injection_id}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        if response.status() == StatusCode::OK {
            let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
                .await
                .unwrap();
            let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            match parsed["status"].as_str() {
                Some("completed") | Some("failed") => return parsed,
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    panic!("injection {injection_id} did not reach a terminal status in time");
}

/// Low-level: composite `injection::` id persists under relational writer (#376 seam).
#[tokio::test]
async fn e2e_spec118_pg_persist_injection_composite_document_id() {
    pin_relational_authority();
    let Some(url) = require_db() else {
        eprintln!("SKIP e2e_spec118_pg_persist_injection_composite_document_id: no DATABASE_URL");
        return;
    };

    let pool = PgPool::connect(&url).await.expect("connect");
    let tenant = Uuid::parse_str(DEFAULT_TENANT).unwrap();
    let workspace = Uuid::parse_str(DEFAULT_WORKSPACE).unwrap();
    let inj = Uuid::new_v4();
    let composite = format!("injection::{workspace}::{inj}");

    sqlx::query(
        r#"
        INSERT INTO public.documents (id, tenant_id, workspace_id, title, content, status, metadata)
        VALUES ($1, $2, $3, 'SPEC-118 PG verify', 'glossary', 'processing',
                jsonb_build_object('source_type', 'injection', 'source_document_id', $4::text))
        ON CONFLICT (id) DO UPDATE SET updated_at = now()
        "#,
    )
    .bind(inj)
    .bind(tenant)
    .bind(workspace)
    .bind(&composite)
    .execute(&pool)
    .await
    .expect("ensure injection document parent");

    let ctx = IngestionPersistContext::new(
        composite.clone(),
        Some(tenant.to_string()),
        Some(workspace.to_string()),
    );
    let result = ProcessingResult {
        document_id: composite.clone(),
        chunks: vec![TextChunk {
            id: format!("{composite}-chunk-0"),
            content: "glossary Term Alpha relates to Term Beta.".into(),
            index: 0,
            start_offset: 0,
            end_offset: 40,
            start_line: 1,
            end_line: 1,
            token_count: 8,
            embedding: None,
            section: None,
            page_start: None,
            page_end: None,
            modality: None,
        }],
        extractions: vec![],
        stats: Default::default(),
        lineage: None,
    };

    let repo = Arc::new(PostgresChunkRepository::new(pool.clone()));
    persist_relational_chunks(repo.as_ref(), &ctx, &result)
        .await
        .expect("SPEC-118: injection:: composite must persist under relational authority (#376)");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.chunks WHERE document_id = $1")
            .bind(inj)
            .fetch_one(&pool)
            .await
            .expect("count chunks");
    assert!(count >= 1, "expected chunks for injection UUID {inj}");

    let legacy: Option<String> = sqlx::query_scalar(
        "SELECT metadata->>'legacy_document_id' FROM public.chunks WHERE document_id = $1 LIMIT 1",
    )
    .bind(inj)
    .fetch_one(&pool)
    .await
    .expect("legacy_document_id");
    assert_eq!(legacy.as_deref(), Some(composite.as_str()));

    // Delete cascade: removing documents row must remove chunks.
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(inj)
        .execute(&pool)
        .await
        .expect("delete injection document");
    let after: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.chunks WHERE document_id = $1")
            .bind(inj)
            .fetch_one(&pool)
            .await
            .expect("count after delete");
    assert_eq!(after, 0, "chunks must CASCADE on documents delete");
}

/// Full worker path: PUT injection with authority=relational + Postgres pg_pool.
#[tokio::test]
async fn e2e_spec118_worker_injection_relational_completes_with_chunks() {
    pin_relational_authority();
    let Some(app) = create_postgres_mock_app_or_skip().await else {
        eprintln!(
            "SKIP e2e_spec118_worker_injection_relational_completes_with_chunks: no DATABASE_URL"
        );
        return;
    };

    // Authority must remain relational for this suite (harness must not flip to kv).
    assert_eq!(
        std::env::var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY")
            .ok()
            .as_deref(),
        Some("relational"),
        "SPEC-118 CI blind-spot: relational authority must stay pinned"
    );

    let body = json!({
        "name": "SPEC-118 Relational Glossary",
        "content": "Term Alpha relates to Term Beta. Enterprise Brain uses knowledge injection for glossary enrichment without citation."
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/workspaces/{DEFAULT_WORKSPACE}/injection"))
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .expect("put injection");
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let injection_id = parsed["injection_id"]
        .as_str()
        .expect("injection_id")
        .to_string();
    let workspace_id = parsed["workspace_id"]
        .as_str()
        .unwrap_or(DEFAULT_WORKSPACE)
        .to_string();

    let detail = wait_for_injection_status(&app, &workspace_id, &injection_id).await;
    assert_eq!(
        detail["status"].as_str(),
        Some("completed"),
        "injection must complete under relational authority; got: {detail}"
    );

    let url = require_db().expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.expect("connect");
    let inj = Uuid::parse_str(&injection_id).unwrap();
    let chunk_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.chunks WHERE document_id = $1")
            .bind(inj)
            .fetch_one(&pool)
            .await
            .expect("chunk count");
    assert!(
        chunk_count >= 1,
        "SPEC-118: relational chunks must exist for injection UUID {injection_id}"
    );

    let legacy: Option<String> = sqlx::query_scalar(
        "SELECT metadata->>'legacy_document_id' FROM public.chunks WHERE document_id = $1 LIMIT 1",
    )
    .bind(inj)
    .fetch_one(&pool)
    .await
    .expect("legacy");
    assert!(
        legacy
            .as_deref()
            .is_some_and(|s| s.starts_with("injection::") && s.contains(&injection_id)),
        "legacy_document_id bridge missing: {legacy:?}"
    );

    // Citation exclusion (SPEC-0002 / SPEC-028): no injection:: in query sources.
    let query_body = json!({
        "query": "What is Term Alpha?",
        "mode": "naive"
    });
    let qresp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/query")
                .header("Content-Type", "application/json")
                .header("X-Workspace-ID", &workspace_id)
                .body(Body::from(query_body.to_string()))
                .unwrap(),
        )
        .await
        .expect("query");
    assert_eq!(qresp.status(), StatusCode::OK, "query must succeed");
    let qbytes = axum::body::to_bytes(qresp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let qparsed: serde_json::Value = serde_json::from_slice(&qbytes).unwrap();
    let sources = qparsed
        .get("sources")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for source in &sources {
        let doc_id = source
            .get("document_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            !doc_id.starts_with("injection::"),
            "SPEC-0002 VIOLATION: injection cited: {source}"
        );
    }

    // DELETE injection → typed chunks cascade.
    let del = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/workspaces/{workspace_id}/injections/{injection_id}"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("delete");
    assert!(
        del.status() == StatusCode::OK || del.status() == StatusCode::NO_CONTENT,
        "delete status {}",
        del.status()
    );

    // Allow async cleanup
    tokio::time::sleep(Duration::from_millis(500)).await;
    let after: i64 =
        sqlx::query_scalar("SELECT count(*) FROM public.chunks WHERE document_id = $1")
            .bind(inj)
            .fetch_one(&pool)
            .await
            .expect("chunk count after delete");
    assert_eq!(
        after, 0,
        "delete injection must cascade-remove relational chunks for {injection_id}"
    );
}

/// Source-level guard: memory worker harness still pins kv — relational coverage lives here.
#[test]
fn contract_spec118_memory_harness_documents_kv_pin_and_relational_suite() {
    let common_src = include_str!("common/mod.rs");
    assert!(
        common_src.contains("EDGEQUAKE_CHUNK_TEXT_AUTHORITY") && common_src.contains("\"kv\""),
        "memory harness still pins kv (expected)"
    );
    // This file is the relational SSOT e2e (must pin relational, not kv).
    let this_src = include_str!("e2e_spec118_injection_relational_pg.rs");
    assert!(this_src.contains("pin_relational_authority"));
    assert!(this_src.contains("\"relational\""));
}
