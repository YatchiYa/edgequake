//! SPEC-091 Doc 23 KVH-AC-01/02: Absent → ping issues no SQL against `eq_*_kv`.
//!
//! Run:
//!   DATABASE_URL=... cargo test -p edgequake-storage --features postgres \
//!     --test contract_spec091_kv_ping_short_circuits -- --test-threads=1

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::adapters::postgres::PostgresKVStorage;
use edgequake_storage::traits::KVStorage;
use postgres_test_config::{contract_postgres_config, require_or_skip_postgres};

#[tokio::test]
async fn contract_spec091_kv_ping_short_circuits_when_dropped() {
    let Some(cfg) = require_or_skip_postgres("kvh_ping") else {
        return;
    };
    let _ = contract_postgres_config("kvh_ping");
    let kv = PostgresKVStorage::new(cfg);
    kv.initialize().await.expect("initialize");
    // Post-125 scratch DB has no eq_*_kv; initialize probes Absent via information_schema.
    assert!(
        kv.kv_relation_absent_cached(),
        "scratch DB must cache Absent after initialize probe"
    );

    kv.reset_kv_raw_sql_attempts();
    for _ in 0..20 {
        kv.ping().await.expect("ping");
    }
    assert_eq!(
        kv.kv_raw_sql_attempts(),
        0,
        "Absent cache must short-circuit ping to zero KV SQL (LAW-KVH1)"
    );
}

#[tokio::test]
async fn contract_spec091_kv_seed_from_dropped_skips_probe_sql() {
    let Some(cfg) = require_or_skip_postgres("kvh_seed") else {
        return;
    };
    let kv = PostgresKVStorage::new(cfg);
    kv.seed_relation_from_dropped(true);
    kv.reset_kv_raw_sql_attempts();
    kv.ping().await.expect("ping");
    assert_eq!(kv.kv_raw_sql_attempts(), 0);
    assert!(kv.kv_relation_absent_cached());
}
