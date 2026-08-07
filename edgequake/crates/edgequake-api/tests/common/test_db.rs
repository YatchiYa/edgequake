//! Dedicated scratch test-database routing + auto-provisioning (test-only).
//!
//! # Root cause this fixes
//!
//! E2E tests historically resolved `DATABASE_URL` (or `/tmp/edgequake-db-url`),
//! both of which point at the **shared dev database**. KV/vector tables get a
//! per-test random namespace, but post-SPEC-091 `public.documents` is a single
//! global typed table — so every document-writing e2e test leaked rows into the
//! dev document store (e.g. 1600 `Wipe Scale` rows, repeated `Tech Article PG`),
//! which surfaced in the dev UI document list as "many documents with the same
//! title".
//!
//! Routing every test to an isolated `{db}_test` database (same already-migrated
//! schema) makes that pollution impossible while keeping test behavior identical.
//!
//! # Usage
//!
//! ```ignore
//! #[path = "common/test_db.rs"]
//! mod test_db;
//! let url = test_db::isolated_test_url(&base_url);
//! ```
//!
//! Self-contained on purpose (only `sqlx` + `std` + `tokio`) so test binaries can
//! include it via `#[path]` without pulling the heavier `common/mod.rs` helpers.

#![allow(dead_code)]

use std::env;

/// Embedded migrations (SSOT: `edgequake/migrations`). Mirrors the storage-side
/// harness (`edgequake-storage/tests/support/postgres_test_config.rs`).
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// Rewrite `base_url` to the dedicated scratch test database and ensure it is
/// provisioned (created + migrated) once per process. Returns the test URL.
///
/// Override the whole URL with `EDGEQUAKE_TEST_DATABASE_URL` (e.g. CI pointing at
/// another cluster); the database is otherwise derived as `{db}_test`.
pub fn isolated_test_url(base_url: &str) -> String {
    let test = to_test_database_url(base_url);
    ensure_test_db_ready(&test);
    test
}

/// Rewrite the database in a connection URL to the dedicated scratch test
/// database (`{db}_test`), preserving credentials, host, port, and query string.
fn to_test_database_url(url: &str) -> String {
    if let Ok(over) = env::var("EDGEQUAKE_TEST_DATABASE_URL") {
        if !over.trim().is_empty() {
            return over.trim().to_string();
        }
    }
    let (head, query) = match url.split_once('?') {
        Some((h, q)) => (h, Some(q)),
        None => (url, None),
    };
    let rewritten = match head.rfind('/') {
        Some(idx) => {
            let (prefix, db) = head.split_at(idx + 1);
            if db.is_empty() || db.ends_with("_test") {
                // Idempotent: a base already suffixed `_test` is left as-is so
                // overrides/defaults that already point at a scratch DB don't
                // become `{db}_test_test`.
                head.to_string()
            } else {
                format!("{prefix}{db}_test")
            }
        }
        None => head.to_string(),
    };
    match query {
        Some(q) => format!("{rewritten}?{q}"),
        None => rewritten,
    }
}

/// Provision the scratch test database once per test process: create it when
/// missing, then apply all embedded migrations so tests boot against an
/// already-migrated schema identical to (but isolated from) the dev database.
///
/// Best-effort: when the server is unreachable the caller still returns a URL
/// and the test soft-skips / fails on connect exactly as against the dev DB
/// today. Runs on a dedicated thread + runtime so it is safe to call from a
/// sync context reached by async tests (no nested-runtime panic).
fn ensure_test_db_ready(url: &str) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let url = url.to_string();
        let _ = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test-db provision runtime");
            rt.block_on(provision_test_db(&url));
        })
        .join();
    });
}

async fn provision_test_db(url: &str) {
    let (head, _) = match url.split_once('?') {
        Some((h, q)) => (h, Some(q)),
        None => (url, None),
    };
    let Some(idx) = head.rfind('/') else {
        return;
    };
    let (prefix, db) = head.split_at(idx + 1);
    if db.is_empty() {
        return;
    }
    let admin_url = format!("{prefix}postgres");
    let Ok(admin) = sqlx::PgPool::connect(&admin_url).await else {
        return;
    };
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(db)
            .fetch_one(&admin)
            .await
            .unwrap_or(false);
    if !exists {
        // CREATE DATABASE cannot be parameterized; the name is derived internally.
        let _ = sqlx::query(&format!("CREATE DATABASE {db}"))
            .execute(&admin)
            .await;
    }
    admin.close().await;

    if let Ok(pool) = sqlx::PgPool::connect(url).await {
        // Scratch DB only: reconcile known expandable-migration checksum
        // drifts (SPEC-110/111) so sqlx migrate can apply pending files
        // without requiring EDGEQUAKE_DEV_MODE (prod still fails closed).
        repair_test_db_migration_checksums(&pool).await;
        // Idempotent: applies only pending migrations, so concurrent test
        // processes and repeat runs converge without dropping anything.
        if let Err(e) = MIGRATOR.run(&pool).await {
            eprintln!("test-db provisioning migrate failed: {e}");
        }
        pool.close().await;
    }
}

/// Update `_sqlx_migrations.checksum` for known broken→fixed pairs so the
/// isolated `{db}_test` scratch database can continue after SPEC-110/111
/// in-place migration edits. No-op when the table or version is absent.
async fn repair_test_db_migration_checksums(pool: &sqlx::PgPool) {
    const REPAIRS: &[(i64, &str, &str)] = &[
        (
            125,
            "67b73fd0f683dd5cae06213ae59c75c2f8fea214074e8b250997aa77efc90a1fa01c14764f9fdb968b0e73685136b2f6",
            "9ae99858a9c88ec9b0a195447d6f7e2601fb4423f0d846314b6aa06d337ad9e74e9a8998ae7359fba65df694d5b1eeec",
        ),
        (
            131,
            "461fa2a7c560513df711f954edd4f24444c91cd0385a70189e41cecdebaf2f53cca49c932122b0d002407a6c7fc0dbe8",
            "1b42205577666dc31fa346c42eb8e787c78208b6438da2822245ec61d65f3d538df8f985b132b7e3a3930b7272c87a14",
        ),
        (
            131,
            "d6bc6c00b753f8599248dda86ce5d314e147491bcbb9932273c43afcbfc84a5d51c6a797387dfffeeca00588dc02c896",
            "1b42205577666dc31fa346c42eb8e787c78208b6438da2822245ec61d65f3d538df8f985b132b7e3a3930b7272c87a14",
        ),
    ];
    let Ok(exists): Result<bool, _> =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations') IS NOT NULL")
            .fetch_one(pool)
            .await
    else {
        return;
    };
    if !exists {
        return;
    }
    for &(version, broken, fixed) in REPAIRS {
        let _ = sqlx::query(
            "UPDATE _sqlx_migrations SET checksum = decode($1, 'hex') \
             WHERE version = $2 AND success = true \
               AND encode(checksum, 'hex') = $3",
        )
        .bind(fixed)
        .bind(version)
        .bind(broken)
        .execute(pool)
        .await;
    }
}
