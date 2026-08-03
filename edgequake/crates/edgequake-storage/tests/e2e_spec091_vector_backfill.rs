//! SPEC-091 W3: legacy `eq_*_vectors` chunk rows → typed `chunk_embeddings`
//! backfill (engine descriptor) + verify, with crash-resume behavior.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_backfill -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::migration_engine::chunk_embedding_backfill::ChunkEmbeddingBackfillJob;
use edgequake_storage::migration_engine::BackfillJob;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::Value;
use sqlx::PgPool;

const DIM: usize = 1536;

/// Drive the job batch-by-batch (mirrors the runner loop, minus lease control)
/// until the source is exhausted, then run verify.
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

#[tokio::test]
async fn e2e_spec091_vector_backfill_covers_all_chunks() {
    let Some(cfg) = require_or_skip_postgres("spec091_w3_backfill") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "backfill").await;
    let doc = w3::seed_document(&pool, ws).await;
    // Relational spine must exist (the backfill joins through it).
    for i in 0..5 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    // Legacy chunk vectors only (no typed rows yet) — the pre-cutover state.
    let table = w3::create_vectors_table(&pool, "w3backfill").await;
    for i in 0..5 {
        let emb = w3::make_embedding(DIM, 100 + i as u32);
        w3::seed_legacy_chunk_vector(&pool, &table, doc, i, &emb).await;
    }

    let job = ChunkEmbeddingBackfillJob::new(format!("public.{table}"), "w3-bf-model".into());
    assert_eq!(job.step_id(), w3::W3_STEP);

    // Estimate counts the legacy chunk rows.
    let estimate = job.estimate_total(&pool).await.expect("estimate");
    assert_eq!(estimate, 5);

    run_to_completion(&pool, &job).await;

    // Typed coverage = 100% of the legacy chunk rows.
    let typed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("typed count");
    assert_eq!(typed, 5, "typed coverage must reach 100%");

    // Verify passes (coverage + sampled vector equality).
    let report = job.verify(&pool).await.expect("verify");
    assert_eq!(report.expected, 5);
    assert!(report.actual >= 5);
    assert_eq!(report.mismatches, 0, "sampled vectors must match");
    assert!(report.passes());

    // Idempotent rerun: re-running the whole backfill writes nothing new.
    run_to_completion(&pool, &job).await;
    let typed2: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("typed count 2");
    assert_eq!(typed2, 5, "backfill is idempotent");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_vector_backfill_crash_resume() {
    let Some(cfg) = require_or_skip_postgres("spec091_w3_resume") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "resume").await;
    let doc = w3::seed_document(&pool, ws).await;
    for i in 0..6 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    let table = w3::create_vectors_table(&pool, "w3resume").await;
    for i in 0..6 {
        let emb = w3::make_embedding(DIM, 200 + i as u32);
        w3::seed_legacy_chunk_vector(&pool, &table, doc, i, &emb).await;
    }

    let job = ChunkEmbeddingBackfillJob::new(format!("public.{table}"), "w3-bf-model".into());

    // Simulate a crash: run exactly one batch (limit 4 < 6), then stop with the
    // committed cursor (as the runner ledger would persist it).
    let mut cursor = job.initial_cursor();
    let mut tx = pool.begin().await.expect("begin");
    let outcome = job
        .run_batch(&mut tx, &cursor, 4)
        .await
        .expect("first batch");
    tx.commit().await.expect("commit");
    cursor = outcome.next_cursor.expect("more work remains");

    let partial: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("partial count");
    assert!(partial < 6, "crash leaves a partial backfill");

    // Resume from the persisted cursor → completes without duplicating.
    let mut cursor: Value = cursor;
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let outcome = job
            .run_batch(&mut tx, &cursor, 4)
            .await
            .expect("resume batch");
        tx.commit().await.expect("commit");
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("resumed count");
    assert_eq!(total, 6, "resume completes without duplicates");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_vector_backfill_missing_legacy_table_is_clean() {
    // EC-35: legacy vectors relation dropped ⇒ estimate 0 / verify passes.
    let Some(cfg) = require_or_skip_postgres("spec091_w3_gone") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let job = ChunkEmbeddingBackfillJob::new("public.eq_w3_gone_vectors".into(), "m".into());
    let estimate = job.estimate_total(&pool).await.expect("estimate gone");
    assert_eq!(estimate, 0);
    let report = job.verify(&pool).await.expect("verify gone");
    assert!(report.passes());
}
