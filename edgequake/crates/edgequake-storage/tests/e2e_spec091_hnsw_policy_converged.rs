//! SPEC-091 IW1 (GAP-091-25 / LD-06): HNSW `ef_construction` is converged.
//!
//! Pins the single build policy: runtime default = 128, migration 129 ledger
//! row present, and `docker/init.sql` no longer introduces a third value.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::hnsw_ef_construction_from_env;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};

#[test]
fn runtime_default_ef_construction_is_128() {
    // Unset override so the compiled default is what we assert.
    let prev = std::env::var("EDGEQUAKE_HNSW_EF_CONSTRUCTION").ok();
    std::env::remove_var("EDGEQUAKE_HNSW_EF_CONSTRUCTION");
    assert_eq!(
        hnsw_ef_construction_from_env(),
        128,
        "LD-06: runtime SSOT must be 128"
    );
    if let Some(v) = prev {
        std::env::set_var("EDGEQUAKE_HNSW_EF_CONSTRUCTION", v);
    }
}

#[test]
fn init_sql_does_not_introduce_third_ef_value() {
    // tests/ → edgequake-storage → crates → edgequake/docker/init.sql
    let init = include_str!("../../../docker/init.sql");
    assert!(
        !init.contains("ef_construction = 64"),
        "init.sql must not keep the historical ef_construction=64 third value"
    );
    assert!(
        init.contains("ef_construction = 128"),
        "init.sql must use the converged ef_construction=128"
    );
}

#[tokio::test]
async fn schema_generation_ledger_records_converged_policy() {
    let Some(cfg) = require_or_skip_postgres("spec091_hnsw_policy") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    // Ensure migration 129 applied (migrator in contract_pg_pool).
    let notes: Option<String> = sqlx::query_scalar(
        "SELECT notes FROM public.edgequake_schema_generation \
         WHERE relation_name = 'chunk_embeddings.hnsw'",
    )
    .fetch_optional(&pool)
    .await
    .expect("ledger query");

    let notes = notes.expect("migration 129/132 must insert chunk_embeddings.hnsw ledger row");
    assert!(
        notes.contains("ef_construction=128"),
        "ledger notes must cite ef_construction=128, got: {notes}"
    );

    let has_hnsw: bool = sqlx::query_scalar(
        "SELECT EXISTS (\
           SELECT 1 FROM pg_indexes \
           WHERE schemaname = 'public' \
             AND tablename = 'chunk_embeddings' \
             AND indexname IN (\
               'idx_chunk_embeddings_hnsw_d768',\
               'idx_chunk_embeddings_hnsw_d1024',\
               'idx_chunk_embeddings_hnsw_d1536'\
             )\
         )",
    )
    .fetch_one(&pool)
    .await
    .expect("index probe");
    assert!(
        has_hnsw,
        "dim-scoped chunk_embeddings HNSW indexes must exist after migration 132"
    );
}
