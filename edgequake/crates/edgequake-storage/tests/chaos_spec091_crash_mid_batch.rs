//! SPEC-091 IW5: chaos — graceful cancel mid-backfill, resume without duplicates.
//!
//! Simulates crash/lease loss via operator cancel (in-process kill -9 is not
//! attempted; cancel releases the lease the same way a dead worker would after
//! TTL expiry). Reuses W3 backfill helpers + migration ledger.
//!
//! Run:
//!   DATABASE_URL=... cargo test -p edgequake-storage --features postgres \
//!     --test chaos_spec091_crash_mid_batch -- --test-threads=1
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::migration_engine::chunk_embedding_backfill::ChunkEmbeddingBackfillJob;
use edgequake_storage::migration_engine::lease::{
    claim_lease, control_job, ensure_job_row, JobControl,
};
use edgequake_storage::migration_engine::BackfillJob;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use sqlx::PgPool;

const DIM: usize = 1536;
const OWNER_A: &str = "chaos-worker-a";
const OWNER_B: &str = "chaos-worker-b";

async fn run_one_batch(
    pool: &PgPool,
    job: &dyn BackfillJob,
    cursor: &serde_json::Value,
    limit: i64,
) -> edgequake_storage::migration_engine::BatchOutcome {
    let mut tx = pool.begin().await.expect("begin");
    let outcome = job.run_batch(&mut tx, cursor, limit).await.expect("batch");
    tx.commit().await.expect("commit");
    outcome
}

async fn run_to_completion(pool: &PgPool, job: &dyn BackfillJob, cursor: serde_json::Value) {
    let mut cursor = cursor;
    loop {
        let outcome = run_one_batch(pool, job, &cursor, 4).await;
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
}

#[tokio::test]
async fn chaos_spec091_crash_mid_batch_cancel_then_resume() {
    let Some(cfg) = require_or_skip_postgres("spec091_chaos_crash") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "chaos-crash").await;
    let doc = w3::seed_document(&pool, ws).await;
    for i in 0..8 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    let table = w3::create_vectors_table(&pool, "w3chaos").await;
    for i in 0..8 {
        let emb = w3::make_embedding(DIM, 300 + i as u32);
        w3::seed_legacy_chunk_vector(&pool, &table, doc, i, &emb).await;
    }

    let job = ChunkEmbeddingBackfillJob::new(format!("public.{table}"), "w3-chaos-model".into());
    ensure_job_row(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "reversible",
        4,
        Some(8),
    )
    .await
    .expect("ensure job");

    let lease_a = claim_lease(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        OWNER_A,
        60,
    )
    .await
    .expect("claim a")
    .expect("lease a");

    let mut cursor = job.initial_cursor();
    let first = run_one_batch(&pool, &job, &cursor, 4).await;
    cursor = first.next_cursor.expect("partial cursor");

    // Operator cancel ≡ graceful crash (lease released; cursor persisted externally).
    control_job(&pool, lease_a.job_id, JobControl::Cancel)
        .await
        .expect("cancel");

    let partial: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("partial");
    assert!(partial > 0 && partial < 8, "cancel leaves partial coverage");

    // Re-register + new worker resumes from persisted cursor (no duplicates).
    ensure_job_row(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "reversible",
        4,
        Some(8),
    )
    .await
    .expect("re-register");
    sqlx::query(
        "UPDATE edgequake.edgequake_migration_job SET state = 'running', cursor_position = $2, \
         processed_count = $3, lease_owner = NULL, lease_expires_at = NULL \
         WHERE step_id = $1 AND schema_generation = $4",
    )
    .bind(job.step_id())
    .bind(&cursor)
    .bind(first.scanned)
    .bind(job.schema_generation())
    .execute(&pool)
    .await
    .expect("restore cursor");

    let _lease_b = claim_lease(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        OWNER_B,
        60,
    )
    .await
    .expect("claim b")
    .expect("lease b");

    run_to_completion(&pool, &job, cursor).await;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("total");
    assert_eq!(total, 8, "resume completes without duplicate embeddings");

    let report = job.verify(&pool).await.expect("verify");
    assert_eq!(report.mismatches, 0);

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
    w3::clear_w3_job(&pool).await;
}
