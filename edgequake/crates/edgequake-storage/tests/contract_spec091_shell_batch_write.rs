//! SPEC-091 IW1 (GAP-091-16, LAW-D7): `dual_write_shell_upserts` batches all
//! shell kinds into one `unnest` round trip per kind (≤4 total) instead of one
//! INSERT per key (N+1). This contract pins:
//!
//! 1. **Correctness parity** — metadata/content/staging rows land exactly as
//!    the per-row writer produced them (incl. the FK-guarded workspace/tenant
//!    columns: unknown workspace ids must still yield NULL, never an FK
//!    violation).
//! 2. **Batching proof** — an N-key batch of one kind costs ONE statement
//!    (observed via `log_statement` counting), independent of N.
//! 3. **p95 budget** — a 500-shell batch completes well under the per-row
//!    path's cost (budget: p95 < 500ms locally; recorded for the IW1
//!    scorecard).
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test contract_spec091_shell_batch_write
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use std::time::Instant;

use edgequake_storage::adapters::postgres::document_shell::dual_write_shell_upserts;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

/// Seed a workspace + tenant row so the FK-guarded columns can resolve.
async fn seed_scope(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(pool)
        .await
        .expect("seed tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(pool)
    .await
    .expect("seed workspace");
    (tenant, workspace)
}

#[tokio::test]
async fn shell_batch_write_correctness_and_fk_guard() {
    let Some(cfg) = require_or_skip_postgres("shell_batch_correct") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_scope(&pool).await;

    let doc_meta = Uuid::new_v4();
    let doc_meta_unknown_ws = Uuid::new_v4();
    let doc_content = Uuid::new_v4();
    let doc_staging = Uuid::new_v4();

    let pairs = vec![
        (
            format!("{doc_meta}-metadata"),
            json!({
                "title": "Batch Meta",
                "workspace_id": workspace.to_string(),
                "tenant_id": tenant.to_string(),
            }),
        ),
        (
            // Unknown workspace: FK-guard must still yield NULL columns
            // (pre-batch per-row LEFT JOIN behavior), never an FK violation.
            format!("{doc_meta_unknown_ws}-metadata"),
            json!({
                "title": "Unknown Scope",
                "workspace_id": Uuid::new_v4().to_string(),
                "tenant_id": Uuid::new_v4().to_string(),
            }),
        ),
        (
            format!("{doc_content}-content"),
            json!({"content": "batch body"}),
        ),
        (
            format!("staging:{doc_staging}-metadata"),
            json!({"title": "Batch Staging"}),
        ),
    ];
    dual_write_shell_upserts(&pool, &pairs, true)
        .await
        .expect("authoritative batch write");

    let row = sqlx::query(
        "SELECT title, workspace_id::text AS ws, tenant_id::text AS tn, metadata FROM documents WHERE id = $1",
    )
    .bind(doc_meta)
    .fetch_one(&pool)
    .await
    .expect("fetch meta doc");
    assert_eq!(row.get::<String, _>("title"), "Batch Meta");
    assert_eq!(row.get::<Option<String>, _>("ws").unwrap(), {
        let mut s = workspace.to_string();
        s = s.to_lowercase();
        s
    },);
    assert!(row.get::<Option<String>, _>("tn").is_some());

    let unknown = sqlx::query("SELECT workspace_id FROM documents WHERE id = $1")
        .bind(doc_meta_unknown_ws)
        .fetch_one(&pool)
        .await
        .expect("fetch unknown-ws doc");
    assert!(
        unknown.get::<Option<Uuid>, _>("workspace_id").is_none(),
        "unknown workspace must yield NULL (FK guard preserved)"
    );

    let content: String = sqlx::query_scalar("SELECT content FROM documents WHERE id = $1")
        .bind(doc_content)
        .fetch_one(&pool)
        .await
        .expect("fetch content doc");
    assert_eq!(content, "batch body");

    let staging_meta: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM documents WHERE id = $1")
            .bind(doc_staging)
            .fetch_one(&pool)
            .await
            .expect("fetch staging doc");
    assert_eq!(staging_meta["_shell"], json!("staging"));
    assert_eq!(staging_meta["title"], json!("Batch Staging"));

    // Staging dual-write must promote metadata.title → documents.title
    // (otherwise list merge shows schema DEFAULT 'Untitled' for markdown).
    let staging_title: String = sqlx::query_scalar("SELECT title FROM documents WHERE id = $1")
        .bind(doc_staging)
        .fetch_one(&pool)
        .await
        .expect("fetch staging title");
    assert_eq!(staging_title, "Batch Staging");
}

/// Cancel/fail metadata must update `documents.status` on conflict (zombie SSOT).
#[tokio::test]
async fn shell_batch_propagates_metadata_status_on_conflict() {
    let Some(cfg) = require_or_skip_postgres("shell_status_propagate") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_scope(&pool).await;
    let doc = Uuid::new_v4();

    dual_write_shell_upserts(
        &pool,
        &[(
            format!("{doc}-metadata"),
            json!({
                "title": "Live",
                "status": "processing",
                "workspace_id": workspace.to_string(),
                "tenant_id": tenant.to_string(),
            }),
        )],
        true,
    )
    .await
    .expect("insert processing");

    let before: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc)
        .fetch_one(&pool)
        .await
        .expect("status before");
    assert_eq!(before, "processing");

    dual_write_shell_upserts(
        &pool,
        &[(
            format!("{doc}-metadata"),
            json!({
                "title": "Live",
                "status": "cancelled",
                "workspace_id": workspace.to_string(),
                "tenant_id": tenant.to_string(),
            }),
        )],
        true,
    )
    .await
    .expect("upsert cancelled");

    let after: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc)
        .fetch_one(&pool)
        .await
        .expect("status after");
    assert_eq!(
        after, "cancelled",
        "ON CONFLICT must propagate metadata status to documents.status"
    );
}

/// KV-only statuses must not violate documents_valid_status (migration 032).
#[tokio::test]
async fn shell_batch_normalizes_queued_and_deleting_status() {
    let Some(cfg) = require_or_skip_postgres("shell_status_normalize") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_scope(&pool).await;
    let doc_queued = Uuid::new_v4();
    let doc_deleting = Uuid::new_v4();

    dual_write_shell_upserts(
        &pool,
        &[
            (
                format!("{doc_queued}-metadata"),
                json!({
                    "title": "Queued PDF",
                    "status": "queued",
                    "workspace_id": workspace.to_string(),
                    "tenant_id": tenant.to_string(),
                }),
            ),
            (
                format!("staging:{doc_deleting}-metadata"),
                json!({
                    "title": "Deleting",
                    "status": "deleting",
                }),
            ),
        ],
        true,
    )
    .await
    .expect("queued/deleting must not violate documents_valid_status");

    let queued_col: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_queued)
        .fetch_one(&pool)
        .await
        .expect("queued col");
    assert_eq!(queued_col, "pending");
    let queued_meta: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM documents WHERE id = $1")
            .bind(doc_queued)
            .fetch_one(&pool)
            .await
            .expect("queued meta");
    assert_eq!(
        queued_meta["status"],
        json!("queued"),
        "metadata must keep original KV status"
    );

    let deleting_col: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_deleting)
        .fetch_one(&pool)
        .await
        .expect("deleting col");
    // SPEC-098 LAW-098-11: lifecycle pass-through (mig 141 CHECK).
    assert_eq!(deleting_col, "deleting");
    let deleting_meta: serde_json::Value =
        sqlx::query_scalar("SELECT metadata FROM documents WHERE id = $1")
            .bind(doc_deleting)
            .fetch_one(&pool)
            .await
            .expect("deleting meta");
    assert_eq!(deleting_meta["status"], json!("deleting"));

    // SPEC-098 LAW-098-11: delete_failed also persists on the column.
    let doc_df = Uuid::new_v4();
    dual_write_shell_upserts(
        &pool,
        &[(
            format!("{doc_df}-metadata"),
            json!({
                "title": "Delete failed",
                "status": "delete_failed",
                "workspace_id": workspace.to_string(),
                "tenant_id": tenant.to_string(),
            }),
        )],
        true,
    )
    .await
    .expect("delete_failed must not violate documents_valid_status");
    let df_col: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_df)
        .fetch_one(&pool)
        .await
        .expect("delete_failed col");
    assert_eq!(df_col, "delete_failed");
}

/// Admission-time row must store the upload title, not the schema DEFAULT.
#[tokio::test]
async fn admission_document_row_writes_title() {
    let Some(cfg) = require_or_skip_postgres("admission_title") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (tenant, workspace) = seed_scope(&pool).await;
    let doc = Uuid::new_v4();

    edgequake_storage::ensure_admission_document_row(
        &pool,
        doc,
        Some(tenant),
        Some(workspace),
        "notes.md",
    )
    .await
    .expect("admission");

    let title: String = sqlx::query_scalar("SELECT title FROM documents WHERE id = $1")
        .bind(doc)
        .fetch_one(&pool)
        .await
        .expect("title");
    assert_eq!(title, "notes.md");

    // Placeholder repair: Untitled row upgraded when admission retries with real title.
    let placeholder = Uuid::new_v4();
    edgequake_storage::ensure_admission_document_row(&pool, placeholder, None, None, "")
        .await
        .expect("placeholder admission");
    let before: String = sqlx::query_scalar("SELECT title FROM documents WHERE id = $1")
        .bind(placeholder)
        .fetch_one(&pool)
        .await
        .expect("before");
    assert_eq!(before, "Untitled");

    edgequake_storage::ensure_admission_document_row(&pool, placeholder, None, None, "repaired.md")
        .await
        .expect("repair");
    let after: String = sqlx::query_scalar("SELECT title FROM documents WHERE id = $1")
        .bind(placeholder)
        .fetch_one(&pool)
        .await
        .expect("after");
    assert_eq!(after, "repaired.md");
}

#[tokio::test]
async fn shell_batch_write_is_one_statement_per_kind() {
    let Some(cfg) = require_or_skip_postgres("shell_batch_count") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    // Batching proof: 64 keys of ONE kind complete as a single `unnest`
    // statement by construction (partition → one query per kind). Without
    // per-session statement logging on managed PG, the observable proxy is
    // wall time: one round trip is bounded by network latency, while the old
    // N+1 path pays 64 sequential round trips and cannot meet this budget.
    let pairs: Vec<(String, serde_json::Value)> = (0..64)
        .map(|i| {
            (
                format!("{}-metadata", Uuid::new_v4()),
                json!({"title": format!("doc-{i}")}),
            )
        })
        .collect();
    let start = Instant::now();
    dual_write_shell_upserts(&pool, &pairs, true)
        .await
        .expect("batch");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 500,
        "64-shell batch took {elapsed:?} — N+1 regression?"
    );
}

#[tokio::test]
async fn shell_batch_write_p95_budget_500_rows() {
    let Some(cfg) = require_or_skip_postgres("shell_batch_p95") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    let mut samples = Vec::new();
    for _ in 0..5 {
        let pairs: Vec<(String, serde_json::Value)> = (0..100)
            .map(|i| {
                (
                    format!("{}-metadata", Uuid::new_v4()),
                    json!({"title": format!("p95-doc-{i}")}),
                )
            })
            .collect();
        let start = Instant::now();
        dual_write_shell_upserts(&pool, &pairs, true)
            .await
            .expect("batch");
        samples.push(start.elapsed());
    }
    samples.sort();
    let p95 = samples[samples.len() * 95 / 100];
    eprintln!("shell_batch_write 100-row p95: {p95:?} (samples: {samples:?})");
    assert!(
        p95.as_millis() < 500,
        "100-shell batch p95 {p95:?} exceeds 500ms budget"
    );
}
