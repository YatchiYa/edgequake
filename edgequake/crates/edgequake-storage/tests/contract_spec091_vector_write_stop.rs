//! SPEC-091: legacy vector write-stop under `typed_embeddings`.
//!
//! When typed is authority, `PgVectorStorage::initialize` must not CREATE
//! missing workspace `eq_*_vectors` tables, and `upsert` / `upsert_report_created`
//! must succeed without INSERT (so KG persist no longer 42P01s).
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test contract_spec091_vector_write_stop -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::adapters::postgres::{PgVectorStorage, PostgresPool};
use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{vector_backend_from_env, VECTOR_BACKEND_ENV};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use uuid::Uuid;

async fn table_exists(pool: &sqlx::PgPool, table: &str) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = $1)",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn typed_backend_write_stops_legacy_upsert_without_table() {
    let Some(cfg) = require_or_skip_postgres("spec091_vws") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let ns = format!("vws_{}", Uuid::new_v4().as_simple());

    std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");
    assert!(matches!(
        vector_backend_from_env(),
        edgequake_storage::VectorBackend::TypedEmbeddings
    ));

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns.clone();
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        8,
    );
    let table = storage
        .vectors_table_name()
        .trim_start_matches("public.")
        .to_string();
    assert!(
        !table_exists(&pool, &table).await,
        "precondition: legacy table {table} must be absent"
    );

    storage
        .initialize()
        .await
        .expect("initialize must not fail when typed write-stops CREATE");
    assert!(
        !table_exists(&pool, &table).await,
        "typed backend must not CREATE legacy eq_*_vectors"
    );

    let emb = vec![0.1_f32; 8];
    let batch = vec![(
        format!("{}-chunk-0", Uuid::new_v4()),
        emb,
        json!({"document_id": "doc", "workspace_id": Uuid::new_v4().to_string()}),
    )];
    storage
        .upsert(&batch)
        .await
        .expect("typed write-stop upsert must not 42P01");
    let created = storage
        .upsert_report_created(&batch)
        .await
        .expect("typed write-stop upsert_report_created must not 42P01");
    assert!(
        created.is_empty(),
        "write-stop returns no legacy-created ids"
    );
    assert!(
        !table_exists(&pool, &table).await,
        "upsert must not create the legacy table either"
    );

    std::env::remove_var(VECTOR_BACKEND_ENV);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn legacy_tables_backend_still_creates_and_upserts() {
    let Some(cfg) = require_or_skip_postgres("spec091_vws_legacy") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let ns = format!("vwsl_{}", Uuid::new_v4().as_simple());

    std::env::set_var(VECTOR_BACKEND_ENV, "legacy_tables");

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns;
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        8,
    );
    // Physical name is `eq_{prefix}_vectors` where prefix = `eq_{namespace}`.
    let table = storage
        .vectors_table_name()
        .trim_start_matches("public.")
        .to_string();

    storage.initialize().await.expect("legacy initialize");
    assert!(
        table_exists(&pool, &table).await,
        "legacy_tables backend must CREATE {table}"
    );

    let emb = vec![0.2_f32; 8];
    let id = format!("{}-chunk-0", Uuid::new_v4());
    storage
        .upsert(&[(id.clone(), emb, json!({}))])
        .await
        .expect("legacy upsert");

    let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM public.{table}"))
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 1);

    sqlx::query(&format!("DROP TABLE IF EXISTS public.{table}"))
        .execute(&pool)
        .await
        .ok();
    std::env::remove_var(VECTOR_BACKEND_ENV);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn typed_backend_write_stops_legacy_mutates_without_table() {
    let Some(cfg) = require_or_skip_postgres("spec091_vws_mutate") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let ns = format!("vwsm_{}", Uuid::new_v4().as_simple());
    std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns;
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        8,
    );
    let table = storage
        .vectors_table_name()
        .trim_start_matches("public.")
        .to_string();
    assert!(
        !table_exists(&pool, &table).await,
        "precondition: legacy table {table} must be absent"
    );

    storage
        .delete_by_document("doc-missing")
        .await
        .expect("delete_by_document must not 42P01 under typed");
    storage
        .clear_workspace(&Uuid::new_v4())
        .await
        .expect("clear_workspace must not 42P01 under typed");
    storage
        .delete_entity("ENTITY_X")
        .await
        .expect("delete_entity must not 42P01 under typed");
    storage
        .delete_entities_batch(&["A".into(), "B".into()])
        .await
        .expect("delete_entities_batch must not 42P01 under typed");
    storage
        .delete_entity_relations("ENTITY_X")
        .await
        .expect("delete_entity_relations must not 42P01 under typed");
    storage
        .clear()
        .await
        .expect("clear must not 42P01 under typed");
    storage
        .delete(&["id-1".into(), "id-2".into()])
        .await
        .expect("delete(ids) must not 42P01 under typed");

    assert!(
        !table_exists(&pool, &table).await,
        "mutates must not CREATE the legacy table"
    );
    std::env::remove_var(VECTOR_BACKEND_ENV);
}

#[test]
fn contract_spec383_mutates_probe_before_delete() {
    let src = include_str!("../src/adapters/postgres/vector/storage_impl.rs");
    let helper = include_str!("../src/adapters/postgres/vector/mod.rs");
    assert!(
        helper.contains("fn skip_legacy_mutate_if_absent"),
        "PgVectorStorage must expose skip_legacy_mutate_if_absent"
    );
    assert!(
        helper.contains("fn legacy_vectors_relation_exists_cached"),
        "existence probe must be cached (DRY with chunk_kv_table_exists)"
    );
    for (fn_name, op) in [
        ("async fn delete(", "Delete"),
        ("async fn delete_entity(", "Delete entity"),
        ("async fn delete_entities_batch(", "Batch delete entities"),
        (
            "async fn delete_entity_relations(",
            "Delete entity relations",
        ),
        ("async fn clear(", "Clear"),
        ("async fn clear_workspace(", "Clear workspace"),
        ("async fn delete_by_document(", "Delete by document"),
    ] {
        let start = src
            .find(fn_name)
            .unwrap_or_else(|| panic!("missing {fn_name}"));
        let body = &src[start..];
        let delete_at = body
            .find("DELETE FROM")
            .unwrap_or_else(|| panic!("{fn_name} must contain DELETE FROM"));
        let skip_at = body
            .find("skip_legacy_mutate_if_absent")
            .unwrap_or_else(|| panic!("{fn_name} must probe before DELETE ({op})"));
        assert!(
            skip_at < delete_at,
            "{fn_name} must call skip_legacy_mutate_if_absent before DELETE FROM"
        );
    }
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn legacy_chunk_ddl_retired_probe_contract() {
    let Some(cfg) = require_or_skip_postgres("spec091_vws_ddl") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let ns = format!("vwsd_{}", Uuid::new_v4().as_simple());
    std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns;
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        8,
    );
    let table = storage
        .vectors_table_name()
        .trim_start_matches("public.")
        .to_string();

    assert!(
        !table_exists(&pool, &table).await,
        "precondition: {table} absent"
    );

    let applied_131: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 131 AND success)",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);

    if applied_131 {
        // Fleet DDL retired globally — missing table is still "retired".
        assert!(
            storage.probe_legacy_vector_ddl_retired().await,
            "131+typed → legacy_vector_ddl_retired"
        );
        assert!(
            storage.probe_legacy_chunk_ddl_retired().await,
            "131 implies chunk DDL retired even when table is absent"
        );
        std::env::remove_var(VECTOR_BACKEND_ENV);
        return;
    }

    // Pre-131: missing table must NOT retire (never-created ≠ dropped).
    assert!(
        !storage.probe_legacy_chunk_ddl_retired().await,
        "missing table must not retire chunk DDL pre-131"
    );

    let applied_126: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 126 AND success)",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !applied_126 {
        eprintln!("migration 126 not applied — skipping exists+retired branch");
        std::env::remove_var(VECTOR_BACKEND_ENV);
        return;
    }

    sqlx::query(&format!(
        "CREATE TABLE public.{table} (id TEXT PRIMARY KEY, embedding vector(8), \
         metadata JSONB DEFAULT '{{}}')"
    ))
    .execute(&pool)
    .await
    .expect("create empty-ish legacy table");
    sqlx::query(&format!(
        "INSERT INTO public.{table} (id, embedding) VALUES ('doc-chunk-0', '[0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8]'::vector)"
    ))
    .execute(&pool)
    .await
    .expect("seed chunk-only row");

    assert!(
        storage.probe_legacy_chunk_ddl_retired().await,
        "exists + non_chunk=0 +126+typed → retired"
    );

    sqlx::query(&format!("DROP TABLE IF EXISTS public.{table}"))
        .execute(&pool)
        .await
        .ok();
    std::env::remove_var(VECTOR_BACKEND_ENV);
}

/// SPEC-111: typed write-stop must NOT skip lifecycle DELETE — wipe residue
/// would otherwise leave orphan fleet rows that fail provenance-stamp verify.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn typed_backend_clear_workspace_purges_residual_legacy_rows() {
    let Some(cfg) = require_or_skip_postgres("spec091_vws_purge") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;

    let ns = format!("vwsp_{}", Uuid::new_v4().as_simple());
    std::env::set_var(VECTOR_BACKEND_ENV, "typed_embeddings");

    let mut pg_cfg = cfg.clone();
    pg_cfg.namespace = ns;
    let storage = PgVectorStorage::with_pool(
        PostgresPool::from_existing(pool.clone(), pg_cfg.clone()),
        pg_cfg,
        8,
    );
    let table = storage
        .vectors_table_name()
        .trim_start_matches("public.")
        .to_string();
    let ws = Uuid::new_v4();

    // Simulate pre-131 residue: legacy table still present under typed authority.
    sqlx::query(&format!(
        "CREATE TABLE public.{table} (
            id TEXT PRIMARY KEY,
            embedding halfvec(8) NOT NULL,
            metadata JSONB DEFAULT '{{}}'::jsonb,
            workspace_id TEXT
        )"
    ))
    .execute(&pool)
    .await
    .expect("create residual legacy table");
    sqlx::query(&format!(
        "INSERT INTO public.{table} (id, embedding, metadata, workspace_id)
         VALUES ($1, $2::halfvec, $3, $4)"
    ))
    .bind("entity:ORPHAN_QUERY")
    .bind("[0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8]")
    .bind(json!({"workspace_id": ws.to_string()}))
    .bind(ws.to_string())
    .execute(&pool)
    .await
    .expect("seed orphan");

    let n = storage
        .clear_workspace(&ws)
        .await
        .expect("clear_workspace under typed must purge residue");
    assert!(n >= 1, "must delete residual legacy rows, got {n}");
    let left: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM public.{table}"))
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(left, 0, "orphan fleet rows must be gone after wipe clear");

    sqlx::query(&format!("DROP TABLE IF EXISTS public.{table}"))
        .execute(&pool)
        .await
        .ok();
    std::env::remove_var(VECTOR_BACKEND_ENV);
}
