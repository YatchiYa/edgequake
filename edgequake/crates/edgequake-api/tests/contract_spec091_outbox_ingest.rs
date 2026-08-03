//! SPEC-091 IP2 — outbox writers (IP-AC-06).
#![cfg(feature = "postgres")]

#[path = "common/test_db.rs"]
mod test_db;

use edgequake_storage::{
    OutboxSink, PostgresOutboxSink, OUTBOX_AGGREGATE_DOCUMENT, OUTBOX_EVENT_CHUNK_READY,
    OUTBOX_EVENT_COMPENSATE, OUTBOX_EVENT_MERGE_DONE,
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

#[tokio::test]
async fn contract_spec091_outbox_enqueue_milestones() {
    let Some(url) = require_db() else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    // Ensure migration 133 applied (workspace_id column).
    let has_ws: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = 'outbox_events'
              AND column_name = 'workspace_id'
        )",
    )
    .fetch_one(&pool)
    .await
    .expect("column check");
    assert!(has_ws, "migration 133 must add outbox_events.workspace_id");

    let sink = PostgresOutboxSink::new(pool.clone());
    let doc = Uuid::new_v4();
    let ws = Uuid::new_v4();

    sink.enqueue(
        OUTBOX_AGGREGATE_DOCUMENT,
        doc,
        OUTBOX_EVENT_CHUNK_READY,
        serde_json::json!({ "document_id": doc.to_string() }),
        Some(ws),
    )
    .await
    .expect("chunk_ready");
    sink.enqueue(
        OUTBOX_AGGREGATE_DOCUMENT,
        doc,
        OUTBOX_EVENT_MERGE_DONE,
        serde_json::json!({ "document_id": doc.to_string() }),
        Some(ws),
    )
    .await
    .expect("merge_done");
    sink.enqueue(
        OUTBOX_AGGREGATE_DOCUMENT,
        doc,
        OUTBOX_EVENT_COMPENSATE,
        serde_json::json!({ "cause": "test" }),
        Some(ws),
    )
    .await
    .expect("compensate");

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events
         WHERE aggregate_id = $1 AND event_type = ANY($2)",
    )
    .bind(doc)
    .bind(
        &[
            OUTBOX_EVENT_CHUNK_READY.to_string(),
            OUTBOX_EVENT_MERGE_DONE.to_string(),
            OUTBOX_EVENT_COMPENSATE.to_string(),
        ][..],
    )
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(n, 3, "IP-AC-06: each milestone inserts an outbox row");
}
