//! SPEC-091 RM0 — outbox drain consumer (RM-AC-01 / RM-AC-02).
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use edgequake_storage::drain_claim::DrainMode;
use edgequake_storage::outbox_drain::{
    chaos_claim_without_ack, drain_once, outbox_drain_processed_total, outbox_lag_seconds,
    OutboxDrainConfig, OutboxEvent,
};
use edgequake_storage::{
    OutboxSink, PostgresOutboxSink, OUTBOX_AGGREGATE_DOCUMENT, OUTBOX_EVENT_CHUNK_READY,
    OUTBOX_EVENT_COMPENSATE, OUTBOX_EVENT_MERGE_DONE,
};
use sqlx::PgPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

fn require_db() -> Option<String> {
    let base = std::env::var("DATABASE_URL").ok()?;
    if base.trim().is_empty() {
        return None;
    }
    Some(test_db::isolated_test_url(&base))
}

async fn has_drain_columns(pool: &PgPool) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = 'outbox_events'
              AND column_name = 'available_at'
        )",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

#[tokio::test]
async fn contract_spec091_outbox_drain_marks_processed() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    assert!(
        has_drain_columns(&pool).await,
        "migration 134 must add available_at"
    );

    let sink = PostgresOutboxSink::new(pool.clone());
    let doc = Uuid::new_v4();
    let ws = Uuid::new_v4();
    for ev in [
        OUTBOX_EVENT_CHUNK_READY,
        OUTBOX_EVENT_MERGE_DONE,
        OUTBOX_EVENT_COMPENSATE,
    ] {
        sink.enqueue(
            OUTBOX_AGGREGATE_DOCUMENT,
            doc,
            ev,
            serde_json::json!({ "document_id": doc.to_string() }),
            Some(ws),
        )
        .await
        .expect("enqueue");
    }

    let before = outbox_drain_processed_total();
    let cfg = OutboxDrainConfig {
        mode: DrainMode::On,
        interval: std::time::Duration::from_secs(30),
        batch: 50,
        max_attempts: 6,
        ttl_days: 7,
        workspace_id: Some(ws),
    };
    let ack = AtomicUsize::new(0);
    drain_once(&pool, &cfg, &|_event: OutboxEvent| {
        ack.fetch_add(1, Ordering::SeqCst);
        async move { Ok(()) }
    })
    .await
    .expect("drain");

    assert_eq!(ack.load(Ordering::SeqCst), 3, "applier sees all milestones");
    assert!(
        outbox_drain_processed_total() >= before + 3,
        "processed metric advances"
    );

    let unprocessed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND processed_at IS NULL",
    )
    .bind(doc)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(unprocessed, 0, "RM-AC-01: all milestones marked processed");

    let lag = outbox_lag_seconds(&pool).await.expect("lag");
    assert!(lag >= 0, "lag metric path works");
}

#[tokio::test]
async fn contract_spec091_outbox_drain_chaos_abort_resumes() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    if !has_drain_columns(&pool).await {
        eprintln!("skip: migration 134 missing");
        return;
    }

    let sink = PostgresOutboxSink::new(pool.clone());
    let doc = Uuid::new_v4();
    sink.enqueue(
        OUTBOX_AGGREGATE_DOCUMENT,
        doc,
        OUTBOX_EVENT_CHUNK_READY,
        serde_json::json!({}),
        None,
    )
    .await
    .expect("enqueue");

    // Simulate kill mid-claim: bump attempt_count, leave processed_at NULL.
    let n = chaos_claim_without_ack(&pool, 10)
        .await
        .expect("chaos claim");
    assert!(n >= 1, "claimed at least the test row");

    let still: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND processed_at IS NULL",
    )
    .bind(doc)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(still, 1, "abort leaves row unprocessed (no double-ack)");

    let cfg = OutboxDrainConfig {
        mode: DrainMode::On,
        interval: std::time::Duration::from_secs(30),
        batch: 50,
        max_attempts: 6,
        ttl_days: 7,
        workspace_id: None,
    };
    drain_once(&pool, &cfg, &|_e: OutboxEvent| async move { Ok(()) })
        .await
        .expect("resume drain");

    let left: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND processed_at IS NULL",
    )
    .bind(doc)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(
        left, 0,
        "RM-AC-02: resume processes without duplicate side effect"
    );
}
