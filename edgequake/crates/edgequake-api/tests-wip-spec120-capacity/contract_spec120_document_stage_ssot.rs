//! SPEC-120 A6 behavioral contract for run-aware document-stage SSOT.

#![cfg(feature = "postgres")]

use std::time::Duration;

use edgequake_api::document_read_model_keyset::list_relational_documents_keyset_page;
use edgequake_api::middleware::TenantContext;
use edgequake_api::services::{
    begin_document_run, mirror_document_stage_to_relational, FenceEpoch,
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
async fn run_fencing_monotonicity_and_keyset_projection_are_behavioral() {
    let Some(pool) = test_pool().await else {
        eprintln!("Skipping SPEC-120 stage SSOT contract: PostgreSQL unavailable");
        return;
    };
    let tenant = Uuid::new_v4();
    let other_tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    let document = Uuid::new_v4();
    let tenant_name = format!("spec120-{tenant}");
    let other_name = format!("spec120-{other_tenant}");

    for (id, name) in [(tenant, &tenant_name), (other_tenant, &other_name)] {
        sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $2)")
            .bind(id)
            .bind(name)
            .execute(&pool)
            .await
            .expect("create test tenant");
    }
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug)
         VALUES ($1, $2, $3, $3)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("spec120-ws-{workspace}"))
    .execute(&pool)
    .await
    .expect("create test workspace");
    sqlx::query(
        r#"INSERT INTO documents
           (id, tenant_id, workspace_id, title, content, status, fence_epoch, track_id, metadata)
           VALUES ($1, $2, $3, 'stage contract', '', 'indexed', 0, 'old',
             '{"cost_usd":1.25,"input_tokens":42,"source_type":"pdf","pdf_id":"pdf-contract"}')"#,
    )
    .bind(document)
    .bind(tenant)
    .bind(workspace)
    .execute(&pool)
    .await
    .expect("create test document");

    let epoch1 = begin_document_run(
        &document.to_string(),
        "run-1",
        "queued",
        10,
        "Queued",
        0.0,
        Some(&pool),
    )
    .await
    .expect("begin first run");
    assert_eq!(epoch1, FenceEpoch(1));
    assert!(
        mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch1,
            "run-1",
            "processing",
            "converting",
            20,
            Some("Converting 20%"),
            Some(0.2),
        )
        .await
    );
    assert!(
        !mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch1,
            "run-1",
            "processing",
            "converting",
            20,
            Some("Delayed 10%"),
            Some(0.1),
        )
        .await
    );
    assert!(
        mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch1,
            "run-1",
            "processing",
            "extracting",
            50,
            Some("Extracting"),
            Some(0.3),
        )
        .await
    );
    assert!(
        !mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch1,
            "run-1",
            "processing",
            "converting",
            20,
            Some("Late conversion"),
            Some(0.9),
        )
        .await
    );

    let epoch2 = begin_document_run(
        &document.to_string(),
        "run-2",
        "queued",
        10,
        "Queued again",
        0.0,
        Some(&pool),
    )
    .await
    .expect("begin second run");
    assert_eq!(epoch2, FenceEpoch(2));
    assert!(
        !mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch1,
            "run-1",
            "processing",
            "storing",
            100,
            Some("Old run"),
            Some(1.0),
        )
        .await
    );
    assert!(
        mirror_document_stage_to_relational(
            &pool,
            &document.to_string(),
            epoch2,
            "run-2",
            "processing",
            "converting",
            20,
            Some("New run converting"),
            Some(0.25),
        )
        .await
    );

    let context = TenantContext {
        tenant_id: Some(tenant.to_string()),
        workspace_id: Some(workspace.to_string()),
        user_id: None,
    };
    let (page, _) =
        list_relational_documents_keyset_page(&pool, &context, 20, 1, None, None, None, None, None)
            .await
            .expect("read through keyset SSOT");
    let row = page
        .iter()
        .find(|row| row.id == document.to_string())
        .expect("document appears in scoped keyset page");
    assert_eq!(row.track_id.as_deref(), Some("run-2"));
    assert_eq!(row.current_stage.as_deref(), Some("converting"));
    assert_eq!(row.stage_message.as_deref(), Some("New run converting"));
    assert_eq!(row.stage_progress, Some(0.25));
    assert_eq!(row.source_type.as_deref(), Some("pdf"));
    assert_eq!(row.pdf_id.as_deref(), Some("pdf-contract"));
    assert_eq!(row.cost_usd, Some(1.25));
    assert_eq!(row.input_tokens, Some(42));

    let other_context = TenantContext {
        tenant_id: Some(other_tenant.to_string()),
        ..context
    };
    let (other_page, _) = list_relational_documents_keyset_page(
        &pool,
        &other_context,
        20,
        1,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .expect("tenant-isolated keyset read");
    assert!(other_page.iter().all(|row| row.id != document.to_string()));

    sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(document)
        .execute(&pool)
        .await
        .expect("cleanup document");
    sqlx::query("DELETE FROM workspaces WHERE workspace_id = $1")
        .bind(workspace)
        .execute(&pool)
        .await
        .expect("cleanup workspace");
    sqlx::query("DELETE FROM tenants WHERE tenant_id = ANY($1)")
        .bind(vec![tenant, other_tenant])
        .execute(&pool)
        .await
        .expect("cleanup tenants");
}
