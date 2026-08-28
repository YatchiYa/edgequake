//! SPEC-139 unfakable engine gates (21000, W3 coverage-sum, reclaim, remainder).
//!
//! Run:
//!   DATABASE_URL=… cargo test -p edgequake-storage --features postgres \
//!     --test e2e_spec139_engine -- --nocapture --test-threads=1
#![cfg(feature = "postgres")]

#[path = "support/spec091_fixture.rs"]
mod fixture;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/spec091_w3.rs"]
mod w3;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use edgequake_storage::embedding_family::format_relationship_legacy_key;
use edgequake_storage::entity_id::normalize_entity_name;
use edgequake_storage::error::StorageError;
use edgequake_storage::migration_engine::advisor::residue::kv_durable_residue;
use edgequake_storage::migration_engine::chunk_embedding_backfill::ChunkEmbeddingBackfillJob;
use edgequake_storage::migration_engine::family_remainder::{
    ArtifactRemainderJob, ShellRemainderJob,
};
use edgequake_storage::migration_engine::fleet_embedding_backfill::FleetEmbeddingBackfillJob;
use edgequake_storage::migration_engine::lease::{
    claim_lease, ensure_job_row, reclaim_verify_failed_jobs,
};
use edgequake_storage::migration_engine::{
    run_engine, BackfillJob, BatchOutcome, MigrationEngineConfig, MigrationMode, VerifyReport,
};
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const DIM: usize = 1536;

async fn run_to_completion(pool: &PgPool, job: &dyn BackfillJob) {
    let mut cursor = job.initial_cursor();
    loop {
        let mut tx = pool.begin().await.expect("begin");
        let outcome = job.run_batch(&mut tx, &cursor, 64).await.expect("batch");
        tx.commit().await.expect("commit");
        match outcome.next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
}

async fn seed_entity(pool: &PgPool, ws: Uuid, name: &str) -> Uuid {
    sqlx::query(
        "INSERT INTO public.entities (id, name, workspace_id, entity_type, description) \
         VALUES ($1, $2, $3, 'ORG', '') ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(ws)
    .execute(pool)
    .await
    .expect("seed entity");
    sqlx::query_scalar("SELECT id FROM public.entities WHERE name = $1 AND workspace_id = $2")
        .bind(name)
        .bind(ws)
        .fetch_one(pool)
        .await
        .expect("entity id")
}

async fn ensure_legacy_vector_id(pool: &PgPool) {
    let _ = sqlx::raw_sql(include_str!(
        "../../../migrations/143_spec111_legacy_vector_id.sql"
    ))
    .execute(pool)
    .await;
}

async fn upsert_model(pool: &PgPool, name: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO embedding_models (name, dimensions) VALUES ($1, $2) \
         ON CONFLICT (name, dimensions) DO UPDATE SET name = EXCLUDED.name RETURNING id",
    )
    .bind(name)
    .bind(DIM as i32)
    .fetch_one(pool)
    .await
    .expect("model")
}

fn assert_21000(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) {
    match result {
        Err(sqlx::Error::Database(db)) => {
            assert_eq!(
                db.code().as_deref(),
                Some("21000"),
                "broken UNNEST must raise cardinality 21000, got {:?}",
                db.code()
            );
            eprintln!("UNFAKABLE SQLSTATE=21000");
        }
        other => panic!("expected SQLSTATE 21000, got {other:?}"),
    }
}

/// E2E-139-01: real Postgres 21000 on duplicate arbiter keys; patched batch commits.
#[tokio::test]
async fn e2e_spec139_01_iw2_within_batch_dedupe_avoids_21000() {
    let Some(cfg) = require_or_skip_postgres("spec139_01") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    ensure_legacy_vector_id(&pool).await;
    w3::drop_all_vector_tables_except(&pool, "__none__").await;

    let ws = w3::seed_workspace(&pool, "e2e13901").await;
    let display = "Acme Corp Ltd";
    let eid = seed_entity(&pool, ws, display).await;
    assert_eq!(normalize_entity_name(display), "ACME_CORP_LTD");

    let model_id = upsert_model(&pool, "spec139-iw2-model").await;
    let emb = w3::vector_to_text(&w3::make_embedding(DIM, 7));
    let dup_ids = vec![eid, eid];
    let workspaces = vec![ws, ws];
    let lids = vec![
        "entity:Acme Corp Ltd".to_string(),
        "entity:ACME_CORP_LTD".to_string(),
    ];
    let vectors = vec![emb.clone(), emb.clone()];
    let dims = vec![DIM as i32, DIM as i32];
    let mut probe = pool.begin().await.expect("21000 probe tx");
    let raw = sqlx::query(
        "INSERT INTO entity_embeddings \
         (model_id, entity_id, workspace_id, embedding, dimensions, legacy_vector_id) \
         SELECT $1, e, w, v::halfvec, d, lid \
         FROM unnest($2::uuid[], $3::uuid[], $4::text[], $5::int[], $6::text[]) \
           AS t(e, w, v, d, lid) \
         ON CONFLICT (model_id, entity_id) DO UPDATE \
           SET legacy_vector_id = COALESCE(entity_embeddings.legacy_vector_id, EXCLUDED.legacy_vector_id)",
    )
    .bind(model_id)
    .bind(&dup_ids)
    .bind(&workspaces)
    .bind(&vectors)
    .bind(&dims)
    .bind(&lids)
    .execute(&mut *probe)
    .await;
    assert_21000(raw);
    probe.rollback().await.ok();

    let table = w3::create_vectors_table(&pool, "e2e13901").await;
    let meta = json!({"workspace_id": ws.to_string(), "entity_type": "ORG"});
    for (i, key) in ["entity:Acme Corp Ltd", "entity:ACME_CORP_LTD"]
        .iter()
        .enumerate()
    {
        sqlx::query(&format!(
            "INSERT INTO public.{table} (id, embedding, metadata) VALUES ($1, $2::vector, $3)"
        ))
        .bind(*key)
        .bind(w3::vector_to_text(&w3::make_embedding(DIM, 20 + i as u32)))
        .bind(&meta)
        .execute(&pool)
        .await
        .expect("seed colliding entity vectors");
    }

    sqlx::query("DELETE FROM entity_embeddings WHERE entity_id = $1")
        .bind(eid)
        .execute(&pool)
        .await
        .ok();

    let job = FleetEmbeddingBackfillJob::new("spec139-iw2-model".into());
    run_to_completion(&pool, &job).await;

    let typed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM public.entity_embeddings WHERE entity_id = $1")
            .bind(eid)
            .fetch_one(&pool)
            .await
            .expect("typed count");
    assert_eq!(typed, 1, "one typed row after within-batch dedupe");

    let provenance: Option<String> = sqlx::query_scalar(
        "SELECT legacy_vector_id FROM public.entity_embeddings WHERE entity_id = $1",
    )
    .bind(eid)
    .fetch_one(&pool)
    .await
    .expect("provenance");
    assert!(
        provenance
            .as_deref()
            .is_some_and(|s| s.starts_with("entity:")),
        "legacy_vector_id must be stamped, got {provenance:?}"
    );
    eprintln!("UNFAKABLE E2E-139-01 typed_count={typed} provenance_stamped=1");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

/// E2E-139-02: two relationship keys that collapse to one spine — no 21000.
#[tokio::test]
async fn e2e_spec139_02_iw2_relationship_normalize_dedupe() {
    let Some(cfg) = require_or_skip_postgres("spec139_02") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    ensure_legacy_vector_id(&pool).await;
    w3::drop_all_vector_tables_except(&pool, "__none__").await;

    let ws = w3::seed_workspace(&pool, "e2e13902").await;
    let src = seed_entity(&pool, ws, "Alice").await;
    let tgt = seed_entity(&pool, ws, "Bob").await;
    let rid: Uuid = sqlx::query_scalar(
        "INSERT INTO public.relationships \
            (source_id, target_id, relation_type, workspace_id, description) \
         VALUES ($1, $2, 'WORKS_WITH', $3, '') RETURNING id",
    )
    .bind(src)
    .bind(tgt)
    .bind(ws)
    .fetch_one(&pool)
    .await
    .expect("relationship spine");

    let model_id = upsert_model(&pool, "spec139-rel-model").await;
    let emb = w3::vector_to_text(&w3::make_embedding(DIM, 8));
    let mut probe = pool.begin().await.expect("rel 21000 probe");
    let raw = sqlx::query(
        "INSERT INTO relationship_embeddings \
         (model_id, relationship_id, workspace_id, embedding, dimensions, legacy_vector_id) \
         SELECT $1, r, w, v::halfvec, d, lid \
         FROM unnest($2::uuid[], $3::uuid[], $4::text[], $5::int[], $6::text[]) \
           AS t(r, w, v, d, lid) \
         ON CONFLICT (model_id, relationship_id) DO UPDATE \
           SET legacy_vector_id = COALESCE(relationship_embeddings.legacy_vector_id, EXCLUDED.legacy_vector_id)",
    )
    .bind(model_id)
    .bind(vec![rid, rid])
    .bind(vec![ws, ws])
    .bind(vec![emb.clone(), emb.clone()])
    .bind(vec![DIM as i32, DIM as i32])
    .bind(vec![
        "Alice->Bob:WORKS_WITH".to_string(),
        "Alice->Bob:works_with".to_string(),
    ])
    .execute(&mut *probe)
    .await;
    assert_21000(raw);
    probe.rollback().await.ok();

    let upper = format_relationship_legacy_key("Alice", "Bob", "WORKS_WITH");
    let lower = "Alice->Bob:works_with".to_string();
    assert_ne!(upper, lower, "fixture must propose two surface keys");

    let table = w3::create_vectors_table(&pool, "e2e13902").await;
    let meta = json!({"workspace_id": ws.to_string(), "entity_type": "ORG"});
    for (i, key) in [upper.as_str(), lower.as_str()].iter().enumerate() {
        sqlx::query(&format!(
            "INSERT INTO public.{table} (id, embedding, metadata) VALUES ($1, $2::vector, $3)"
        ))
        .bind(*key)
        .bind(w3::vector_to_text(&w3::make_embedding(DIM, 30 + i as u32)))
        .bind(&meta)
        .execute(&pool)
        .await
        .expect("seed colliding rel vectors");
    }

    let job = FleetEmbeddingBackfillJob::new("spec139-rel-model".into());
    run_to_completion(&pool, &job).await;

    let typed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.relationship_embeddings re \
         JOIN public.relationships r ON r.id = re.relationship_id \
         WHERE r.workspace_id = $1",
    )
    .bind(ws)
    .fetch_one(&pool)
    .await
    .expect("rel typed count");
    assert_eq!(typed, 1, "one relationship embedding after dedupe");
    eprintln!("UNFAKABLE E2E-139-02 typed_count={typed}");

    w3::drop_table(&pool, &table).await;
    w3::cleanup_workspace(&pool, ws).await;
}

/// E2E-139-03: W3 actual is coverage SUM, never global COUNT(*) / max().
#[tokio::test]
async fn e2e_spec139_03_w3_verify_coverage_sum_not_global_count() {
    let Some(cfg) = require_or_skip_postgres("spec139_03") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::drop_all_vector_tables_except(&pool, "__none__").await;

    let ws_gap = w3::seed_workspace(&pool, "e2e13903g").await;
    let ws_extra = w3::seed_workspace(&pool, "e2e13903x").await;
    let doc_a = w3::seed_document(&pool, ws_gap).await;
    let doc_b = w3::seed_document(&pool, ws_gap).await;
    let table_a = w3::create_vectors_table(&pool, "e2e13903a").await;
    let table_b = w3::create_vectors_table(&pool, "e2e13903b").await;
    for i in 0..3 {
        w3::seed_legacy_chunk_vector(
            &pool,
            &table_a,
            doc_a,
            i,
            &w3::make_embedding(DIM, 40 + i as u32),
        )
        .await;
    }
    for i in 0..5 {
        w3::seed_legacy_chunk_vector(
            &pool,
            &table_b,
            doc_b,
            i,
            &w3::make_embedding(DIM, 50 + i as u32),
        )
        .await;
    }
    sqlx::query(&format!(
        "INSERT INTO public.{table_a} (id, embedding) VALUES ($1, $2::vector) \
         ON CONFLICT (id) DO NOTHING"
    ))
    .bind("not-a-uuid-chunk-0")
    .bind(w3::vector_to_text(&w3::make_embedding(DIM, 1)))
    .execute(&pool)
    .await
    .expect("malformed chunk id must not inflate expected");

    let extra_doc = w3::seed_document(&pool, ws_extra).await;
    for i in 0..10 {
        w3::seed_chunk(&pool, extra_doc, ws_extra, i, &format!("extra {i}")).await;
    }
    w3::seed_typed_embeddings_bulk(&pool, ws_extra, "spec139-w3-model", DIM, 90).await;

    let global: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.chunk_embeddings")
        .fetch_one(&pool)
        .await
        .expect("global typed");
    assert!(
        global >= 10,
        "fixture needs extra typed rows so old COUNT(*) would inflate actual (got {global})"
    );

    let job = ChunkEmbeddingBackfillJob::new(table_a.clone(), "spec139-w3-model".into());
    let report = job.verify(&pool).await.expect("verify uncovered");
    assert_eq!(report.expected, 8, "SUM of per-table legacy chunks 3+5");
    assert_eq!(
        report.actual, 0,
        "coverage SUM must ignore unrelated global typed rows (old max/COUNT would see {global})"
    );
    assert!(!report.passes(), "uncovered 8 must fail coverage");

    for i in 0..3 {
        w3::seed_chunk(&pool, doc_a, ws_gap, i, &format!("a{i}")).await;
    }
    for i in 0..5 {
        w3::seed_chunk(&pool, doc_b, ws_gap, i, &format!("b{i}")).await;
    }
    w3::seed_typed_embeddings_bulk(&pool, ws_gap, "spec139-w3-model", DIM, 70).await;

    let covered = job.verify(&pool).await.expect("verify covered");
    assert_eq!(covered.expected, 8);
    assert_eq!(
        covered.actual, 8,
        "coverage SUM is 3+5, not global COUNT(*)={global} or max()"
    );
    assert!(covered.passes());
    eprintln!(
        "UNFAKABLE E2E-139-03 expected={} actual_uncovered={} actual_covered={} global_typed={}",
        report.expected, report.actual, covered.actual, global
    );

    w3::drop_table(&pool, &table_a).await;
    w3::drop_table(&pool, &table_b).await;
    w3::cleanup_workspace(&pool, ws_gap).await;
    w3::cleanup_workspace(&pool, ws_extra).await;
}

/// E2E-139-04: reclaim verify-failed W3 so claim_lease succeeds.
#[tokio::test]
async fn e2e_spec139_04_reclaim_verify_failed_w3() {
    let Some(cfg) = require_or_skip_postgres("spec139_04") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    w3::clear_w3_job(&pool).await;

    let job = ChunkEmbeddingBackfillJob::new("unused".into(), "spec139-w3-model".into());
    ensure_job_row(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "reversible",
        64,
        Some(0),
    )
    .await
    .expect("ensure");
    sqlx::query(
        "UPDATE edgequake.edgequake_migration_job \
         SET state = 'failed', last_error = $1, cursor_position = '{\"last_id\":\"zzz\"}' \
         WHERE step_id = $2 AND schema_generation = $3",
    )
    .bind(json!({"verify_failed": {"metric": "w3", "expected": 1, "actual": 0}}))
    .bind(job.step_id())
    .bind(job.schema_generation())
    .execute(&pool)
    .await
    .expect("plant failed");

    let pre = claim_lease(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "e2e139-pre",
        30,
    )
    .await
    .expect("claim pre");
    assert!(
        pre.is_none(),
        "failed jobs must not be claimed before reclaim"
    );

    let n = reclaim_verify_failed_jobs(&pool).await.expect("reclaim");
    assert!(n >= 1, "must reclaim the planted verify_failed row");

    let post = claim_lease(
        &pool,
        job.step_id(),
        &job.step_sha384(),
        job.schema_generation(),
        "e2e139-post",
        30,
    )
    .await
    .expect("claim post");
    let lease = post.expect("reclaimed job must be claimable");
    assert!(
        lease.cursor_position.is_null() || lease.cursor_position == json!({}),
        "reclaim must reset cursor, got {:?}",
        lease.cursor_position
    );
    eprintln!("UNFAKABLE E2E-139-04 reclaimed={n} claim_after=1");

    w3::clear_w3_job(&pool).await;
}

/// E2E-139-05: 119-before-122 residue; remainder copies lineage after shell.
#[tokio::test]
async fn e2e_spec139_05_artifact_remainder_after_shell() {
    let Some(cfg) = require_or_skip_postgres("spec139_05") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    fixture::drop_all_kv_tables(&pool).await;

    let doc_id = Uuid::new_v4();
    let table = fixture::create_kv_table(&pool, "e2e139art").await;
    fixture::seed_kv_row(
        &pool,
        &table,
        &format!("{doc_id}-lineage"),
        json!({"src": "field"}),
    )
    .await;

    let uuid_re = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
    let kind_re = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}-(lineage|multimodal-manifest|multimodal-chunks)$";
    sqlx::query(&format!(
        "INSERT INTO public.document_artifacts (document_id, kind, payload) \
         SELECT left(kv.key, 36)::uuid, substring(kv.key FROM 38), kv.value \
         FROM public.{table} kv \
         WHERE kv.key ~ $1 AND left(kv.key, 36) ~ $2 \
           AND EXISTS (SELECT 1 FROM public.documents d WHERE d.id = left(kv.key, 36)::uuid) \
         ON CONFLICT (document_id, kind) DO UPDATE SET payload = EXCLUDED.payload"
    ))
    .bind(kind_re)
    .bind(uuid_re)
    .execute(&pool)
    .await
    .expect("119-shaped skip");

    let before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM public.document_artifacts WHERE document_id = $1")
            .bind(doc_id)
            .fetch_one(&pool)
            .await
            .expect("before");
    assert_eq!(before, 0, "119 must skip parent-less lineage");

    sqlx::query(
        "INSERT INTO public.documents (id, title, content, status, metadata) \
         VALUES ($1, '', '', 'indexed', '{}'::jsonb)",
    )
    .bind(doc_id)
    .execute(&pool)
    .await
    .expect("122-shaped shell");

    let job = ArtifactRemainderJob::new();
    run_to_completion(&pool, &job).await;

    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM public.document_artifacts \
         WHERE document_id = $1 AND kind = 'lineage'",
    )
    .bind(doc_id)
    .fetch_one(&pool)
    .await
    .expect("after");
    assert_eq!(after, 1, "remainder must insert lineage once parent exists");

    let residue = kv_durable_residue(&pool, &table).await.expect("residue");
    assert_eq!(residue.lineage, 0, "lineage residue must clear");
    let v = job.verify(&pool).await.expect("remainder verify");
    assert!(
        v.passes(),
        "copy-complete verify must pass when leftover is 0"
    );
    eprintln!(
        "UNFAKABLE E2E-139-05 before_artifacts={before} after_lineage={after} residue_lineage={}",
        residue.lineage
    );

    sqlx::query("DELETE FROM public.document_artifacts WHERE document_id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .ok();
    fixture::drop_all_kv_tables(&pool).await;
}

/// E2E-139-07: 122-shaped shell remainder copies metadata keys into documents.
#[tokio::test]
async fn e2e_spec139_07_shell_remainder_copies_metadata() {
    let Some(cfg) = require_or_skip_postgres("spec139_07") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    fixture::drop_all_kv_tables(&pool).await;

    let doc_id = Uuid::new_v4();
    let table = fixture::create_kv_table(&pool, "e2e139sh").await;
    fixture::seed_kv_row(
        &pool,
        &table,
        &format!("{doc_id}-metadata"),
        json!({"title": "shell"}),
    )
    .await;

    let before = kv_durable_residue(&pool, &table).await.expect("before");
    assert!(before.doc_shells >= 1, "fixture must be shell residue");

    let job = ShellRemainderJob::new();
    run_to_completion(&pool, &job).await;
    assert!(job.verify(&pool).await.expect("verify").passes());

    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM public.documents WHERE id = $1)")
            .bind(doc_id)
            .fetch_one(&pool)
            .await
            .expect("doc");
    assert!(exists, "shell remainder must insert documents row");
    let after = kv_durable_residue(&pool, &table).await.expect("after");
    assert_eq!(after.doc_shells, 0);
    eprintln!(
        "UNFAKABLE E2E-139-07 doc_shells_before={} doc_exists=1 doc_shells_after={}",
        before.doc_shells, after.doc_shells
    );

    sqlx::query("DELETE FROM public.documents WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .ok();
    fixture::drop_all_kv_tables(&pool).await;
}

/// E2E-139-08: orphan lineage does not fail remainder verify (no fail-loop).
#[tokio::test]
async fn e2e_spec139_08_orphan_lineage_remainder_verify_passes() {
    let Some(cfg) = require_or_skip_postgres("spec139_08") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    fixture::drop_all_kv_tables(&pool).await;

    let orphan = Uuid::new_v4();
    let table = fixture::create_kv_table(&pool, "e2e139orp").await;
    fixture::seed_kv_row(
        &pool,
        &table,
        &format!("{orphan}-lineage"),
        json!({"src": "orphan"}),
    )
    .await;

    let job = ArtifactRemainderJob::new();
    run_to_completion(&pool, &job).await;
    let v = job.verify(&pool).await.expect("verify");
    assert!(
        v.passes(),
        "remainder must complete even when leftover orphans remain"
    );
    let residue = kv_durable_residue(&pool, &table).await.expect("residue");
    assert!(
        residue.lineage >= 1,
        "orphan must remain advisor-RED (DROP gate), got {}",
        residue.lineage
    );
    eprintln!(
        "UNFAKABLE E2E-139-08 verify_passes=1 residue_lineage={}",
        residue.lineage
    );

    fixture::drop_all_kv_tables(&pool).await;
}

struct BoomJob;

#[async_trait]
impl BackfillJob for BoomJob {
    fn step_id(&self) -> &'static str {
        "e2e-139-06-boom"
    }
    fn step_sha384(&self) -> String {
        "aa".repeat(48)
    }
    fn schema_generation(&self) -> i32 {
        1
    }
    fn initial_cursor(&self) -> Value {
        json!({})
    }
    async fn estimate_total(&self, _pool: &PgPool) -> Result<i64, StorageError> {
        Ok(0)
    }
    async fn run_batch(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _cursor: &Value,
        _limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        Err(StorageError::Database("e2e139 boom 21000 stand-in".into()))
    }
    async fn verify(&self, _pool: &PgPool) -> Result<VerifyReport, StorageError> {
        Ok(VerifyReport {
            metric: "boom".into(),
            expected: 0,
            actual: 0,
            sampled: 0,
            mismatches: 0,
        })
    }
}

struct FlagJob {
    flag: Arc<AtomicBool>,
}

#[async_trait]
impl BackfillJob for FlagJob {
    fn step_id(&self) -> &'static str {
        "e2e-139-06-ok"
    }
    fn step_sha384(&self) -> String {
        "bb".repeat(48)
    }
    fn schema_generation(&self) -> i32 {
        1
    }
    fn initial_cursor(&self) -> Value {
        json!({})
    }
    async fn estimate_total(&self, _pool: &PgPool) -> Result<i64, StorageError> {
        Ok(0)
    }
    async fn run_batch(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _cursor: &Value,
        _limit: i64,
    ) -> Result<BatchOutcome, StorageError> {
        self.flag.store(true, Ordering::SeqCst);
        Ok(BatchOutcome {
            scanned: 0,
            written: 0,
            failed: 0,
            next_cursor: None,
        })
    }
    async fn verify(&self, _pool: &PgPool) -> Result<VerifyReport, StorageError> {
        Ok(VerifyReport {
            metric: "ok".into(),
            expected: 0,
            actual: 0,
            sampled: 0,
            mismatches: 0,
        })
    }
}

/// E2E-139-06: run_engine continues after the first job's run_batch Err.
#[tokio::test]
async fn e2e_spec139_06_run_engine_continues_after_job_err() {
    let Some(cfg) = require_or_skip_postgres("spec139_06") else {
        return;
    };
    let _g = w3::w3_lock().lock().await;
    let pool = contract_pg_pool(&cfg).await;
    sqlx::query(
        "DELETE FROM edgequake.edgequake_migration_batch WHERE job_id IN \
         (SELECT job_id FROM edgequake.edgequake_migration_job \
          WHERE step_id IN ('e2e-139-06-boom', 'e2e-139-06-ok'))",
    )
    .execute(&pool)
    .await
    .ok();
    sqlx::query(
        "DELETE FROM edgequake.edgequake_migration_job \
         WHERE step_id IN ('e2e-139-06-boom', 'e2e-139-06-ok')",
    )
    .execute(&pool)
    .await
    .ok();

    let flag = Arc::new(AtomicBool::new(false));
    let jobs: Vec<Arc<dyn BackfillJob>> =
        vec![Arc::new(BoomJob), Arc::new(FlagJob { flag: flag.clone() })];
    let mut config = MigrationEngineConfig::from_env();
    config.owner = "e2e139-06".into();
    run_engine(pool.clone(), jobs, config, MigrationMode::Automatic)
        .await
        .expect("engine must not abort the loop");
    assert!(
        flag.load(Ordering::SeqCst),
        "second job must run after first job Err"
    );
    eprintln!("UNFAKABLE E2E-139-06 second_job_ran=1");
}
