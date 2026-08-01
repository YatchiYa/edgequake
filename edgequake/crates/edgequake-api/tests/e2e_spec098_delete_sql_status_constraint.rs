//! SPEC-098 F-098-13: documents_valid_status accepts deleting / delete_failed.
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use sqlx::PgPool;
use uuid::Uuid;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

#[tokio::test]
async fn e2e_spec098_delete_sql_status_constraint() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let Ok(pool) = PgPool::connect(&url).await else {
        eprintln!("skip: cannot connect to DATABASE_URL");
        return;
    };

    // Ensure CHECK includes lifecycle statuses (migration 141 / support).
    let apply = include_str!("../../../migrations/support/141/apply.sql");
    sqlx::raw_sql(apply)
        .execute(&pool)
        .await
        .expect("apply support/141");

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
        VALUES ($1, $2, $3, 'lifecycle', 'body', 'completed', $4)
        "#,
    )
    .bind(doc_id)
    .bind(tenant)
    .bind(workspace)
    .bind(format!("hash-{doc_id}"))
    .execute(&pool)
    .await
    .expect("insert completed doc");

    sqlx::query("UPDATE documents SET status = 'deleting' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .expect("status=deleting must be allowed");

    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("read status");
    assert_eq!(status, "deleting");

    sqlx::query("UPDATE documents SET status = 'delete_failed' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .expect("status=delete_failed must be allowed");

    let junk = sqlx::query("UPDATE documents SET status = 'not_a_real_status' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await;
    assert!(
        junk.is_err(),
        "junk status must violate documents_valid_status"
    );
}
