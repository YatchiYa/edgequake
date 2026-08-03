//! SPEC-091 IW0 (GAP-091-07): unclassified KV keys must fail LOUDLY once the
//! legacy KV relation is gone — never silently discard writes. Classified
//! families behind a stale `kv` rollback flag keep the EC-30 degrade (typed
//! authority wins, warn + skip).
//!
//! Post-drop conditions are reproduced by the fresh per-test namespace:
//! `initialize()` is a no-op since Wave D, so `eq_{ns}_kv` never exists.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test contract_spec091_unknown_family_loud
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::KVStorage;
use edgequake_storage::PostgresKVStorage;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};

#[tokio::test]
async fn unclassified_key_errors_loudly_post_drop() {
    let Some(cfg) = require_or_skip_postgres("unknownloud") else {
        return;
    };
    let _pool = contract_pg_pool(&cfg).await;
    let kv = PostgresKVStorage::new(cfg.clone());
    kv.initialize().await.expect("kv init");

    // "totally:unknown:key-1" matches no family in kv.rs::classify_key.
    let err = kv
        .upsert(&[(
            "totally:unknown:key-1".to_string(),
            serde_json::json!({"v": 1}),
        )])
        .await
        .expect_err("unclassified key must error post-drop, not silently vanish");

    let msg = err.to_string();
    assert!(
        msg.contains("totally:unknown:key-1"),
        "error must NAME the unclassified key (GAP-091-07); got: {msg}"
    );
    assert!(
        msg.contains("unclassified"),
        "error must explain the classification gap; got: {msg}"
    );
}

#[tokio::test]
async fn stale_kv_flag_on_classified_family_still_degrades() {
    let Some(cfg) = require_or_skip_postgres("unknownloud_degrade") else {
        return;
    };
    let _pool = contract_pg_pool(&cfg).await;

    // Rollback flag pointing at the dropped relation: EC-30 degrade stays a
    // typed-only no-op (never aborts the caller).
    std::env::set_var("EDGEQUAKE_KV_FAMILY_CACHE", "kv");
    let kv = PostgresKVStorage::new(cfg.clone());
    kv.initialize().await.expect("kv init");
    let cache_key = format!("{}-cache", "e".repeat(64));
    kv.upsert(&[(cache_key, serde_json::json!({"response": "x"}))])
        .await
        .expect("classified family behind stale kv flag must degrade (Ok), not error");
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_CACHE");
}

#[tokio::test]
async fn mixed_batch_with_unclassified_key_aborts() {
    let Some(cfg) = require_or_skip_postgres("unknownloud_mixed") else {
        return;
    };
    let _pool = contract_pg_pool(&cfg).await;

    std::env::set_var("EDGEQUAKE_KV_FAMILY_CACHE", "kv");
    let kv = PostgresKVStorage::new(cfg.clone());
    kv.initialize().await.expect("kv init");
    let err = kv
        .upsert(&[
            (
                format!("{}-cache", "f".repeat(64)),
                serde_json::json!({"r": 1}),
            ),
            ("another:unknown:2".to_string(), serde_json::json!({"v": 2})),
        ])
        .await
        .expect_err("mixed batch containing an unclassified key must abort loudly");
    assert!(
        err.to_string().contains("another:unknown:2"),
        "error must name the unclassified key; got: {err}"
    );
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_CACHE");
}
