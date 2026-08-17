//! Contract: durable progress_counts on documents.metadata (list / Active Runs SSOT).

#![cfg(feature = "postgres")]

use std::time::Duration;

use edgequake_api::services::{
    begin_document_run, chunks_counts, mirror_document_stage_to_relational_with_counts,
    pages_counts, progress_counts_from_value, FenceEpoch,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

async fn test_pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&url)
        .await
        .ok()
}

#[tokio::test]
async fn mirror_persists_structured_page_and_chunk_counts() {
    let Some(pool) = test_pool().await else {
        eprintln!("Skipping progress_counts contract: PostgreSQL unavailable");
        return;
    };

    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    let document = Uuid::new_v4();
    let tenant_name = format!("pc-{tenant}");

    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $2)")
        .bind(tenant)
        .bind(&tenant_name)
        .execute(&pool)
        .await
        .expect("create test tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug)
         VALUES ($1, $2, $3, $3)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("pc-ws-{workspace}"))
    .execute(&pool)
    .await
    .expect("create test workspace");
    sqlx::query(
        r#"INSERT INTO documents
           (id, tenant_id, workspace_id, title, content, status, fence_epoch, track_id, metadata)
           VALUES ($1, $2, $3, 'progress counts', '', 'processing', 0, 'run-pc', '{}')"#,
    )
    .bind(document)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("create test document");

    let epoch = begin_document_run(
        &document.to_string(),
        "run-pc",
        "queued",
        10,
        "Queued",
        0.0,
        Some(&pool),
    )
    .await
    .expect("begin run");
    assert_eq!(epoch, FenceEpoch(1));

    assert!(
        mirror_document_stage_to_relational_with_counts(
            &pool,
            &document.to_string(),
            epoch,
            "run-pc",
            "processing",
            "converting",
            20,
            Some("Converting PDF to Markdown: page 3/10 (30%)"),
            Some(0.3),
            Some(&pages_counts(3, 10)),
        )
        .await
    );

    let counts_json: serde_json::Value =
        sqlx::query_scalar("SELECT metadata->'progress_counts' FROM documents WHERE id = $1")
            .bind(document)
            .fetch_one(&pool)
            .await
            .expect("read progress_counts");
    let pages = progress_counts_from_value(&counts_json).expect("pages counts");
    assert_eq!(pages.unit, "pages");
    assert_eq!(pages.current, 3);
    assert_eq!(pages.total, 10);

    assert!(
        mirror_document_stage_to_relational_with_counts(
            &pool,
            &document.to_string(),
            epoch,
            "run-pc",
            "processing",
            "extracting",
            50,
            Some("Extracting entities: chunk 4/20 (20%)"),
            Some(0.2),
            Some(&chunks_counts(4, 20)),
        )
        .await
    );

    let counts_json: serde_json::Value =
        sqlx::query_scalar("SELECT metadata->'progress_counts' FROM documents WHERE id = $1")
            .bind(document)
            .fetch_one(&pool)
            .await
            .expect("read chunk progress_counts");
    let chunks = progress_counts_from_value(&counts_json).expect("chunk counts");
    assert_eq!(chunks.unit, "chunks");
    assert_eq!(chunks.current, 4);
    assert_eq!(chunks.total, 20);

    let _ = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(document)
        .execute(&pool)
        .await;
}
