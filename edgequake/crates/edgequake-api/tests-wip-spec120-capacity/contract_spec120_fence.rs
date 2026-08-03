//! Contract: SPEC-120 P0/A3 stale writers are rejected after an epoch bump.

#![cfg(feature = "postgres")]

use std::time::Duration;

use edgequake_api::services::{
    assert_fence, bump_fence_epoch, read_fence_epoch, FenceEpoch, FenceError,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::test]
async fn bump_rejects_writer_holding_old_epoch() {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping SPEC-120 fence contract: DATABASE_URL is not set");
            return;
        }
    };
    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("Skipping SPEC-120 fence contract: PostgreSQL unavailable: {error}");
            return;
        }
    };

    let document_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO documents (id, title, content, status, fence_epoch)
         VALUES ($1, 'SPEC-120 fence contract', '', 'pending', 0)",
    )
    .bind(document_id)
    .execute(&pool)
    .await
    .expect("spec120 cancel-fence migration and documents table must be present");

    let document_id = document_id.to_string();
    let held = read_fence_epoch(&document_id, Some(&pool))
        .await
        .expect("initial fence must be readable");
    assert_eq!(held, FenceEpoch(0));

    let bumped = bump_fence_epoch(&document_id, Some(&pool))
        .await
        .expect("fence bump must succeed");
    assert_eq!(bumped, FenceEpoch(1));

    let error = assert_fence(held, &document_id, Some(&pool))
        .await
        .expect_err("old epoch must be rejected");
    assert!(matches!(
        error,
        FenceError::Stale {
            expected: 0,
            actual: 1
        }
    ));

    // INV-2: a second bump still rejects the original held epoch.
    let bumped_again = bump_fence_epoch(&document_id, Some(&pool))
        .await
        .expect("second fence bump must succeed");
    assert_eq!(bumped_again, FenceEpoch(2));
    let error = assert_fence(held, &document_id, Some(&pool))
        .await
        .expect_err("original epoch must still be rejected");
    assert!(matches!(
        error,
        FenceError::Stale {
            expected: 0,
            actual: 2
        }
    ));
    // Matching the current epoch succeeds (legitimate writer after re-read).
    assert_fence(bumped_again, &document_id, Some(&pool))
        .await
        .expect("current epoch must be accepted");

    sqlx::query("DELETE FROM documents WHERE id::text = $1")
        .bind(&document_id)
        .execute(&pool)
        .await
        .expect("test document cleanup must succeed");
}
