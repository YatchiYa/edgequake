//! SPEC-091 migration-console fixtures (doc 15 §"previous-version e2e").
//!
//! Provisions a dedicated **pre-drop** database (`edgequake_spec091_fix`) by
//! replaying migrations `001..=124` and bookkeeping them in `_sqlx_migrations`
//! (without 125), so the advisor can be driven through every earlier cutover
//! phase against a real schema. The dev DB cannot be used for this: migration
//! 125 is already applied there, so `kv_store_dropped` is permanently true.
//!
//! The fixture DB is created once per test process (cached), then *reset*
//! between tests (typed tables truncated, temp KV tables dropped, ledger chunk
//! jobs cleared) so cases stay isolated while sharing one database. Access is
//! serialized via [`fixture_lock`]; env-flag mutation via [`env_lock`].
#![allow(dead_code)]

use sqlx::PgPool;
use std::sync::OnceLock;
use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

use edgequake_storage::PostgresConfig;

/// Database that holds the pre-drop schema (migrations 001..=124 applied).
pub const FIXTURE_DB: &str = "edgequake_spec091_fix";
/// The highest pre-drop migration (125 is the KV drop — intentionally absent).
const PRE_DROP_MAX: i64 = 124;
/// The chunk backfill engine step id (SSOT — `chunk_text_backfill.rs`).
const CHUNK_BACKFILL_STEP: &str = "w1-chunk-text-backfill";

/// Migrations live at `edgequake/migrations` (two levels up from this crate —
/// the repo nests `edgequake/edgequake`).
const MIGRATIONS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");

/// Every env var that owns a family mode — set to `relational` for the
/// ReadyToDrop phase and cleared afterwards (SSOT for the e2e cases).
pub const ALL_FAMILY_ENV_VARS: &[&str] = &[
    "EDGEQUAKE_CHUNK_TEXT_AUTHORITY",
    "EDGEQUAKE_KV_FAMILY_METADATA",
    "EDGEQUAKE_KV_FAMILY_WSDOC",
    "EDGEQUAKE_KV_FAMILY_DOC_HASH",
    "EDGEQUAKE_KV_FAMILY_COMPENSATION_QUARANTINE",
    "EDGEQUAKE_KV_FAMILY_CHECKPOINT",
    "EDGEQUAKE_KV_FAMILY_ARTIFACT",
    "EDGEQUAKE_KV_FAMILY_INJECTION",
    "EDGEQUAKE_KV_FAMILY_CACHE",
];

/// Serialize the whole migration-console critical section (env-flag mutation +
/// shared fixture DB) across tests. A single async-aware `tokio::sync::Mutex`
/// is used so the guard can be held across `.await` without tripping
/// `clippy::await_holding_lock`; these tests are inherently serial anyway
/// (process-global env + one shared fixture DB).
pub fn console_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Clear every family/env flag a case may have set (call under `console_lock`).
pub fn clear_family_env() {
    for var in ALL_FAMILY_ENV_VARS {
        std::env::remove_var(var);
    }
    std::env::remove_var("EDGEQUAKE_MIGRATION_MODE");
}

/// Resolve the pre-drop fixture DB URL, provisioning it on first use.
///
/// Returns `None` when the admin (maintenance) connection is unavailable, so
/// callers can soft-skip exactly like the dev-DB tests do.
pub async fn predrop_fixture_url(cfg: &PostgresConfig) -> Option<String> {
    static CELL: OnceCell<Option<String>> = OnceCell::const_new();
    CELL.get_or_init(|| async { provision(cfg).await })
        .await
        .clone()
}

/// Connect a pool to `database` using the dev-DB credentials from `cfg`.
async fn connect(cfg: &PostgresConfig, database: &str) -> Option<PgPool> {
    let url = format!(
        "postgres://{}:{}@{}:{}/{}",
        cfg.user, cfg.password, cfg.host, cfg.port, database
    );
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(15))
        .connect(&url)
        .await
        .ok()
}

async fn provision(cfg: &PostgresConfig) -> Option<String> {
    let admin = connect(cfg, "postgres").await?;

    // Reuse a current fixture when present (fast repeat runs); rebuild otherwise.
    if fixture_is_current(cfg, &admin).await {
        return Some(fixture_url(cfg));
    }

    // (Re)create from scratch. Terminate stragglers from a previous run first.
    sqlx::query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1 AND pid <> pg_backend_pid()")
        .bind(FIXTURE_DB)
        .execute(&admin)
        .await
        .ok();
    sqlx::query(&format!("DROP DATABASE IF EXISTS {FIXTURE_DB}"))
        .execute(&admin)
        .await
        .ok()?;
    sqlx::query(&format!("CREATE DATABASE {FIXTURE_DB}"))
        .execute(&admin)
        .await
        .ok()?;

    let fixture = connect(cfg, FIXTURE_DB).await?;
    let ok = apply_predrop_migrations(&fixture).await.is_some()
        && bookkeep_predrop(&fixture).await.is_some();
    fixture.close().await;
    ok.then(|| fixture_url(cfg))
}

fn fixture_url(cfg: &PostgresConfig) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        cfg.user, cfg.password, cfg.host, cfg.port, FIXTURE_DB
    )
}

/// A fixture is reusable when it already has migrations 001..=124 booked and
/// the drop migration 125 is absent.
async fn fixture_is_current(cfg: &PostgresConfig, admin: &PgPool) -> bool {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(FIXTURE_DB)
            .fetch_one(admin)
            .await
            .unwrap_or(false);
    if !exists {
        return false;
    }
    let Some(pool) = connect(cfg, FIXTURE_DB).await else {
        return false;
    };
    let ok: bool = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), -1) = $1 \
           AND NOT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = 125) \
         FROM _sqlx_migrations",
    )
    .bind(PRE_DROP_MAX)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    pool.close().await;
    ok
}

/// Replay migrations `001..=124` (skip the 125 drop) in order on one connection.
async fn apply_predrop_migrations(pool: &PgPool) -> Option<()> {
    let mut files: Vec<(i64, std::path::PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(MIGRATIONS_DIR).ok()? {
        let path = entry.ok()?.path();
        let name = path.file_name()?.to_str()?.to_string();
        if !name.ends_with(".sql") {
            continue;
        }
        let Ok(version) = name[..3].parse::<i64>() else {
            continue;
        };
        if (1..=PRE_DROP_MAX).contains(&version) {
            files.push((version, path));
        }
    }
    files.sort_by_key(|(v, _)| *v);
    if files.len() < 120 {
        // Sanity: the migrations dir must resolve, else we would silently test
        // against an empty schema.
        eprintln!(
            "SKIP fixture: only {} migrations found under {MIGRATIONS_DIR}",
            files.len()
        );
        return None;
    }

    let mut conn = pool.acquire().await.ok()?;
    for (version, path) in files {
        let sql = std::fs::read_to_string(&path).ok()?;
        sqlx::raw_sql(&sql)
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                eprintln!("fixture migration {version} failed: {e}");
                e
            })
            .ok()?;
    }
    Some(())
}

/// Bookkeep 001..=124 as applied (mirroring sqlx) so the advisor reads
/// "125 not yet applied" → the store is pre-drop.
async fn bookkeep_predrop(pool: &PgPool) -> Option<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _sqlx_migrations (\
            version BIGINT PRIMARY KEY,\
            description TEXT NOT NULL,\
            installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),\
            success BOOLEAN NOT NULL,\
            checksum BYTEA NOT NULL,\
            execution_time BIGINT NOT NULL)",
    )
    .execute(pool)
    .await
    .ok()?;
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) \
         SELECT generate_series(1, $1), 'fixture', TRUE, '\\x00'::bytea, 0 \
         ON CONFLICT (version) DO NOTHING",
    )
    .bind(PRE_DROP_MAX)
    .execute(pool)
    .await
    .ok()?;
    Some(())
}

/// Reset the fixture to a clean pre-drop baseline between tests: drop any test
/// KV tables, clear chunk backfill ledger rows, and truncate the typed tables
/// a case may have seeded.
pub async fn reset_predrop_fixture(pool: &PgPool) {
    drop_all_kv_tables(pool).await;
    clear_chunk_backfill_jobs(pool).await;
    // Typed SSOT tables a case may seed (fixture starts empty; CASCADE also
    // clears dependent rows such as FK children).
    sqlx::query(
        "TRUNCATE public.chunks, public.documents, public.document_artifacts, \
         public.ingestion_dedup CASCADE",
    )
    .execute(pool)
    .await
    .expect("truncate typed tables");
}

/// Create an isolated KV table `public.eq_<ns>_kv` (minimal generic-KV shape).
pub async fn create_kv_table(pool: &PgPool, ns: &str) -> String {
    let table = format!("eq_{ns}_kv");
    sqlx::query(&format!(
        "CREATE TABLE public.{table} (key TEXT PRIMARY KEY, value JSONB)"
    ))
    .execute(pool)
    .await
    .expect("create kv table");
    table
}

/// Seed `n` chunk-text residue rows for `doc_id` (key `{uuid}-chunk-{i}`,
/// value `{content}`) — durable per the migration-125 chunk predicate.
pub async fn seed_chunk_text_residue(pool: &PgPool, table: &str, doc_id: Uuid, n: i64) {
    for i in 0..n {
        sqlx::query(&format!(
            "INSERT INTO public.{table} (key, value) VALUES ($1, $2) ON CONFLICT (key) DO NOTHING"
        ))
        .bind(format!("{doc_id}-chunk-{i}"))
        .bind(serde_json::json!({"content": format!("chunk text {i}")}))
        .execute(pool)
        .await
        .expect("seed chunk_text residue");
    }
}

/// Seed a typed `public.chunks` row so the 125 guard/verify see the text as
/// already represented (EC-34: chunk present in typed ⇒ KV row is redundant).
pub async fn seed_typed_chunk(pool: &PgPool, doc_id: Uuid, chunk_index: i32, content: &str) {
    sqlx::query(
        "INSERT INTO public.chunks (id, document_id, chunk_index, content) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(doc_id)
    .bind(chunk_index)
    .bind(content)
    .execute(pool)
    .await
    .expect("seed typed chunk");
}

/// Insert one raw KV row (generic helper for the durable-category fixtures).
pub async fn seed_kv_row(pool: &PgPool, table: &str, key: &str, value: serde_json::Value) {
    sqlx::query(&format!(
        "INSERT INTO public.{table} (key, value) VALUES ($1, $2) ON CONFLICT (key) DO NOTHING"
    ))
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .expect("seed kv row");
}

/// Seed one durable residue row for **every** durable family against a fresh
/// `doc_id`, so all seven durable families report residue > 0 (a fresh,
/// pre-cutover deployment where nothing has been migrated to typed yet).
pub async fn seed_all_durable_residue(pool: &PgPool, table: &str, doc_id: Uuid) {
    let rows: [(String, serde_json::Value); 7] = [
        (
            format!("{doc_id}-chunk-0"),
            serde_json::json!({"content": "text"}),
        ),
        (
            format!("{doc_id}-metadata"),
            serde_json::json!({"title": "t"}),
        ),
        (format!("wsdoc:{doc_id}"), serde_json::json!({"ws": 1})),
        ("staging:hash:abc".to_string(), serde_json::json!({"h": 1})),
        ("doc:hash:def".to_string(), serde_json::json!({"h": 1})),
        (format!("{doc_id}-lineage"), serde_json::json!({"src": "x"})),
        (format!("injection::{doc_id}"), serde_json::json!({"i": 1})),
    ];
    for (key, value) in rows {
        seed_kv_row(pool, table, &key, value).await;
    }
}

/// Seed a typed `public.documents` shell row (id only) for shell/durable tests.
pub async fn seed_typed_document(pool: &PgPool, doc_id: Uuid) {
    sqlx::query(
        "INSERT INTO public.documents (id, title, content, status) \
         VALUES ($1, 'fixture doc', 'fixture content', 'completed') ON CONFLICT (id) DO NOTHING",
    )
    .bind(doc_id)
    .execute(pool)
    .await
    .expect("seed typed document");
}

/// Drop every `public.eq_*_kv` table (test artifacts).
pub async fn drop_all_kv_tables(pool: &PgPool) {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relname LIKE 'eq\\_%\\_kv' ESCAPE '\\'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for t in tables {
        sqlx::query(&format!("DROP TABLE IF EXISTS public.{t} CASCADE"))
            .execute(pool)
            .await
            .expect("drop kv table");
    }
}

/// Insert a chunk backfill job at a high schema generation so the advisor picks
/// it as the latest; returns its job_id.
pub async fn seed_chunk_backfill_job(
    pool: &PgPool,
    state: &str,
    processed: i64,
    total: i64,
) -> Uuid {
    let completed = matches!(state, "completed" | "cancelled" | "failed");
    sqlx::query_scalar(
        "INSERT INTO edgequake.edgequake_migration_job \
            (step_id, step_sha384, schema_generation, state, reversibility, batch_size, \
             estimated_total, processed_count, lease_owner, lease_expires_at, heartbeat_at, completed_at) \
         VALUES ($1, 'deadbeef', 999, $2, 'reversible', 100, $3, $4, \
                 'e2e-owner', now() + interval '60 seconds', now(), \
                 CASE WHEN $5 THEN now() ELSE NULL END) \
         RETURNING job_id",
    )
    .bind(CHUNK_BACKFILL_STEP)
    .bind(state)
    .bind(total)
    .bind(processed)
    .bind(completed)
    .fetch_one(pool)
    .await
    .expect("seed chunk backfill job")
}

/// Delete every chunk backfill job (and its batches) from the ledger.
pub async fn clear_chunk_backfill_jobs(pool: &PgPool) {
    sqlx::query(
        "DELETE FROM edgequake.edgequake_migration_batch WHERE job_id IN \
                 (SELECT job_id FROM edgequake.edgequake_migration_job WHERE step_id = $1)",
    )
    .bind(CHUNK_BACKFILL_STEP)
    .execute(pool)
    .await
    .expect("clear batches");
    sqlx::query("DELETE FROM edgequake.edgequake_migration_job WHERE step_id = $1")
        .bind(CHUNK_BACKFILL_STEP)
        .execute(pool)
        .await
        .expect("clear jobs");
}
