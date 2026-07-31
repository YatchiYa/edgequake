//! SPEC-091 W4: legacy chunk-vector retirement — fleet-wide engine verify →
//! `VectorPosture.retirable` → guarded migration 126 (delete covered chunk
//! rows, drop chunk-dedicated tables, keep entity/rel/report vectors).
//!
//! The migration-126 SQL is executed directly (idempotent DO blocks) so the
//! guard + drop semantics are exercised regardless of the shared DB's
//! `_sqlx_migrations` state.
//!
//! Run:
//!   DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
//!     cargo test -p edgequake-storage --features postgres --test e2e_spec091_vector_retire -- --nocapture
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use edgequake_storage::migration_engine::advisor;
use edgequake_storage::migration_engine::chunk_embedding_backfill::ChunkEmbeddingBackfillJob;
use edgequake_storage::migration_engine::BackfillJob;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use sqlx::PgPool;

const DIM: usize = 1536;
const MIGRATION_126: &str = include_str!("../../../migrations/126_spec091_vector_drop.sql");

/// Drive the fleet job to exhaustion, then verify (mirrors the runner loop).
async fn run_to_completion(pool: &PgPool, job: &dyn BackfillJob) {
    let mut cursor = job.initial_cursor();
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let outcome = job.run_batch(&mut tx, &cursor, 8).await.expect("batch");
        tx.commit().await.expect("commit");
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
}

async fn table_exists(pool: &PgPool, table: &str) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = $1)",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .expect("table exists")
}

async fn row_count(pool: &PgPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM public.{table}"))
        .fetch_one(pool)
        .await
        .expect("row count")
}

#[tokio::test]
async fn e2e_spec091_retire_guard_aborts_on_uncovered_chunk() {
    let Some(cfg) = require_or_skip_postgres("spec091_w4_guard") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "guard").await;
    let doc = w3::seed_document(&pool, ws).await;
    w3::seed_chunk(&pool, doc, ws, 0, "uncovered chunk").await;
    // A legacy chunk vector with NO typed row → must abort the drop.
    let table = w3::create_vectors_table(&pool, "w4guard").await;
    w3::seed_legacy_chunk_vector(&pool, &table, doc, 0, &w3::make_embedding(DIM, 1)).await;

    let res = sqlx::raw_sql(MIGRATION_126).execute(&pool).await;
    assert!(
        res.is_err(),
        "guard must abort when a legacy chunk row is not covered in chunk_embeddings"
    );
    let msg = format!("{res:?}");
    assert!(
        msg.contains("SPEC-091 W4 ABORT"),
        "abort message must name the W4 guard: {msg}"
    );

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_retire_drops_covered_chunk_rows_and_dedicated_table() {
    let Some(cfg) = require_or_skip_postgres("spec091_w4_drop") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "drop").await;
    let doc = w3::seed_document(&pool, ws).await;
    for i in 0..4 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    // Chunk-dedicated legacy table (only chunk rows).
    let table = w3::create_vectors_table(&pool, "w4drop").await;
    for i in 0..4 {
        w3::seed_legacy_chunk_vector(
            &pool,
            &table,
            doc,
            i,
            &w3::make_embedding(DIM, 10 + i as u32),
        )
        .await;
    }

    // Run the fleet backfill so every chunk row is covered in typed SSOT.
    let job = ChunkEmbeddingBackfillJob::new(String::new(), "w4-drop-model".into());
    run_to_completion(&pool, &job).await;
    let report = job.verify(&pool).await.expect("verify");
    assert!(report.passes(), "fleet verify must pass before retire");
    let typed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("typed");
    assert_eq!(typed, 4);

    // Advisor posture must report the fleet retirable (backend flipped + 0 residue
    // after the drop deletes rows — but pre-drop the guard drives it). Run 126.
    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "chunk_embeddings");
    sqlx::raw_sql(MIGRATION_126)
        .execute(&pool)
        .await
        .expect("migration 126 applies once covered");

    // Chunk-dedicated table is dropped.
    assert!(
        !table_exists(&pool, &table).await,
        "chunk-dedicated table must be dropped after retire"
    );
    // Typed rows are untouched (the SSOT survives the drop).
    let typed_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("typed after");
    assert_eq!(typed_after, 4, "typed SSOT must survive the drop");
    std::env::remove_var("EDGEQUAKE_VECTOR_BACKEND");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_retire_keeps_entity_vectors_in_shared_table() {
    let Some(cfg) = require_or_skip_postgres("spec091_w4_shared") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "shared").await;
    let doc = w3::seed_document(&pool, ws).await;
    w3::seed_chunk(&pool, doc, ws, 0, "shared chunk").await;
    // Shared table: one chunk row + one NON-chunk (entity) row.
    let table = w3::create_vectors_table(&pool, "w4shared").await;
    w3::seed_legacy_chunk_vector(&pool, &table, doc, 0, &w3::make_embedding(DIM, 7)).await;
    sqlx::query(&format!(
        "INSERT INTO public.{table} (id, embedding) VALUES ('entity:SARAH_CHEN', $1::vector) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind(w3::vector_to_text(&w3::make_embedding(DIM, 8)))
    .execute(&pool)
    .await
    .expect("seed entity vector");

    // Cover the chunk row via the fleet backfill.
    let job = ChunkEmbeddingBackfillJob::new(String::new(), "w4-shared-model".into());
    run_to_completion(&pool, &job).await;
    let report = job.verify(&pool).await.expect("verify");
    assert!(report.passes());

    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "chunk_embeddings");
    sqlx::raw_sql(MIGRATION_126)
        .execute(&pool)
        .await
        .expect("migration 126 applies");

    // Shared table survives (it still holds the entity vector) — chunk rows gone.
    assert!(
        table_exists(&pool, &table).await,
        "shared table with entity vectors must be kept (out of W4 scope)"
    );
    assert_eq!(
        row_count(&pool, &table).await,
        1,
        "only the entity row remains; chunk rows deleted"
    );
    let remaining_id: String = sqlx::query_scalar(&format!("SELECT id FROM public.{table}"))
        .fetch_one(&pool)
        .await
        .expect("remaining id");
    assert_eq!(remaining_id, "entity:SARAH_CHEN");
    std::env::remove_var("EDGEQUAKE_VECTOR_BACKEND");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

#[tokio::test]
async fn e2e_spec091_retire_fleet_spans_multiple_legacy_tables() {
    let Some(cfg) = require_or_skip_postgres("spec091_w4_fleet") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let ws = w3::seed_workspace(&pool, "fleet").await;
    let doc = w3::seed_document(&pool, ws).await;
    for i in 0..6 {
        w3::seed_chunk(&pool, doc, ws, i, &format!("chunk {i}")).await;
    }
    // Two legacy tables (shared + a per-workspace one) — the fleet job must
    // traverse both.
    let t1 = w3::create_vectors_table(&pool, "w4fleet_a").await;
    let t2 = w3::create_vectors_table(&pool, "w4fleet_b").await;
    for i in 0..3 {
        w3::seed_legacy_chunk_vector(&pool, &t1, doc, i, &w3::make_embedding(DIM, 20 + i as u32))
            .await;
    }
    for i in 3..6 {
        w3::seed_legacy_chunk_vector(&pool, &t2, doc, i, &w3::make_embedding(DIM, 30 + i as u32))
            .await;
    }

    let job = ChunkEmbeddingBackfillJob::new(String::new(), "w4-fleet-model".into());
    assert_eq!(job.estimate_total(&pool).await.expect("estimate"), 6);
    run_to_completion(&pool, &job).await;

    let typed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM chunk_embeddings WHERE workspace_id = $1")
            .bind(ws)
            .fetch_one(&pool)
            .await
            .expect("typed");
    assert_eq!(
        typed, 6,
        "fleet backfill must cover chunks across both tables"
    );
    let report = job.verify(&pool).await.expect("verify");
    assert_eq!(report.expected, 6);
    assert!(report.passes());

    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "chunk_embeddings");
    sqlx::raw_sql(MIGRATION_126)
        .execute(&pool)
        .await
        .expect("migration 126 applies");
    assert!(!table_exists(&pool, &t1).await, "t1 dropped");
    assert!(!table_exists(&pool, &t2).await, "t2 dropped");
    std::env::remove_var("EDGEQUAKE_VECTOR_BACKEND");

    w3::drop_table(&pool, &t1).await;
    w3::drop_table(&pool, &t2).await;
    w3::cleanup_workspace(&pool, ws).await;
}

// ---------------------------------------------------------------------------
// Drift-guard contract (mirrors contract_spec091_advisor_matches_125_guard):
// the migration-126 in-SQL guard must abort exactly when the advisor's
// VectorPosture is NOT retirable, and pass exactly when it is — on both a
// residue-bearing and a fully-covered fleet.
// ---------------------------------------------------------------------------

/// Extract the W4 coverage-guard `DO $$ ... END $$;` block (the one that
/// raises `SPEC-091 W4 ABORT`) from migration 126.
fn extract_126_guard_sql() -> String {
    let marker = "SPEC-091 W4 ABORT";
    let marker_at = MIGRATION_126
        .find(marker)
        .expect("guard abort message in migration 126");
    let start = MIGRATION_126[..marker_at]
        .rfind("DO $$")
        .expect("guard DO block start");
    let end = MIGRATION_126[start..]
        .find("END $$;")
        .map(|i| start + i + "END $$;".len())
        .expect("guard DO block end");
    MIGRATION_126[start..end].to_string()
}

#[tokio::test]
async fn contract_spec091_advisor_matches_126_guard() {
    let Some(cfg) = require_or_skip_postgres("spec091_w4_parity") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;
    std::env::set_var("EDGEQUAKE_VECTOR_BACKEND", "chunk_embeddings");
    std::env::set_var("EDGEQUAKE_EMBEDDING_MODEL", "w4-parity-model");

    // 1) Residue-bearing fleet (uncovered chunk) → guard aborts AND posture
    //    is not retirable.
    let ws = w3::seed_workspace(&pool, "parity").await;
    let doc = w3::seed_document(&pool, ws).await;
    w3::seed_chunk(&pool, doc, ws, 0, "parity chunk").await;
    let table = w3::create_vectors_table(&pool, "w4parity").await;
    w3::seed_legacy_chunk_vector(&pool, &table, doc, 0, &w3::make_embedding(DIM, 42)).await;

    let guard_sql = extract_126_guard_sql();
    let guard = sqlx::raw_sql(&guard_sql).execute(&pool).await;
    assert!(
        guard.is_err(),
        "126 guard must abort while a chunk row is uncovered"
    );
    let posture = advisor::posture(&pool).await.expect("posture");
    assert!(
        !posture.vector.retirable(),
        "advisor must not report retirable while residue remains (drift-guard)"
    );
    assert!(
        advisor::derive_actions(&posture)
            .iter()
            .any(|a| a.verb == "drop" && a.target == "vector-legacy" && !a.enabled),
        "drop vector-legacy must be gated while residue remains"
    );

    // 2) Cover the chunk via the fleet backfill → guard passes AND (after the
    //    drop drains legacy rows) posture is retirable.
    let job = ChunkEmbeddingBackfillJob::new(String::new(), "w4-parity-model".into());
    run_to_completion(&pool, &job).await;
    assert!(job.verify(&pool).await.expect("verify").passes());

    sqlx::raw_sql(&guard_sql)
        .execute(&pool)
        .await
        .expect("126 guard passes once every chunk is covered");

    // After the full drop, legacy chunk rows are 0 → retirable flips true.
    sqlx::raw_sql(MIGRATION_126)
        .execute(&pool)
        .await
        .expect("migration 126 applies");
    let posture = advisor::posture(&pool).await.expect("posture post-drop");
    assert_eq!(posture.vector.legacy_chunk_rows, 0);
    // Fleet fully dropped → posture reports dropped (terminal, idempotent).
    assert!(
        posture.vector.dropped || posture.vector.retirable(),
        "post-drop fleet is dropped or retirable (got legacy={}, dropped={})",
        posture.vector.legacy_chunk_rows,
        posture.vector.dropped
    );

    std::env::remove_var("EDGEQUAKE_VECTOR_BACKEND");
    std::env::remove_var("EDGEQUAKE_EMBEDDING_MODEL");
    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}
