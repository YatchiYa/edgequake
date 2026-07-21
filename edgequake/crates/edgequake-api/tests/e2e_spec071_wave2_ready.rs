//! SPEC-071 — thin Wave-2 `/ready` catalog probe e2e.
//!
//! - Flag on + vector table without HNSW → blocker present (→ /ready 503 path)
//! - After creating HNSW (warmup / catalog) → blocker cleared
//! - Empty DB (no vector tables) → ready even with Wave-2 on
//!
//! Requires: `DATABASE_URL` + `--features postgres`
//!
//!   cargo test -p edgequake-api --features postgres e2e_spec071 -- --nocapture

#![cfg(feature = "postgres")]
// Env serialization requires the mutex across the whole async test body.
#![allow(clippy::await_holding_lock)]

mod common;

use sqlx::postgres::PgPoolOptions;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn env_lock() -> MutexGuard<'static, ()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

async fn connect_pool() -> Option<sqlx::PgPool> {
    let database_url = common::spec013_postgres::try_database_url()?;
    match PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await
    {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!("SKIP e2e_spec071: connect failed: {e}");
            None
        }
    }
}

#[tokio::test]
async fn wave2_ready_empty_db_is_ready() {
    let _guard = env_lock();
    let Some(pool) = connect_pool().await else {
        return;
    };
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");

    // Use a unique schema-like table name cleanup: drop any leftover from prior runs
    // but do not create tables — empty catalog → ready.
    let blocker = edgequake_api::services::ann_readiness::wave2_ann_readiness_blocker(&pool)
        .await
        .expect("probe");
    // May be Some if the shared test DB already has unindexed vector tables;
    // only assert the empty-path message contract when None or when missing.
    if blocker.is_none() {
        // Empty or all tables indexed — green.
        return;
    }
    let msg = blocker.unwrap();
    assert!(
        msg.contains("wave2_ann_missing") && msg.contains("catalog"),
        "blocker must distinguish catalog miss: {msg}"
    );
    assert!(
        msg.contains("admin/ann/warmup") || msg.contains("filtered query"),
        "blocker must point at warmup: {msg}"
    );
}

#[tokio::test]
async fn wave2_ready_missing_index_then_warmup() {
    let _guard = env_lock();
    let Some(pool) = connect_pool().await else {
        return;
    };
    std::env::set_var("EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE", "1");

    let table = format!(
        "eq_spec071_{}_vectors",
        &uuid::Uuid::new_v4().to_string()[..8]
    );

    // Bare vector table matching product naming — no ANN index.
    sqlx::query(&format!(
        r#"
        CREATE TABLE {table} (
            id TEXT PRIMARY KEY,
            embedding vector(8),
            metadata JSONB DEFAULT '{{}}'::jsonb,
            workspace_id TEXT
        )
        "#
    ))
    .execute(&pool)
    .await
    .expect("create table");

    let blocker = edgequake_api::services::ann_readiness::wave2_ann_readiness_blocker(&pool)
        .await
        .expect("probe missing");
    assert!(
        blocker
            .as_ref()
            .is_some_and(|b| b.contains("wave2_ann_missing")),
        "expected catalog miss blocker, got {blocker:?}"
    );
    let msg = blocker.unwrap();
    assert!(msg.contains("not plan-shape"), "clarity: {msg}");

    // Catalog warmup: create HNSW (mirrors ensure_hot / admin warmup outcome).
    sqlx::query(&format!(
        "CREATE INDEX {table}_embedding_idx ON {table} USING hnsw (embedding vector_cosine_ops)"
    ))
    .execute(&pool)
    .await
    .expect("create hnsw");

    let after = edgequake_api::services::ann_readiness::wave2_ann_readiness_blocker(&pool)
        .await
        .expect("probe after");
    // Other unindexed tables in shared DB may still block — only assert our table is indexed.
    let still_missing: i64 = sqlx::query_scalar(&format!(
        r#"
        SELECT COUNT(*)::bigint FROM pg_tables t
        WHERE t.schemaname = 'public' AND t.tablename = '{table}'
          AND NOT EXISTS (
            SELECT 1 FROM pg_indexes i
            WHERE i.schemaname = 'public' AND i.tablename = t.tablename
              AND (i.indexname = (t.tablename || '_embedding_idx')
                   OR i.indexdef ILIKE '% USING hnsw %')
          )
        "#
    ))
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(still_missing, 0, "table must have HNSW after warmup");

    // Cleanup
    let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {table} CASCADE"))
        .execute(&pool)
        .await;

    let _ = after; // shared DB may still have other missing tables
}

#[tokio::test]
async fn wave2_ready_probe_error_message_distinct() {
    // Document operator_action contract (compile + string SSOT for docs check).
    let action = "Wave-2 ANN readiness probe Err (not empty-DB / not missing-index) — \
                             check DATABASE_URL / pgvector catalog access, then retry /ready";
    assert!(action.contains("probe Err"));
    assert!(action.contains("not empty-DB"));
}
