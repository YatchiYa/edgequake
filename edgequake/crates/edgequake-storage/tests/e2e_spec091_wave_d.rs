//! SPEC-091 Wave D: live E2E for the KV write-stop end state.
//!
//! Proves the adapter end state against a migrated database:
//!   1. Fresh-install: zero `eq_*_kv` relations after migrations (no runtime DDL).
//!   2. Write-stop: shell/cache upserts never touch KV; they land in
//!      `documents` / `llm_cache` (typed authority, SSOT).
//!   3. Typed CAS: `transition_if_status` on metadata shells runs one atomic
//!      UPDATE against `documents.metadata`.
//!   4. 42P01 tolerance: a namespace whose KV table never existed reads as
//!      empty and writes route typed-only (post-drop behavior, simulated
//!      without dropping anything).
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec091_wave_d -- --nocapture

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::adapters::postgres::{apply_serving_fence, PostgresKVStorage};
use edgequake_storage::traits::{KVStorage, VectorSearchResult};
use edgequake_storage::SERVING_FENCE_ENV;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::json;
use sqlx::types::Uuid;
use sqlx::PgPool;

/// Serialize the env-mutating tests in this binary.
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn fresh_ids() -> (Uuid, Uuid) {
    (Uuid::new_v4(), Uuid::new_v4())
}

async fn cleanup_doc(pool: &PgPool, doc: Uuid) {
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(doc)
        .execute(pool)
        .await
        .ok();
}

/// 1. Fresh-install proof: migrations leave ZERO generic KV relations and no
///    KV stats trigger functions. The guarded drop (125) is a no-op when the
///    runtime DDL never created them.
#[tokio::test]
async fn e2e_spec091_fresh_install_has_no_kv_relations() {
    let Some(cfg) = require_or_skip_postgres("spec091waved_schema") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    let (kv_tables,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_class c \
         JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'public' AND c.relkind = 'r' \
           AND (c.relname LIKE 'eq\\_%\\_kv' ESCAPE '\\' \
                OR c.relname LIKE 'eq\\_%\\_kv\\_stats' ESCAPE '\\')",
    )
    .fetch_one(&pool)
    .await
    .expect("kv relation count");
    assert_eq!(
        kv_tables, 0,
        "fresh install must have zero generic KV relations (SPEC-091 exit bar)"
    );

    let (kv_fns,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_proc p \
         JOIN pg_namespace n ON n.oid = p.pronamespace \
         WHERE n.nspname = 'public' \
           AND p.proname LIKE 'eq\\_%\\_kv\\_stats\\_%' ESCAPE '\\'",
    )
    .fetch_one(&pool)
    .await
    .expect("kv function count");
    assert_eq!(kv_fns, 0, "no KV stats trigger functions after the drop");

    // Typed authority tables all exist.
    for table in [
        "documents",
        "chunks",
        "ingestion_dedup",
        "compensation_quarantine",
        "pipeline_checkpoints",
        "document_artifacts",
        "llm_cache",
    ] {
        let (exists,): (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name = $1)",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("table exists check");
        assert!(exists, "typed table public.{table} must exist");
    }
}

/// 2. Write-stop + SSOT: with default (relational) family flags, a metadata
///    shell upsert mutates `documents.metadata` and NOTHING writes KV; reads
///    come back typed-first; the cache family lands in `llm_cache`.
#[tokio::test]
// WHY: the env_lock guard serializes env-mutating tests; holding it across
// awaits is the point (prevents concurrent env var flips between tests).
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_shell_and_cache_write_stop_round_trip() {
    let _guard = env_lock();
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_METADATA");
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_CACHE");
    std::env::remove_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY");

    let Some(cfg) = require_or_skip_postgres("spec091waved_rw") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (doc, _ws) = fresh_ids();
    cleanup_doc(&pool, doc).await;

    // Parent row (admission) then shell metadata write via the facade.
    edgequake_storage::ensure_admission_document_row(&pool, doc, None, None, "wave-d.md")
        .await
        .expect("admission row");

    let kv = PostgresKVStorage::with_pool(
        edgequake_storage::adapters::postgres::PostgresPool::from_existing(
            pool.clone(),
            cfg.clone(),
        ),
        cfg.clone(),
    );

    let meta_key = format!("{doc}-metadata");
    kv.upsert(&[(
        meta_key.clone(),
        json!({"status": "processed", "title": "wave-d"}),
    )])
    .await
    .expect("shell upsert routes typed");

    // SSOT: documents.metadata carries the write.
    let (status,): (Option<String>,) =
        sqlx::query_as("SELECT metadata->>'status' FROM public.documents WHERE id = $1")
            .bind(doc)
            .fetch_one(&pool)
            .await
            .expect("documents metadata read");
    assert_eq!(status.as_deref(), Some("processed"));

    // Typed-first read through the facade returns the same fact.
    let got = kv.get_by_id(&meta_key).await.expect("facade read");
    assert_eq!(
        got.and_then(|v| v.get("title").and_then(|t| t.as_str()).map(String::from))
            .as_deref(),
        Some("wave-d")
    );

    // Cache family → public.llm_cache (namespace preserved).
    let cache_key = "deadbeef0123456789abcdef-cache".to_string();
    kv.upsert(&[(cache_key.clone(), json!({"result": {"choices": []}}))])
        .await
        .expect("cache upsert routes typed");
    let (cached,): (serde_json::Value,) =
        sqlx::query_as("SELECT value FROM public.llm_cache WHERE cache_key = $1")
            .bind(&cache_key)
            .fetch_one(&pool)
            .await
            .expect("llm_cache read");
    assert!(cached.get("result").is_some());

    let got = kv.get_by_id(&cache_key).await.expect("cache facade read");
    assert!(got.is_some(), "typed-first cache read hits llm_cache");

    kv.delete(std::slice::from_ref(&cache_key))
        .await
        .expect("cache delete");
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM public.llm_cache WHERE cache_key = $1")
        .bind(&cache_key)
        .fetch_one(&pool)
        .await
        .expect("llm_cache count");
    assert_eq!(n, 0, "typed delete removed the cache row");

    cleanup_doc(&pool, doc).await;
}

/// 3. Typed CAS: metadata status transitions run one atomic UPDATE against
///    `documents.metadata` (same semantics as the retired KV CAS).
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_transition_if_status_typed_cas() {
    let _guard = env_lock();
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_METADATA");

    let Some(cfg) = require_or_skip_postgres("spec091waved_cas") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let (doc, _ws) = fresh_ids();
    cleanup_doc(&pool, doc).await;
    edgequake_storage::ensure_admission_document_row(&pool, doc, None, None, "wave-d.md")
        .await
        .expect("admission row");

    let kv = PostgresKVStorage::with_pool(
        edgequake_storage::adapters::postgres::PostgresPool::from_existing(
            pool.clone(),
            cfg.clone(),
        ),
        cfg.clone(),
    );

    let key = format!("{doc}-metadata");
    kv.upsert(&[(key.clone(), json!({"status": "processed"}))])
        .await
        .expect("seed metadata");

    // Expected-state match → transitions.
    assert!(
        kv.transition_if_status(&key, "processed", "deleting")
            .await
            .expect("cas processed→deleting"),
        "CAS must succeed when current status matches"
    );
    // Expected-state mismatch → no-op (false), row unchanged.
    assert!(
        !kv.transition_if_status(&key, "processed", "deleting")
            .await
            .expect("cas mismatch"),
        "CAS must fail when current status moved on"
    );
    let (status,): (Option<String>,) =
        sqlx::query_as("SELECT metadata->>'status' FROM public.documents WHERE id = $1")
            .bind(doc)
            .fetch_one(&pool)
            .await
            .expect("status read");
    assert_eq!(status.as_deref(), Some("deleting"));

    cleanup_doc(&pool, doc).await;
}

/// 4. Post-drop tolerance: a facade whose KV table never existed (random
///    prefix == post-drop state) reads empty, never errors 42P01, and typed
///    routing still persists shell/cache writes.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_missing_kv_table_is_empty_not_error() {
    let _guard = env_lock();
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_METADATA");
    std::env::remove_var("EDGEQUAKE_KV_FAMILY_CACHE");
    std::env::remove_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY");

    let Some(mut cfg) = require_or_skip_postgres("spec091waved_drop") else {
        return;
    };
    // Random prefix: `eq_<rand>_kv` has never been created and never will be.
    cfg.namespace = format!("waved{}", &Uuid::new_v4().simple().to_string()[..12]);
    let pool = contract_pg_pool(&cfg).await;
    let kv = PostgresKVStorage::with_pool(
        edgequake_storage::adapters::postgres::PostgresPool::from_existing(
            pool.clone(),
            cfg.clone(),
        ),
        cfg.clone(),
    );

    // initialize() must not create the table (runtime DDL removed).
    kv.initialize().await.expect("initialize without DDL");
    let (n,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_class c JOIN pg_namespace ns ON ns.oid = c.relnamespace \
         WHERE ns.nspname = 'public' AND c.relname = $1",
    )
    .bind(format!("eq_{}_kv", cfg.namespace))
    .fetch_one(&pool)
    .await
    .expect("table lookup");
    assert_eq!(
        n, 0,
        "initialize() must never create the KV relation (Wave D)"
    );

    // Every read surface degrades to empty/None/Ok — no 42P01 escapes.
    assert!(kv
        .get_by_id("missing-metadata")
        .await
        .expect("get_by_id")
        .is_none());
    assert!(kv.is_empty().await.expect("is_empty"));
    assert_eq!(kv.count().await.expect("count"), 0);
    assert_eq!(kv.keys().await.expect("keys").len(), 0);
    kv.ping().await.expect("ping tolerates missing table");
    kv.delete(&["missing-metadata".to_string()])
        .await
        .expect("delete tolerates missing table");
    assert!(
        !kv.transition_if_status("missing-metadata", "a", "b")
            .await
            .expect("cas tolerates missing table"),
        "missing table has no rows to transition"
    );
    kv.clear().await.expect("clear tolerates missing table");

    // Regression (realized 2026-07-29): `get_by_ids` + `filter_keys` bypassed the
    // tolerant helpers and queried the dropped relation directly, so a retired-KV
    // caller (`ensure_document_exists` → KV `get_by_ids`) 42P01-failed into a 500 —
    // the images-not-served defect. Pin that both degrade like every sibling op.
    assert!(kv
        .get_by_ids(&["missing-metadata".to_string()])
        .await
        .expect("get_by_ids tolerates missing table")
        .is_empty());
    let missing = kv
        .filter_keys(std::collections::HashSet::from(["k1".to_string()]))
        .await
        .expect("filter_keys tolerates missing table");
    assert_eq!(
        missing.len(),
        1,
        "dropped table means every candidate key is missing, not an error"
    );

    // Typed routing still works through the same facade (shell write).
    let (doc, _ws) = fresh_ids();
    cleanup_doc(&pool, doc).await;
    edgequake_storage::ensure_admission_document_row(&pool, doc, None, None, "wave-d.md")
        .await
        .expect("admission row");
    kv.upsert(&[(format!("{doc}-metadata"), json!({"status": "pending"}))])
        .await
        .expect("typed write through post-drop facade");
    let got = kv
        .get_by_id(&format!("{doc}-metadata"))
        .await
        .expect("typed read");
    assert!(got.is_some(), "shell read works with no KV table at all");
    cleanup_doc(&pool, doc).await;
}

/// 5. EC-34 guard logic: the Wave-D drop guard must verify each durable family
///    against its typed SSOT (not trust key prefixes). Un-migrated chunk text /
///    shells / lineage count as durable; already-backfilled rows do NOT.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_drop_guard_verifies_typed_ssot() {
    let _guard = env_lock();
    let Some(cfg) = require_or_skip_postgres("spec091waved_guard") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    let uuid_re = "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})";
    let tbl = format!("guardtmp{}", &Uuid::new_v4().simple().to_string()[..12]);
    sqlx::query(&format!(
        "CREATE TABLE public.{tbl} (key text PRIMARY KEY, value jsonb)"
    ))
    .execute(&pool)
    .await
    .expect("create temp kv table");

    // A migrated document: present in `documents` AND with chunk 0 in `chunks`.
    let (migrated, _ws) = fresh_ids();
    cleanup_doc(&pool, migrated).await;
    edgequake_storage::ensure_admission_document_row(&pool, migrated, None, None, "migrated.md")
        .await
        .expect("migrated admission row");
    sqlx::query(
        "INSERT INTO public.chunks (document_id, content, chunk_index) \
         VALUES ($1, 'backfilled chunk text', 0) \
         ON CONFLICT (document_id, chunk_index) DO NOTHING",
    )
    .bind(migrated)
    .execute(&pool)
    .await
    .expect("seed migrated chunk");
    // An orphan document id: NOT in `documents`, no chunks.
    let orphan = Uuid::new_v4();

    // Populate the temp KV table with a mix of migrated / un-migrated rows.
    let rows: Vec<(String, serde_json::Value)> = vec![
        // migrated chunk → NOT durable (in chunks)
        (
            format!("{migrated}-chunk-0"),
            json!({"content":"backfilled chunk text"}),
        ),
        // orphan chunk → durable (not in chunks)
        (
            format!("{orphan}-chunk-3"),
            json!({"content":"orphan text"}),
        ),
        // migrated metadata shell → NOT durable (doc exists)
        (
            format!("{migrated}-metadata"),
            json!({"status":"processed"}),
        ),
        // orphan metadata shell → durable (doc missing)
        (format!("{orphan}-metadata"), json!({"status":"processed"})),
        // orphan lineage → durable (no artifact)
        (format!("{orphan}-lineage"), json!({"pipeline":"x"})),
        // conservative dedup prefix → durable regardless
        ("doc:hash:deadbeef".to_string(), json!({"ts":1})),
    ];
    for (k, v) in &rows {
        sqlx::query(&format!(
            "INSERT INTO public.{tbl} (key, value) VALUES ($1, $2)"
        ))
        .bind(k)
        .bind(v)
        .execute(&pool)
        .await
        .expect("insert kv row");
    }

    // The guard's exact durable-count predicate (typed verification).
    let guard_sql = format!(
        "SELECT count(*) FROM public.{tbl} k WHERE \
           (k.key ~ '^{uuid_re}-chunk-[0-9]+$' \
              AND COALESCE(k.value->>'content','') <> '' \
              AND NOT EXISTS (SELECT 1 FROM public.chunks c \
                     WHERE c.document_id = left(k.key,36)::uuid \
                       AND c.chunk_index = substring(k.key from 44)::int)) \
           OR ((k.key LIKE '%-metadata' OR k.key LIKE '%-content') \
              AND NOT EXISTS (SELECT 1 FROM public.documents d \
                     WHERE d.id = NULLIF(substring(k.key from '{uuid_re}'), '')::uuid)) \
           OR (k.key LIKE '%-lineage' \
              AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a \
                     WHERE a.kind='lineage' AND a.document_id = NULLIF(substring(k.key from '{uuid_re}'), '')::uuid)) \
           OR (k.key LIKE '%-multimodal-manifest' \
              AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a \
                     WHERE a.kind='multimodal-manifest' AND a.document_id = NULLIF(substring(k.key from '{uuid_re}'), '')::uuid)) \
           OR (k.key LIKE '%-multimodal-chunks' \
              AND NOT EXISTS (SELECT 1 FROM public.document_artifacts a \
                     WHERE a.kind='multimodal-chunks' AND a.document_id = NULLIF(substring(k.key from '{uuid_re}'), '')::uuid)) \
           OR k.key LIKE 'doc:hash:%' OR k.key LIKE 'staging:hash:%' \
           OR k.key LIKE 'wsdoc:%' OR k.key LIKE 'injection::%'"
    );
    let (durable,): (i64,) = sqlx::query_as(&guard_sql)
        .fetch_one(&pool)
        .await
        .expect("guard count");
    // Durable: orphan chunk + orphan metadata + orphan lineage + dedup = 4.
    // NOT durable: migrated chunk (in chunks) + migrated metadata (doc exists).
    assert_eq!(
        durable, 4,
        "guard must count only un-migrated durable rows (chunk/shell/lineage/dedup)"
    );

    sqlx::query(&format!("DROP TABLE public.{tbl}"))
        .execute(&pool)
        .await
        .ok();
    cleanup_doc(&pool, migrated).await;
}

/// 6. EC-35 engine post-drop: with the generic KV relation gone, the chunk
///    backfill job must report estimate 0 (EC-15 zero-estimate no-op), not a
///    boot-time 42P01 error — so `make dev` stays clean after Wave D.
#[tokio::test]
async fn e2e_spec091_chunk_backfill_estimate_zero_when_kv_dropped() {
    use edgequake_storage::migration_engine::{
        chunk_text_backfill::ChunkTextBackfillJob, BackfillJob,
    };

    let Some(cfg) = require_or_skip_postgres("spec091waved_engine") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    // A KV table name that has never existed / has been dropped (post-Wave-D).
    let missing = format!(
        "public.eq_missing{}_kv",
        &Uuid::new_v4().simple().to_string()[..10]
    );
    let job = ChunkTextBackfillJob::new(missing);
    let estimate = job
        .estimate_total(&pool)
        .await
        .expect("estimate must tolerate a dropped KV source");
    assert_eq!(
        estimate, 0,
        "dropped KV source must estimate 0 rows (nothing to backfill), not error"
    );
}

/// 7. EC-30 write-stop: a stale `EDGEQUAKE_CHUNK_TEXT_AUTHORITY=dual` flag
///    pointing at a dropped KV table must degrade to a typed-only no-op — the
///    raw KV upsert returns Ok (warn), never a 42P01 hard error.
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_dual_chunk_write_tolerates_dropped_kv() {
    let _guard = env_lock();
    std::env::set_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY", "dual");

    let Some(mut cfg) = require_or_skip_postgres("spec091waved_dualtol") else {
        std::env::remove_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY");
        return;
    };
    // Random prefix: `eq_<rand>_kv` has never existed (post-drop state).
    cfg.namespace = format!("dualtol{}", &Uuid::new_v4().simple().to_string()[..12]);
    let pool = contract_pg_pool(&cfg).await;
    let kv = PostgresKVStorage::with_pool(
        edgequake_storage::adapters::postgres::PostgresPool::from_existing(
            pool.clone(),
            cfg.clone(),
        ),
        cfg.clone(),
    );
    kv.initialize().await.expect("initialize without DDL");

    // Dual mode routes this chunk key into the raw KV batch write; the missing
    // table must be a no-op, not "KV upsert failed: relation ... does not exist".
    let doc = Uuid::new_v4();
    let chunk_key = format!("{doc}-chunk-0");
    kv.upsert(&[(chunk_key, json!({"content": "text", "index": 0}))])
        .await
        .expect("dual-mode chunk upsert must tolerate the dropped KV table");

    std::env::remove_var("EDGEQUAKE_CHUNK_TEXT_AUTHORITY");
}

/// 8. EC-35 verify-stop: with the KV source dropped, the post-backfill
///    verification phase must report a clean pass (expected=0, mismatches=0),
///    not a 42P01 error (dev boots run the engine in `automatic` mode).
#[tokio::test]
async fn e2e_spec091_chunk_backfill_verify_passes_when_kv_dropped() {
    use edgequake_storage::migration_engine::{
        chunk_text_backfill::ChunkTextBackfillJob, BackfillJob,
    };

    let Some(cfg) = require_or_skip_postgres("spec091waved_verify") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;
    let job = ChunkTextBackfillJob::new("public.eq_spec091waved_missing_verify_kv".to_string());

    let report = job
        .verify(&pool)
        .await
        .expect("verify must tolerate a dropped KV source");
    assert!(
        report.passes(),
        "dropped KV source must yield a passing report (expected=0, mismatches=0)"
    );
    assert_eq!(report.expected, 0);
    assert_eq!(report.sampled, 0);
    assert_eq!(report.mismatches, 0);
}

/// 9. Serving-fence regression (realized 2026-07-29): the fence JOIN must resolve
///    against `public.chunk_serving_state` (the SSOT, where the write path inserts),
///    never the `edgequake` compat schema (which exposes no `chunk_serving_state`
///    view). Previously `JOIN edgequake.chunk_serving_state` errored "relation does
///    not exist"; the `?` propagated and failed the *entire* vector search, so with
///    `EDGEQUAKE_SERVING_FENCE=on` every query degraded to 0 chunks/entities/
///    relationships and the LLM hallucinated an ungrounded answer. This test runs the
///    real `apply_serving_fence` with the fence ON against a live DB — the gap that
///    let the bug ship (prior e2e ran with the fence OFF, making the call a no-op).
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn e2e_spec091_serving_fence_ready_join_executes_on_live_db() {
    let _guard = env_lock();
    let Some(cfg) = require_or_skip_postgres("spec091_fence") else {
        return;
    };
    let pool = contract_pg_pool(&cfg).await;

    let (doc, chunk_uuid, stale_chunk_uuid) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    sqlx::query(
        "INSERT INTO public.documents (id, title, content, status) \
         VALUES ($1, 'fence doc', 'c', 'completed') ON CONFLICT (id) DO NOTHING",
    )
    .bind(doc)
    .execute(&pool)
    .await
    .expect("seed document");
    sqlx::query(
        "INSERT INTO public.chunks (id, document_id, chunk_index, content) \
         VALUES ($1, $2, 0, 'ready chunk'), ($3, $2, 1, 'not-ready chunk')",
    )
    .bind(chunk_uuid)
    .bind(doc)
    .bind(stale_chunk_uuid)
    .execute(&pool)
    .await
    .expect("seed chunks");
    // Only the first chunk is serving-ready; the second stays `embedded`.
    sqlx::query(
        "INSERT INTO public.chunk_serving_state (chunk_id, state) \
         VALUES ($1, 'ready'), ($2, 'embedded') \
         ON CONFLICT (chunk_id) DO UPDATE SET state = EXCLUDED.state",
    )
    .bind(chunk_uuid)
    .bind(stale_chunk_uuid)
    .execute(&pool)
    .await
    .expect("seed serving state");

    let results = vec![
        VectorSearchResult {
            id: format!("{doc}-chunk-0"),
            score: 0.9,
            metadata: json!({}),
        },
        VectorSearchResult {
            id: format!("{doc}-chunk-1"),
            score: 0.8,
            metadata: json!({}),
        },
        // Entity vectors are outside the fence domain and must always pass through.
        VectorSearchResult {
            id: "entity:LIGHTRAG".to_string(),
            score: 0.7,
            metadata: json!({}),
        },
    ];

    // SPEC-091 IP2 / IP-AC-05: unset → fence on; non-ready chunks invisible.
    std::env::remove_var(SERVING_FENCE_ENV);
    let out_default = apply_serving_fence(&pool, results.clone())
        .await
        .expect("default-on fence JOIN must execute");
    let ids_default: Vec<&str> = out_default.iter().map(|r| r.id.as_str()).collect();
    assert!(
        ids_default.contains(&format!("{doc}-chunk-0").as_str()),
        "ready chunk must pass default-on fence"
    );
    assert!(
        !ids_default.contains(&format!("{doc}-chunk-1").as_str()),
        "non-ready chunk must be hidden when EDGEQUAKE_SERVING_FENCE is unset"
    );

    std::env::set_var(SERVING_FENCE_ENV, "on");
    let out = apply_serving_fence(&pool, results)
        .await
        .expect("fence JOIN must execute against public.chunk_serving_state, not error");
    std::env::remove_var(SERVING_FENCE_ENV);

    let ids: Vec<&str> = out.iter().map(|r| r.id.as_str()).collect();
    assert!(
        ids.contains(&format!("{doc}-chunk-0").as_str()),
        "ready chunk must pass the fence"
    );
    assert!(
        !ids.contains(&format!("{doc}-chunk-1").as_str()),
        "non-ready chunk must be hidden by the fence"
    );
    assert!(
        ids.contains(&"entity:LIGHTRAG"),
        "entity vectors are not fenced and must pass through"
    );

    cleanup_doc(&pool, doc).await;
}
