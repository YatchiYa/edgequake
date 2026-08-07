//! SPEC-110: migration 118/121 ON CONFLICT conflict-key cardinality.
//!
//! Proves v0.24.1 broken wsdoc SQL fails with Postgres 21000 on multi-workspace
//! membership keys, and that patched 118/121 (`DISTINCT ON` conflict key) succeed
//! and are idempotent.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec110_wsdoc_on_conflict -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use sqlx::PgPool;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use uuid::Uuid;

const MIGRATION_118: &str = include_str!("../../../migrations/118_spec091_wsdoc_backfill.sql");
const MIGRATION_121: &str = include_str!("../../../migrations/121_spec091_injection_backfill.sql");

/// Pre-SPEC-110 body of migration 118 (`SELECT DISTINCT` full tuple) — must fail
/// with 21000 when the same document_id appears under two workspaces.
const OLD_118_SQL: &str = r#"
DO $$
DECLARE
    kv_table RECORD;
    uuid_re constant text := '^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$';
BEGIN
FOR kv_table IN
    SELECT tablename FROM pg_tables
    WHERE schemaname = 'public' AND tablename LIKE 'eq\_%\_kv'
LOOP
    EXECUTE format($f$
        INSERT INTO public.documents (id, workspace_id, content, status)
        SELECT DISTINCT split_part(kv.key, ':', 3)::uuid,
                        split_part(kv.key, ':', 2)::uuid, '', 'indexed'
        FROM %I kv
        WHERE kv.key LIKE 'wsdoc:%%'
          AND split_part(kv.key, ':', 2) ~ $1
          AND split_part(kv.key, ':', 3) ~ $1
          AND EXISTS (
              SELECT 1 FROM public.workspaces w
              WHERE w.workspace_id = split_part(kv.key, ':', 2)::uuid)
        ON CONFLICT (id) DO UPDATE SET
            workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)
    $f$, kv_table.tablename) USING uuid_re;
END LOOP;
END $$;
"#;

fn spec110_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct Fixture {
    tenant: Uuid,
    ws_lo: Uuid,
    ws_hi: Uuid,
    doc_id: Uuid,
    inj_id: Uuid,
    kv_table: String,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let mut a = Uuid::new_v4();
        let mut b = Uuid::new_v4();
        if a.as_bytes() > b.as_bytes() {
            std::mem::swap(&mut a, &mut b);
        }
        // Lexicographic string order matches uuid text compare for DISTINCT ON ORDER BY.
        let (ws_lo, ws_hi) = if a.to_string() <= b.to_string() {
            (a, b)
        } else {
            (b, a)
        };
        Self {
            tenant: Uuid::new_v4(),
            ws_lo,
            ws_hi,
            doc_id: Uuid::new_v4(),
            inj_id: Uuid::new_v4(),
            kv_table: format!("eq_spec110_{tag}_kv"),
        }
    }
}

async fn seed_two_workspaces(pool: &PgPool, fx: &Fixture) {
    let slug_t = format!("t-{}", fx.tenant.as_simple());
    sqlx::query(
        "INSERT INTO tenants (tenant_id, name, slug, is_active) VALUES ($1, $2, $3, TRUE) \
         ON CONFLICT (tenant_id) DO NOTHING",
    )
    .bind(fx.tenant)
    .bind(format!("spec110-{}", fx.tenant.as_simple()))
    .bind(&slug_t)
    .execute(pool)
    .await
    .expect("seed tenant");

    for (ws, name) in [(fx.ws_lo, "lo"), (fx.ws_hi, "hi")] {
        let slug = format!("{name}-{}", ws.as_simple());
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (workspace_id) DO NOTHING",
        )
        .bind(ws)
        .bind(fx.tenant)
        .bind(format!("spec110-{name}"))
        .bind(&slug)
        .execute(pool)
        .await
        .expect("seed workspace");
    }
}

async fn ensure_kv_table(pool: &PgPool, fx: &Fixture) {
    sqlx::query(&format!(
        "CREATE TABLE IF NOT EXISTS public.{} (key TEXT PRIMARY KEY, value JSONB NOT NULL DEFAULT '{{}}'::jsonb)",
        fx.kv_table
    ))
    .execute(pool)
    .await
    .expect("create kv table");
    sqlx::query(&format!("DELETE FROM public.{}", fx.kv_table))
        .execute(pool)
        .await
        .expect("clear kv");
}

async fn seed_multi_ws_wsdoc(pool: &PgPool, fx: &Fixture) {
    for ws in [fx.ws_lo, fx.ws_hi] {
        let key = format!("wsdoc:{}:{}", ws, fx.doc_id);
        sqlx::query(&format!(
            "INSERT INTO public.{} (key, value) VALUES ($1, '{{}}'::jsonb) \
             ON CONFLICT (key) DO NOTHING",
            fx.kv_table
        ))
        .bind(&key)
        .execute(pool)
        .await
        .expect("seed wsdoc");
    }
}

async fn seed_multi_ws_injection(pool: &PgPool, fx: &Fixture) {
    for ws in [fx.ws_lo, fx.ws_hi] {
        let key = format!("injection::{}:x:{}-metadata", ws, fx.inj_id);
        let value = serde_json::json!({
            "name": "inj",
            "content": "body",
            "status": "completed"
        });
        sqlx::query(&format!(
            "INSERT INTO public.{} (key, value) VALUES ($1, $2::jsonb) \
             ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
            fx.kv_table
        ))
        .bind(&key)
        .bind(value)
        .execute(pool)
        .await
        .expect("seed injection");
    }
}

async fn cleanup_all(pool: &PgPool, fx: &Fixture) {
    let _ = sqlx::query("DELETE FROM public.documents WHERE id = ANY($1)")
        .bind(&[fx.doc_id, fx.inj_id][..])
        .execute(pool)
        .await;
    let _ = sqlx::query(&format!("DROP TABLE IF EXISTS public.{}", fx.kv_table))
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM public.workspaces WHERE workspace_id = ANY($1)")
        .bind(&[fx.ws_lo, fx.ws_hi][..])
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM public.tenants WHERE tenant_id = $1")
        .bind(fx.tenant)
        .execute(pool)
        .await;
}

/// E2E-110-01: old SQL fails with cardinality violation on multi-ws wsdoc.
#[tokio::test]
async fn e2e110_01_old_118_fails_on_multi_ws_wsdoc() {
    let Some(cfg) = require_or_skip_postgres("spec110_old") else {
        return;
    };
    let _g = spec110_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let fx = Fixture::new("old");
    cleanup_all(&pool, &fx).await;
    seed_two_workspaces(&pool, &fx).await;
    ensure_kv_table(&pool, &fx).await;
    seed_multi_ws_wsdoc(&pool, &fx).await;

    let res = sqlx::raw_sql(OLD_118_SQL).execute(&pool).await;
    assert!(res.is_err(), "old 118 must fail on multi-ws wsdoc keys");
    let msg = format!("{res:?}");
    assert!(
        msg.contains("cannot affect row a second time")
            || msg.contains("CardinalityViolation")
            || msg.contains("21000"),
        "expected Postgres 21000 / affect row a second time, got: {msg}"
    );

    cleanup_all(&pool, &fx).await;
}

/// E2E-110-02 + E2E-110-03: patched 118 succeeds, picks min workspace, re-run stable.
#[tokio::test]
async fn e2e110_02_03_patched_118_collapses_and_idempotent() {
    let Some(cfg) = require_or_skip_postgres("spec110_patched") else {
        return;
    };
    let _g = spec110_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let fx = Fixture::new("patched");
    cleanup_all(&pool, &fx).await;
    seed_two_workspaces(&pool, &fx).await;
    ensure_kv_table(&pool, &fx).await;
    seed_multi_ws_wsdoc(&pool, &fx).await;

    sqlx::raw_sql(MIGRATION_118)
        .execute(&pool)
        .await
        .expect("patched 118 must succeed");

    let ws: Option<Uuid> =
        sqlx::query_scalar("SELECT workspace_id FROM public.documents WHERE id = $1")
            .bind(fx.doc_id)
            .fetch_optional(&pool)
            .await
            .expect("select doc");
    assert_eq!(
        ws,
        Some(fx.ws_lo),
        "LAW-M5: lexicographic min workspace wins"
    );

    sqlx::query("UPDATE public.documents SET workspace_id = $1 WHERE id = $2")
        .bind(fx.ws_hi)
        .bind(fx.doc_id)
        .execute(&pool)
        .await
        .expect("force workspace hi");

    sqlx::raw_sql(MIGRATION_118)
        .execute(&pool)
        .await
        .expect("idempotent 118 re-run");

    let ws_after: Uuid =
        sqlx::query_scalar("SELECT workspace_id FROM public.documents WHERE id = $1")
            .bind(fx.doc_id)
            .fetch_one(&pool)
            .await
            .expect("select after re-run");
    assert_eq!(
        ws_after, fx.ws_hi,
        "COALESCE must not overwrite non-NULL workspace"
    );

    cleanup_all(&pool, &fx).await;
}

/// E2E-110-04: patched 121 handles same injection id under two workspaces.
#[tokio::test]
async fn e2e110_04_patched_121_injection_multi_ws() {
    let Some(cfg) = require_or_skip_postgres("spec110_inj") else {
        return;
    };
    let _g = spec110_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    let fx = Fixture::new("inj");
    cleanup_all(&pool, &fx).await;
    seed_two_workspaces(&pool, &fx).await;
    ensure_kv_table(&pool, &fx).await;
    seed_multi_ws_injection(&pool, &fx).await;

    sqlx::raw_sql(MIGRATION_121)
        .execute(&pool)
        .await
        .expect("patched 121 must succeed");

    let row: Option<(Uuid, String)> =
        sqlx::query_as("SELECT workspace_id, status FROM public.documents WHERE id = $1")
            .bind(fx.inj_id)
            .fetch_optional(&pool)
            .await
            .expect("select inj");
    let (ws, status) = row.expect("injection document row");
    assert_eq!(ws, fx.ws_lo);
    assert_eq!(status, "indexed");

    sqlx::raw_sql(MIGRATION_121)
        .execute(&pool)
        .await
        .expect("idempotent 121 re-run");

    cleanup_all(&pool, &fx).await;
}

/// E2E-110-05: source guard — migrations use DISTINCT ON on conflict key.
#[test]
fn e2e110_05_source_guard_distinct_on() {
    assert!(
        MIGRATION_118.contains("DISTINCT ON (doc_id)"),
        "118 must DISTINCT ON conflict key doc_id"
    );
    assert!(
        !MIGRATION_118.contains("SELECT DISTINCT split_part"),
        "118 must not use bare SELECT DISTINCT on split_part tuple"
    );
    assert!(
        MIGRATION_121.contains("DISTINCT ON (inj_id)"),
        "121 must DISTINCT ON conflict key inj_id"
    );
}
