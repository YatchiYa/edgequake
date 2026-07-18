//! SPEC-059 Wave 1 — atomic upsert_report_created (xmax) under concurrency.

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::PgVectorStorage;
use std::collections::HashSet;
use std::sync::Arc;

fn emb(seed: f32) -> Vec<f32> {
    vec![seed, seed + 0.1, seed + 0.2, seed + 0.3]
}

#[tokio::test]
async fn spec059_upsert_report_created_concurrent_at_most_one_insert() {
    let Some(config) =
        postgres_test_config::contract_postgres_config("spec059_upsert_report_created")
    else {
        if std::env::var("EDGEQUAKE_REQUIRE_POSTGRES_TESTS").as_deref() == Ok("1") {
            panic!("DATABASE_URL or POSTGRES_PASSWORD required");
        }
        eprintln!("SKIP spec059_upsert_report_created: no DATABASE_URL / POSTGRES_PASSWORD");
        return;
    };

    let storage = Arc::new(PgVectorStorage::with_dimension(config, 4));
    storage.initialize().await.expect("vector init");

    let shared_id = "spec059-shared-entity".to_string();
    let n_workers = 8usize;
    let mut handles = Vec::with_capacity(n_workers);
    for w in 0..n_workers {
        let storage = Arc::clone(&storage);
        let id = shared_id.clone();
        handles.push(tokio::spawn(async move {
            let data = [(
                id,
                emb(w as f32),
                serde_json::json!({"type": "entity", "worker": w}),
            )];
            storage.upsert_report_created(&data).await
        }));
    }

    let mut created_reports = 0usize;
    let mut creators = HashSet::new();
    for h in handles {
        let created = h.await.expect("join").expect("upsert");
        if created.contains(&shared_id) {
            created_reports += 1;
            creators.insert(created.clone());
        }
    }

    assert_eq!(
        created_reports, 1,
        "exactly one worker must report insert for the shared id (got {created_reports})"
    );
    assert!(
        storage.get_by_id(&shared_id).await.unwrap().is_some(),
        "shared vector must exist after concurrent upserts"
    );

    // Compensate of a "loser" that incorrectly listed the id would delete it —
    // with xmax, losers have empty created lists so compensate is a no-op.
    let losers_created: Vec<String> = Vec::new();
    storage.delete(&losers_created).await.unwrap();
    assert!(
        storage.get_by_id(&shared_id).await.unwrap().is_some(),
        "vector must survive compensate of empty loser artifact list"
    );

    storage.delete(&[shared_id]).await.ok();
}

#[tokio::test]
async fn spec059_upsert_report_created_sequential_update_empty() {
    let Some(config) =
        postgres_test_config::contract_postgres_config("spec059_upsert_report_seq")
    else {
        if std::env::var("EDGEQUAKE_REQUIRE_POSTGRES_TESTS").as_deref() == Ok("1") {
            panic!("DATABASE_URL or POSTGRES_PASSWORD required");
        }
        eprintln!("SKIP spec059_upsert_report_seq: no DATABASE_URL / POSTGRES_PASSWORD");
        return;
    };

    let storage = PgVectorStorage::with_dimension(config, 4);
    storage.initialize().await.expect("vector init");
    let id = "spec059-seq".to_string();
    let first = storage
        .upsert_report_created(&[(
            id.clone(),
            emb(1.0),
            serde_json::json!({"type": "entity"}),
        )])
        .await
        .expect("first");
    assert_eq!(first, vec![id.clone()]);
    let second = storage
        .upsert_report_created(&[(
            id.clone(),
            emb(2.0),
            serde_json::json!({"type": "entity", "v": 2}),
        )])
        .await
        .expect("second");
    assert!(second.is_empty(), "update must not report created: {second:?}");
    storage.delete(&[id]).await.ok();
}
