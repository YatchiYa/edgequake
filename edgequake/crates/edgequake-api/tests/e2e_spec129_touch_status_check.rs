//! SPEC-129 / #381: touch_document_status projects CHECK-safe column statuses.
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use edgequake_storage::{
    relational_documents_status_for_write, PdfDocumentStorage, PostgresPdfStorage,
};
use sqlx::PgPool;
use uuid::Uuid;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

async fn seed_tenant_workspace_doc(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let apply = include_str!("../../../migrations/support/141/apply.sql");
    sqlx::raw_sql(apply)
        .execute(pool)
        .await
        .expect("apply support/141");

    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    let doc_id = Uuid::new_v4();

    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("w-{workspace}"))
    .bind(format!("w-{workspace}"))
    .execute(pool)
    .await
    .expect("workspace");

    sqlx::query(
        r#"
        INSERT INTO documents (id, tenant_id, workspace_id, title, content, status, content_hash)
        VALUES ($1, $2, $3, 'spec129', 'body', 'failed', $4)
        "#,
    )
    .bind(doc_id)
    .bind(tenant)
    .bind(workspace)
    .bind(format!("hash-{doc_id}"))
    .execute(pool)
    .await
    .expect("insert failed doc");

    (tenant, workspace, doc_id)
}

#[tokio::test]
async fn e2e_spec129_touch_status_check() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let Ok(pool) = PgPool::connect(&url).await else {
        eprintln!("skip: cannot connect to DATABASE_URL");
        return;
    };

    let (_tenant, _workspace, doc_id) = seed_tenant_workspace_doc(&pool).await;

    // Raw KV stage must still be rejected by CHECK (no widen).
    let raw = sqlx::query("UPDATE documents SET status = 're_embedding' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await;
    assert!(
        raw.is_err(),
        "raw re_embedding must violate documents_valid_status"
    );

    let storage = PostgresPdfStorage::new(pool.clone());

    storage
        .touch_document_status(&doc_id, "re_embedding")
        .await
        .expect("touch re_embedding must succeed after SPEC-129");
    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("read status");
    assert_eq!(status, "processing");

    storage
        .touch_document_status(&doc_id, "queued")
        .await
        .expect("touch queued");
    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending");

    storage
        .touch_document_status(&doc_id, "merging")
        .await
        .expect("touch merging");
    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "processing");

    storage
        .touch_document_status(&doc_id, "deleting")
        .await
        .expect("touch deleting");
    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "deleting");

    storage
        .touch_document_status(&doc_id, "completed")
        .await
        .expect("touch completed");
    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "indexed");

    assert_eq!(
        relational_documents_status_for_write("re_embedding"),
        "processing"
    );
}

#[tokio::test]
async fn e2e_spec129_ensure_document_record_projects_re_embedding() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let Ok(pool) = PgPool::connect(&url).await else {
        eprintln!("skip: cannot connect to DATABASE_URL");
        return;
    };

    let (tenant, workspace, _seed_doc) = seed_tenant_workspace_doc(&pool).await;
    let doc_id = Uuid::new_v4();
    let storage = PostgresPdfStorage::new(pool.clone());

    storage
        .ensure_document_record(
            &doc_id,
            &workspace,
            Some(&tenant),
            "spec129-ensure",
            "body",
            "re_embedding",
        )
        .await
        .expect("ensure_document_record(re_embedding) must succeed after SPEC-129");

    let status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .expect("read status");
    assert_eq!(status, "processing");
}

/// Source contract: writers must call the SPEC-129 SSOT helper; KV honesty kept.
#[test]
fn contract_spec129_writers_reference_helper() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let storage_root = std::path::Path::new(manifest).join("../edgequake-storage/src");
    let api_root = std::path::Path::new(manifest).join("src");

    let files = [
        storage_root.join("adapters/postgres/pdf_storage_impl.rs"),
        storage_root.join("adapters/memory/pdf.rs"),
        api_root.join("services/task_document_sync.rs"),
        api_root.join("processor/status_updates.rs"),
        api_root.join("services/document_stage_mirror.rs"),
        api_root.join("processor/text_insert/finalize.rs"),
    ];
    for path in &files {
        let src = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            src.contains("relational_documents_status_for_write"),
            "{} must call relational_documents_status_for_write (SPEC-129)",
            path.display()
        );
        assert!(
            !src.contains("let pg_status = if status == \"completed\""),
            "{} must not use incomplete completed→indexed-only map",
            path.display()
        );
        assert!(
            !src.contains("let pg_status = if final_status == \"completed\""),
            "{} must not use incomplete final_status completed→indexed-only map",
            path.display()
        );
    }

    let pg_impl = std::fs::read_to_string(
        storage_root.join("adapters/postgres/pdf_storage_impl.rs"),
    )
    .expect("pdf_storage_impl");
    let ensure_idx = pg_impl
        .find("async fn ensure_document_record")
        .expect("ensure_document_record present");
    let ensure_tail = &pg_impl[ensure_idx..ensure_idx.saturating_add(1200)];
    assert!(
        ensure_tail.contains("relational_documents_status_for_write"),
        "ensure_document_record must project via relational_documents_status_for_write"
    );

    let extraction = std::fs::read_to_string(api_root.join("processor/text_insert/extraction.rs"))
        .expect("extraction.rs");
    assert!(
        extraction.contains("re_embedding"),
        "KV honesty: slim-resume must still set re_embedding"
    );
}
