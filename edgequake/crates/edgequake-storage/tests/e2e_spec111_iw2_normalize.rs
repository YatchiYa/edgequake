//! SPEC-111 E2E: iw2 normalize join + unresolved fails verify.
//!
//! Run:
//!   DATABASE_URL=… cargo test -p edgequake-storage --features postgres \
//!     --test e2e_spec111_iw2_normalize -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::embedding_family::EmbeddingFamily;
use edgequake_storage::entity_id::normalize_entity_name;
use edgequake_storage::migration_engine::coverage::EntityNameIndex;
use edgequake_storage::migration_engine::fleet_embedding_backfill::FleetEmbeddingBackfillJob;
use edgequake_storage::migration_engine::BackfillJob;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use sqlx::PgPool;
use uuid::Uuid;

const DIM: usize = 1536;

async fn run_to_completion(pool: &PgPool, job: &dyn BackfillJob) {
    let mut cursor = job.initial_cursor();
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let outcome = job.run_batch(&mut tx, &cursor, 8).await.expect("batch");
        tx.commit().await.expect("commit");
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
}

async fn seed_entity(pool: &PgPool, ws: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO public.entities (id, name, workspace_id, entity_type, description) \
         VALUES ($1, $2, $3, 'ORG', '') ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(name)
    .bind(ws)
    .execute(pool)
    .await
    .expect("seed entity");
    // Return actual id if conflict
    sqlx::query_scalar("SELECT id FROM public.entities WHERE name = $1 AND workspace_id = $2")
        .bind(name)
        .bind(ws)
        .fetch_one(pool)
        .await
        .expect("entity id")
}

#[tokio::test]
async fn e2e_spec111_iw2_normalize_join_writes_with_provenance() {
    let Some(cfg) = require_or_skip_postgres("spec111_iw2_norm") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    // Ensure legacy_vector_id column exists (migration 143).
    let _ = sqlx::raw_sql(include_str!(
        "../../../migrations/143_spec111_legacy_vector_id.sql"
    ))
    .execute(&pool)
    .await;

    let ws = w3::seed_workspace(&pool, "iw2norm").await;
    let display = "Acme Corp Ltd";
    let eid = seed_entity(&pool, ws, display).await;
    assert_eq!(normalize_entity_name(display), "ACME_CORP_LTD");

    let table = w3::create_vectors_table(&pool, "iw2norm").await;
    let legacy_id = "entity:ACME_CORP_LTD";
    let emb = w3::make_embedding(DIM, 99);
    sqlx::query(&format!(
        "INSERT INTO public.{table} (id, embedding, metadata) VALUES ($1, $2::vector, $3) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind(legacy_id)
    .bind(w3::vector_to_text(&emb))
    .bind(serde_json::json!({"workspace_id": ws.to_string()}))
    .execute(&pool)
    .await
    .expect("seed legacy entity vector");

    let job = FleetEmbeddingBackfillJob::new("spec111-iw2-model".into());
    run_to_completion(&pool, &job).await;

    let typed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.entity_embeddings \
         WHERE entity_id = $1 AND legacy_vector_id = $2",
    )
    .bind(eid)
    .bind(legacy_id)
    .fetch_one(&pool)
    .await
    .expect("typed count");
    assert_eq!(
        typed, 1,
        "normalize join must write entity_embeddings with provenance"
    );

    let report = job.verify(&pool).await.expect("verify");
    assert!(
        report.actual >= 1 && report.expected >= 1,
        "coverage actual/expected: {report:?}"
    );

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec111_iw2_unresolved_increments_failed() {
    let Some(cfg) = require_or_skip_postgres("spec111_iw2_fail") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let _ = sqlx::raw_sql(include_str!(
        "../../../migrations/143_spec111_legacy_vector_id.sql"
    ))
    .execute(&pool)
    .await;

    let ws = w3::seed_workspace(&pool, "iw2fail").await;
    // No spine entity — join must fail.
    let table = w3::create_vectors_table(&pool, "iw2fail").await;
    let emb = w3::make_embedding(DIM, 7);
    sqlx::query(&format!(
        "INSERT INTO public.{table} (id, embedding, metadata) VALUES ($1, $2::vector, $3) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind(format!(
        "entity:GHOST_T{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
    .bind(w3::vector_to_text(&emb))
    .bind(serde_json::json!({"workspace_id": ws.to_string()}))
    .execute(&pool)
    .await
    .expect("seed");

    let job = FleetEmbeddingBackfillJob::new("spec111-iw2-fail-model".into());
    let mut cursor = job.initial_cursor();
    let mut failed_total = 0i64;
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let outcome = job.run_batch(&mut tx, &cursor, 8).await.expect("batch");
        tx.commit().await.expect("commit");
        failed_total += outcome.failed;
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
    assert!(
        failed_total > 0,
        "unresolved entity join must increment failed (got {failed_total})"
    );

    std::env::set_var("EDGEQUAKE_MIGRATION_VERIFY_EQUALITY", "0");
    let report = job.verify(&pool).await.expect("verify");
    std::env::remove_var("EDGEQUAKE_MIGRATION_VERIFY_EQUALITY");
    assert!(
        !report.passes() || report.actual < report.expected,
        "unresolved coverage must not pass as full success: {report:?}"
    );

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[test]
fn contract_spec111_entity_name_index_unit() {
    let id = Uuid::new_v4();
    let index = EntityNameIndex::from_rows([(id, "Acme Corp Ltd".into())]);
    assert_eq!(index.resolve("ACME_CORP_LTD"), Some(id));
    let _ = EmbeddingFamily::Entity;
}
