//! SPEC-091 P1: live E2E for migration job control (pause/resume/cancel),
//! the batch-progress race guard, and the rate/ETA detail surface.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec091_job_control -- --nocapture

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::migration_engine::lease::{
    control_job, current_state, job_detail, record_batch_progress, JobControl,
};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use sqlx::PgPool;

/// Insert a synthetic job row in 'running' state with a lease, returning its id.
async fn seed_running_job(pool: &PgPool, step_id: &str) -> sqlx::types::Uuid {
    let job_id: sqlx::types::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO edgequake.edgequake_migration_job
            (step_id, step_sha384, schema_generation, state, reversibility,
             batch_size, estimated_total, lease_owner, lease_expires_at, heartbeat_at)
        VALUES ($1, 'deadbeef', 999, 'running', 'reversible',
                100, 1000, 'e2e-owner', now() + interval '60 seconds', now())
        RETURNING job_id
        "#,
    )
    .bind(step_id)
    .fetch_one(pool)
    .await
    .expect("seed job");
    job_id
}

async fn cleanup(pool: &PgPool, job_id: sqlx::types::Uuid) {
    sqlx::query("DELETE FROM edgequake.edgequake_migration_job WHERE job_id = $1")
        .bind(job_id)
        .execute(pool)
        .await
        .expect("cleanup job");
}

#[tokio::test]
async fn e2e_spec091_pause_resume_cancel_state_machine() {
    let Some(cfg) = require_or_skip_postgres("spec091_jobctl") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let step = format!(
        "e2e-jobctl-{}",
        &sqlx::types::Uuid::new_v4().to_string()[..8]
    );
    let job_id = seed_running_job(&pool, &step).await;

    // Pause from running → paused.
    let s = control_job(&pool, job_id, JobControl::Pause)
        .await
        .expect("pause");
    assert_eq!(s, "paused");
    assert_eq!(
        current_state(&pool, job_id).await.unwrap().as_deref(),
        Some("paused")
    );

    // Pause again → illegal (already paused).
    assert!(control_job(&pool, job_id, JobControl::Pause).await.is_err());

    // Resume → running.
    let s = control_job(&pool, job_id, JobControl::Resume)
        .await
        .expect("resume");
    assert_eq!(s, "running");

    // Cancel from running → cancelled; lease released.
    let s = control_job(&pool, job_id, JobControl::Cancel)
        .await
        .expect("cancel");
    assert_eq!(s, "cancelled");
    let (owner, completed_at): (Option<String>, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as(
            "SELECT lease_owner, completed_at FROM edgequake.edgequake_migration_job \
             WHERE job_id = $1",
        )
        .bind(job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(owner.is_none(), "cancel releases the lease");
    assert!(completed_at.is_some(), "cancel stamps completed_at");

    // Terminal: resume/pause/cancel all illegal from cancelled.
    assert!(control_job(&pool, job_id, JobControl::Resume)
        .await
        .is_err());
    assert!(control_job(&pool, job_id, JobControl::Pause).await.is_err());
    assert!(control_job(&pool, job_id, JobControl::Cancel)
        .await
        .is_err());

    cleanup(&pool, job_id).await;
}

#[tokio::test]
async fn e2e_spec091_batch_progress_does_not_resurrect_paused() {
    let Some(cfg) = require_or_skip_postgres("spec091_jobctl") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let step = format!(
        "e2e-jobctl-race-{}",
        &sqlx::types::Uuid::new_v4().to_string()[..8]
    );
    let job_id = seed_running_job(&pool, &step).await;

    // Race: operator pauses while the runner's batch is in flight; when the
    // batch commits, record_batch_progress must NOT flip paused → running.
    control_job(&pool, job_id, JobControl::Pause)
        .await
        .expect("pause");
    record_batch_progress(
        &pool,
        job_id,
        "e2e-owner",
        60,
        50,
        0,
        &serde_json::json!({"k": 50}),
        100,
        None,
    )
    .await
    .expect("record progress");
    assert_eq!(
        current_state(&pool, job_id).await.unwrap().as_deref(),
        Some("paused"),
        "batch commit must not resurrect a paused job"
    );

    // Progress still landed (data + ledger monotonicity preserved).
    let processed: i64 = sqlx::query_scalar(
        "SELECT processed_count FROM edgequake.edgequake_migration_job WHERE job_id = $1",
    )
    .bind(job_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(processed, 50);

    cleanup(&pool, job_id).await;
}

#[tokio::test]
async fn e2e_spec091_job_detail_rate_and_eta() {
    let Some(cfg) = require_or_skip_postgres("spec091_jobctl") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let step = format!(
        "e2e-jobctl-detail-{}",
        &sqlx::types::Uuid::new_v4().to_string()[..8]
    );
    let job_id = seed_running_job(&pool, &step).await;

    // Seed ledger: 2 batches, 100 rows in 200ms each → 500 rows/s.
    for (seq, from, to) in [
        (
            1i64,
            serde_json::json!({"k": 0}),
            serde_json::json!({"k": 100}),
        ),
        (
            2,
            serde_json::json!({"k": 100}),
            serde_json::json!({"k": 200}),
        ),
    ] {
        sqlx::query(
            "INSERT INTO edgequake.edgequake_migration_batch \
             (job_id, batch_seq, cursor_from, cursor_to, row_count, duration_ms) \
             VALUES ($1, $2, $3, $4, 100, 200)",
        )
        .bind(job_id)
        .bind(seq)
        .bind(from)
        .bind(to)
        .execute(&pool)
        .await
        .expect("seed batch");
    }
    sqlx::query(
        "UPDATE edgequake.edgequake_migration_job SET processed_count = 200 WHERE job_id = $1",
    )
    .bind(job_id)
    .execute(&pool)
    .await
    .unwrap();

    let detail = job_detail(&pool, job_id)
        .await
        .expect("detail")
        .expect("job exists");
    assert_eq!(detail.processed_count, 200);
    assert_eq!(detail.estimated_total, Some(1000));
    assert_eq!(detail.recent_batches.len(), 2);
    let rate = detail.rows_per_sec.expect("rate");
    assert!((rate - 500.0).abs() < 1.0, "rate ≈ 500 rows/s, got {rate}");
    let eta = detail.eta_seconds.expect("eta");
    assert!(
        (eta - 1.6).abs() < 0.01,
        "800 rows @ 500/s ≈ 1.6s, got {eta}"
    );
    let pct = detail.completion_pct.expect("pct");
    assert!((pct - 20.0).abs() < 0.01);

    // Unknown job → None.
    let missing = job_detail(&pool, sqlx::types::Uuid::new_v4())
        .await
        .unwrap();
    assert!(missing.is_none());

    cleanup(&pool, job_id).await;
}
