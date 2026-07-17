//! SPEC-057 P1: Postgres SKIP LOCKED claim / lease e2e.
//!
//! Run with:
//!   DATABASE_URL=... cargo test -p edgequake-tasks --features postgres --test postgres_claim_lease
//!
//! Skips cleanly when DATABASE_URL / POSTGRES_PASSWORD is unset.

#![cfg(feature = "postgres")]

use std::env;
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use edgequake_tasks::postgres::PostgresTaskStorage;
use edgequake_tasks::{Task, TaskStatus, TaskStorage, TaskType};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

fn get_database_url() -> Option<String> {
    env::var("DATABASE_URL").ok().or_else(|| {
        let password = env::var("POSTGRES_PASSWORD").ok()?;
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "edgequake".to_string());
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "edgequake".to_string());
        Some(format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, host, port, db
        ))
    })
}

async fn create_test_pool() -> Option<PgPool> {
    let database_url = get_database_url()?;
    PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await
        .ok()
}

/// Ensure mig 088 lease columns exist (idempotent).
async fn ensure_lease_columns(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_owner TEXT")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_token UUID")
        .execute(pool)
        .await?;
    sqlx::query("ALTER TABLE tasks ADD COLUMN IF NOT EXISTS lease_expires_at TIMESTAMPTZ")
        .execute(pool)
        .await?;
    Ok(())
}

/// Make this row the oldest claimable candidate (shared-DB safe).
async fn make_oldest(pool: &PgPool, track_id: &str) -> Result<(), sqlx::Error> {
    let epoch = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    sqlx::query("UPDATE tasks SET created_at = $2 WHERE track_id = $1")
        .bind(track_id)
        .bind(epoch)
        .execute(pool)
        .await?;
    Ok(())
}

macro_rules! require_postgres {
    () => {
        match create_test_pool().await {
            Some(pool) => {
                if let Err(e) = ensure_lease_columns(&pool).await {
                    eprintln!("Skipping: cannot ensure lease columns: {e}");
                    return;
                }
                if sqlx::query("SELECT 1 FROM tasks LIMIT 0")
                    .execute(&pool)
                    .await
                    .is_err()
                {
                    eprintln!("Skipping: tasks table missing — run migrations first");
                    return;
                }
                pool
            }
            None => {
                eprintln!("Skipping: DATABASE_URL or POSTGRES_PASSWORD not set");
                return;
            }
        }
    };
}

fn sample_task(status: TaskStatus) -> Task {
    let mut task = Task::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskType::Insert,
        serde_json::json!({ "document_id": format!("claim-e2e-{}", Uuid::new_v4()) }),
    );
    task.status = status;
    task
}

async fn cleanup(pool: &PgPool, track_id: &str) {
    let _ = sqlx::query("DELETE FROM tasks WHERE track_id = $1")
        .bind(track_id)
        .execute(pool)
        .await;
}

async fn release_if_held(storage: &PostgresTaskStorage, task: &Task, worker: &str) {
    if let Some(token) = task.lease_token {
        let _ = storage.release_claim(&task.track_id, worker, token).await;
    }
}

#[tokio::test]
async fn postgres_claim_next_pending_without_wake() {
    let pool = require_postgres!();
    let storage = PostgresTaskStorage::new(pool.clone());
    let task = sample_task(TaskStatus::Pending);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.expect("create");
    make_oldest(&pool, &track_id).await.expect("oldest");

    let claimed = storage
        .claim_next("pg-worker-a", Duration::from_secs(120))
        .await
        .expect("claim")
        .expect("Pending must be claimable without channel wake");
    assert_eq!(claimed.track_id, track_id);
    assert_eq!(claimed.status, TaskStatus::Processing);
    assert_eq!(claimed.lease_owner.as_deref(), Some("pg-worker-a"));
    assert!(claimed.lease_token.is_some());

    cleanup(&pool, &track_id).await;
}

#[tokio::test]
async fn postgres_dual_claim_next_race_one_winner() {
    let pool = require_postgres!();
    let storage = Arc::new(PostgresTaskStorage::new(pool.clone()));
    let task = sample_task(TaskStatus::Pending);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.expect("create");
    make_oldest(&pool, &track_id).await.expect("oldest");

    let s1 = Arc::clone(&storage);
    let s2 = Arc::clone(&storage);
    let (a, b) = tokio::join!(
        s1.claim_next("race-w1", Duration::from_secs(120)),
        s2.claim_next("race-w2", Duration::from_secs(120)),
    );

    let a = a.expect("claim a");
    let b = b.expect("claim b");
    let a_ours = a.as_ref().map(|t| t.track_id.as_str()) == Some(track_id.as_str());
    let b_ours = b.as_ref().map(|t| t.track_id.as_str()) == Some(track_id.as_str());
    assert!(
        a_ours ^ b_ours,
        "exactly one worker must claim our track; a={a:?} b={b:?}"
    );

    // Release any non-target claim so we don't strand foreign Pending rows.
    if let Some(ref t) = a {
        if !a_ours {
            release_if_held(&s1, t, "race-w1").await;
        }
    }
    if let Some(ref t) = b {
        if !b_ours {
            release_if_held(&s2, t, "race-w2").await;
        }
    }

    cleanup(&pool, &track_id).await;
}

#[tokio::test]
async fn postgres_cancelled_never_claimed() {
    let pool = require_postgres!();
    let storage = PostgresTaskStorage::new(pool.clone());
    let mut task = sample_task(TaskStatus::Cancelled);
    task.mark_cancelled();
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.expect("create");
    make_oldest(&pool, &track_id).await.expect("oldest");

    // Oldest is Cancelled — claim must skip it (may return next Pending or None).
    let claimed = storage
        .claim_next("pg-worker-c", Duration::from_secs(120))
        .await
        .expect("claim");
    assert!(
        claimed.as_ref().map(|t| t.track_id.as_str()) != Some(track_id.as_str()),
        "Cancelled must never be claimed"
    );
    if let Some(ref t) = claimed {
        release_if_held(&storage, t, "pg-worker-c").await;
    }

    cleanup(&pool, &track_id).await;
}

#[tokio::test]
async fn postgres_refresh_lease_and_release_claim_cas() {
    let pool = require_postgres!();
    let storage = PostgresTaskStorage::new(pool.clone());
    let task = sample_task(TaskStatus::Pending);
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.expect("create");
    make_oldest(&pool, &track_id).await.expect("oldest");

    let claimed = storage
        .claim_next("owner", Duration::from_secs(120))
        .await
        .expect("claim")
        .expect("claimed");
    assert_eq!(claimed.track_id, track_id);
    let token = claimed.lease_token.expect("token");

    assert!(storage
        .refresh_lease(&track_id, "owner", token, Duration::from_secs(120))
        .await
        .expect("refresh"));
    assert!(!storage
        .refresh_lease(&track_id, "intruder", token, Duration::from_secs(120))
        .await
        .expect("refresh wrong owner"));

    assert!(storage
        .release_claim(&track_id, "owner", token)
        .await
        .expect("release"));
    let pending = storage.get_task(&track_id).await.expect("get").unwrap();
    assert_eq!(pending.status, TaskStatus::Pending);
    assert!(pending.lease_owner.is_none());

    cleanup(&pool, &track_id).await;
}

#[tokio::test]
async fn postgres_claim_reclaims_expired_processing() {
    let pool = require_postgres!();
    let storage = PostgresTaskStorage::new(pool.clone());
    let mut task = sample_task(TaskStatus::Processing);
    task.mark_processing();
    task.lease_owner = Some("dead-worker".into());
    task.lease_token = Some(Uuid::new_v4());
    task.lease_expires_at = Some(Utc::now() - chrono::Duration::seconds(5));
    let track_id = task.track_id.clone();
    storage.create_task(&task).await.expect("create");
    make_oldest(&pool, &track_id).await.expect("oldest");

    let claimed = storage
        .claim_next("alive-worker", Duration::from_secs(120))
        .await
        .expect("claim")
        .expect("expired Processing must be reclaimable");
    assert_eq!(claimed.track_id, track_id);
    assert_eq!(claimed.lease_owner.as_deref(), Some("alive-worker"));

    cleanup(&pool, &track_id).await;
}
