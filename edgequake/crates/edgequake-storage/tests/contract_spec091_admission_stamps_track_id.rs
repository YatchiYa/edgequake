//! SPEC-091 Doc 23 KVH-AC-05: admission stamps `documents.track_id`.

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::ensure_admission_document_row_with_track;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use uuid::Uuid;

#[tokio::test]
async fn contract_spec091_admission_stamps_track_id() {
    let Some(cfg) = require_or_skip_postgres("kvh_admit") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let doc = Uuid::new_v4();
    let track = format!("insert-{}", Uuid::new_v4());

    ensure_admission_document_row_with_track(&pool, doc, None, None, "admit.md", Some(&track))
        .await
        .expect("admit");

    let got: Option<String> =
        sqlx::query_scalar("SELECT track_id FROM public.documents WHERE id = $1")
            .bind(doc)
            .fetch_one(&pool)
            .await
            .expect("read track_id");
    assert_eq!(got.as_deref(), Some(track.as_str()));

    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(doc)
        .execute(&pool)
        .await
        .ok();
}
