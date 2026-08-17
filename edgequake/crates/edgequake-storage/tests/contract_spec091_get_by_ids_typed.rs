//! SPEC-091 IW0 (GAP-091-04): `get_by_ids` must route through the SAME
//! typed-first merge pipeline as `get_by_ids_ordered` — cache → shell → chunk
//! → KV fallback — so document downloads keep working on post-125 databases
//! where `eq_*_kv` is dropped and every family home is a typed table.
//!
//! The fresh per-test namespace never had an `eq_{ns}_kv` relation
//! (`initialize()` is a no-op since Wave D), which reproduces post-drop
//! conditions exactly: any KV-only read path returns nothing.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test contract_spec091_get_by_ids_typed
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::adapters::postgres::document_shell::dual_write_shell_upserts;
use edgequake_storage::adapters::postgres::llm_cache::cache_upsert;
use edgequake_storage::traits::KVStorage;
use edgequake_storage::PostgresKVStorage;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use uuid::Uuid;

#[tokio::test]
async fn get_by_ids_resolves_typed_shell_and_cache_post_drop() {
    let Some(cfg) = require_or_skip_postgres("getbyids_typed") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    // Seed typed homes directly (no KV anywhere — post-125 conditions).
    let doc = Uuid::new_v4();
    let metadata_key = format!("{doc}-metadata");
    let cache_key = format!("{}-cache", "d".repeat(64));

    dual_write_shell_upserts(
        &pool,
        &[(
            metadata_key.clone(),
            serde_json::json!({"title": "Post-Drop Download", "status": "completed"}),
        )],
        true,
    )
    .await
    .expect("seed shell");
    cache_upsert(
        &pool,
        &cfg.namespace,
        &[(
            cache_key.clone(),
            serde_json::json!({"response": "typed-cache"}),
        )],
    )
    .await
    .expect("seed cache");

    // Sanity: the legacy KV relation for this namespace does NOT exist.
    let kv = PostgresKVStorage::new(cfg.clone());
    kv.initialize().await.expect("kv init");
    let table = format!("eq_{}_kv", cfg.namespace.replace('-', "_"));
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = $1)",
    )
    .bind(&table)
    .fetch_one(&pool)
    .await
    .expect("pg_tables");
    assert!(!exists, "test requires post-drop conditions (no {table})");

    // The regression: pre-fix this returned an EMPTY vec (KV-only INNER JOIN),
    // 404-ing document downloads on post-125 databases.
    let values = kv
        .get_by_ids(&[
            metadata_key.clone(),
            cache_key.clone(),
            format!("{doc}-nonexistent"),
        ])
        .await
        .expect("get_by_ids");

    assert_eq!(
        values.len(),
        2,
        "typed values must resolve even with the KV relation dropped; got {values:?}"
    );
    assert_eq!(
        values[0].get("title").and_then(|v| v.as_str()),
        Some("Post-Drop Download"),
        "shell metadata must come first (input order preserved)"
    );
    assert_eq!(
        values[1].get("response").and_then(|v| v.as_str()),
        Some("typed-cache"),
        "cache value must follow in input order"
    );
}

#[tokio::test]
async fn get_by_ids_ordered_and_unordered_agree_on_typed_values() {
    let Some(cfg) = require_or_skip_postgres("getbyids_parity") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    let doc = Uuid::new_v4();
    let metadata_key = format!("{doc}-metadata");
    dual_write_shell_upserts(
        &pool,
        &[(
            metadata_key.clone(),
            serde_json::json!({"title": "Parity Doc"}),
        )],
        true,
    )
    .await
    .expect("seed shell");

    let kv = PostgresKVStorage::new(cfg.clone());
    kv.initialize().await.expect("kv init");
    let ids = vec![metadata_key, format!("{doc}-missing")];

    let compacted = kv.get_by_ids(&ids).await.expect("get_by_ids");
    let ordered = kv
        .get_by_ids_ordered(&ids)
        .await
        .expect("get_by_ids_ordered");

    // DRY contract: unordered = ordered minus the None slots, same order.
    let ordered_flattened: Vec<_> = ordered.into_iter().flatten().collect();
    assert_eq!(compacted, ordered_flattened);
    assert_eq!(compacted.len(), 1);
}
