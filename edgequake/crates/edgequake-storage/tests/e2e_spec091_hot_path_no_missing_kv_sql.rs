//! SPEC-091 Doc 23 KVH-AC-10: list/hydrate/wipe-style KV calls under drop → 0 SQL.
//!
//! Run:
//!   DATABASE_URL=... cargo test -p edgequake-storage --features postgres \
//!     --test e2e_spec091_hot_path_no_missing_kv_sql -- --test-threads=1

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use std::collections::HashSet;

use edgequake_storage::adapters::postgres::{PostgresKVStorage, PostgresPool};
use edgequake_storage::traits::KVStorage;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn e2e_spec091_hot_path_no_missing_kv_sql() {
    let Some(cfg) = require_or_skip_postgres("kvh_hot") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let kv = PostgresKVStorage::with_pool(PostgresPool::from_existing(pool.clone(), cfg.clone()), cfg);
    kv.seed_relation_from_dropped(true);
    kv.reset_kv_raw_sql_attempts();

    let doc = Uuid::new_v4();
    let ids = vec![
        format!("{doc}-metadata"),
        format!("{doc}-chunk-0"),
        format!("wsdoc:x:{doc}"),
    ];

    let _ = kv.get_by_id(&ids[0]).await.expect("get");
    let _ = kv.get_by_ids(&ids).await.expect("get_by_ids");
    let _ = kv.get_by_ids_ordered(&ids).await.expect("ordered");
    let _ = kv
        .filter_keys(ids.iter().cloned().collect::<HashSet<_>>())
        .await
        .expect("filter");
    let _ = kv.keys_like("%-metadata").await.expect("keys_like");
    let _ = kv
        .keys_with_prefix_limited(&format!("{doc}-chunk-"), 10)
        .await
        .expect("prefix");
    let _ = kv.keys_with_suffix_limited("-metadata", 10).await.expect("suffix");
    let _ = kv.count().await.expect("count");
    let _ = kv.is_empty().await.expect("empty");
    let _ = kv
        .count_embedded_chunks_for_docs(&[doc.to_string()])
        .await
        .expect("embedded");
    let _ = kv
        .upsert(&[(format!("{doc}-metadata"), json!({"title": "t", "status": "pending"}))])
        .await
        .expect("upsert shell");
    let _ = kv.delete(&ids).await.expect("delete");
    let _ = kv.clear().await.expect("clear");
    let _ = kv
        .transition_if_status(&format!("{doc}-metadata"), "pending", "processing")
        .await
        .expect("cas");

    assert_eq!(
        kv.kv_raw_sql_attempts(),
        0,
        "post-drop hot path must not issue SQL against missing eq_*_kv"
    );

    // Relational chunk count path may hit `chunks` (not counted as kv_raw_sql).
    let _ = pool; // keep pool alive for FK-safe typed writes above
}
