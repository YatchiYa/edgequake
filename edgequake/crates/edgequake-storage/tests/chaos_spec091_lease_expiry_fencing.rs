//! SPEC-091 IW5: chaos — expired lease fencing; only the active owner writes.
//!
//! Two workers compete after worker A's lease expires. Stale progress updates
//! must not advance the ledger (record_batch_progress is owner-gated).
//!
//! Run:
//!   DATABASE_URL=... cargo test -p edgequake-storage --features postgres \
//!     --test chaos_spec091_lease_expiry_fencing -- --test-threads=1
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::migration_engine::chunk_embedding_backfill::ChunkEmbeddingBackfillJob;
use edgequake_storage::migration_engine::lease::{
    claim_lease, ensure_job_row, record_batch_progress,
};
use edgequake_storage::migration_engine::BackfillJob;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};

const DIM: usize = 1536;
const OWNER_A: &str = "fence-worker-a";
const OWNER_B: &str = "fence-worker-b";

#[tokio::test]
async fn chaos_spec091_lease_expiry_fencing_two_workers() {
    let Some(cfg) = require_or_skip_postgres("spec091_chaos_fence") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "chaos-fence").await;
    let doc = w3::seed_document(&pool, ws).await;
    for i in 0..6 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    let table = w3::create_vectors_table(&pool, "w3fence").await;
    for i in 0..6 {
        let emb = w3::make_embedding(DIM, 400 + i as u32);
        w3::seed_legacy_chunk_vector(&pool, &table, doc, i, &emb).await;
    }

    let job = ChunkEmbeddingBackfillJob::new(format!("public.{table}"), "w3-fence-model".into());
    ensure_job_row(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "reversible",
        3,
        Some(6),
    )
    .await
    .expect("ensure job");

    let lease_a = claim_lease(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        OWNER_A,
        2,
    )
    .await
    .expect("claim a")
    .expect("lease a");

    let cursor = job.initial_cursor();
    let mut tx = pool.begin().await.expect("begin");
    let outcome = job.run_batch(&mut tx, &cursor, 3).await.expect("batch a");
    tx.commit().await.expect("commit");

    record_batch_progress(
        &pool,
        lease_a.job_id,
        OWNER_A,
        2,
        outcome.scanned,
        outcome.failed,
        outcome.next_cursor.as_ref().unwrap(),
        3,
        None,
    )
    .await
    .expect("progress a");

    // Force lease expiry (simulates worker A crash without heartbeat).
    sqlx::query(
        "UPDATE edgequake.edgequake_migration_job SET lease_expires_at = now() - interval '1 second' \
         WHERE job_id = $1",
    )
    .bind(lease_a.job_id)
    .execute(&pool)
    .await
    .expect("expire lease");

    let lease_b = claim_lease(
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
    assert_eq!(lease_a.job_id, lease_b.job_id, "same ledger row re-claimed");

    let processed_before_stale: i64 = sqlx::query_scalar(
        "SELECT processed_count FROM edgequake.edgequake_migration_job WHERE job_id = $1",
    )
    .bind(lease_b.job_id)
    .fetch_one(&pool)
    .await
    .expect("processed before stale");

    // Stale worker A tries to write — must be fenced (0 rows updated).
    record_batch_progress(
        &pool,
        lease_a.job_id,
        OWNER_A,
        60,
        999,
        0,
        &serde_json::json!({"table": null, "last_id": "stale"}),
        3,
        None,
    )
    .await
    .expect("stale progress call");

    let processed_after_stale: i64 = sqlx::query_scalar(
        "SELECT processed_count FROM edgequake.edgequake_migration_job WHERE job_id = $1",
    )
    .bind(lease_b.job_id)
    .fetch_one(&pool)
    .await
    .expect("processed after stale");
    assert_eq!(
        processed_after_stale, processed_before_stale,
        "stale owner must not advance processed_count"
    );

    let owner: String = sqlx::query_scalar(
        "SELECT lease_owner FROM edgequake.edgequake_migration_job WHERE job_id = $1",
    )
    .bind(lease_b.job_id)
    .fetch_one(&pool)
    .await
    .expect("owner");
    assert_eq!(owner, OWNER_B);

    // Worker B completes remaining work.
    let mut cursor = lease_b.cursor_position.clone();
    if cursor.is_null() {
        cursor = outcome.next_cursor.clone().unwrap_or(job.initial_cursor());
    }
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let batch = job.run_batch(&mut tx, &cursor, 3).await.expect("batch b");
        tx.commit().await.expect("commit");
        if let Some(next) = &batch.next_cursor {
            record_batch_progress(
                &pool,
                lease_b.job_id,
                OWNER_B,
                60,
                batch.scanned,
                batch.failed,
                next,
                3,
                None,
            )
            .await
            .expect("progress b");
            cursor = next.clone();
        } else {
            break;
        }
    }

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("total");
    assert_eq!(total, 6, "fenced resume completes without duplicates");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
    w3::clear_w3_job(&pool).await;
}
