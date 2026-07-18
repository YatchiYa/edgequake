//! SPEC-078 — Wave-2 vs post-filter DiskANN vs Filtered-DiskANN labels bake-off.
//!
//! Soft-skip without DB / vectorscale. Hang cliff hard-fails. Does not raise floors.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;
#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/perf_ann_corpus.rs"]
mod perf_ann_corpus;

use edgequake_storage::traits::VectorStorage;
use edgequake_storage::{
    build_diskann_embedding_only_index_sql, build_diskann_labels_index_sql,
    build_filtered_diskann_label_select_sql, build_postfilter_diskann_select_sql,
    diskann_optin_recipe_statements, PgVectorStorage, VectorIndexType, VectorStorageMode,
    WorkspaceLabelMap,
};
use perf_ann_corpus::{emb, seed_ws_split, workspace_filter};
use sqlx::{PgPool, Row};
use std::time::Instant;

const DIM: usize = 64;
const DEFAULT_ROWS: u32 = 2_000;
const TOP_K: usize = 20;
const HOT_WS: &str = "ws-a";
const COLD_WS: &str = "ws-b";
const TENANT: &str = "t-fdl078";
const DEFAULT_HANG_CLIFF_MS: f64 = 5_000.0;
const RECALL_SOFT: f64 = 0.90;

fn hang_cliff_ms() -> f64 {
    std::env::var("EQ_FDL_HANG_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_HANG_CLIFF_MS)
}

fn recall_at_k(reference: &[String], candidate: &[String]) -> f64 {
    if reference.is_empty() {
        return 1.0;
    }
    let set: std::collections::HashSet<&String> = candidate.iter().collect();
    let hit = reference.iter().filter(|id| set.contains(id)).count();
    hit as f64 / reference.len() as f64
}

fn emit(op: &str, p95_ms: f64, pass: bool, plan_class: &str, detail: impl Into<String>) {
    println!(
        "PERF_REPORT {}",
        serde_json::json!({
            "profile": std::env::var("EQ_POSTGRES_PROFILE").unwrap_or_else(|_| "unknown".into()),
            "op": op,
            "p95_ms": p95_ms,
            "plan_class": plan_class,
            "pass": pass,
            "detail": detail.into(),
        })
    );
}

fn emb_literal(v: &[f32]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|x| format!("{x}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

async fn apply_diskann_recipe(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for stmt in diskann_optin_recipe_statements() {
        sqlx::query(&stmt).execute(&mut *tx).await?;
    }
    // Keep planner on index for small smoke corpora.
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

async fn seed_study_table(
    pool: &PgPool,
    table: &str,
    rows: usize,
    label_hot: i16,
    label_cold: i16,
) -> f64 {
    let wall = Instant::now();
    let batch = 400usize;
    for start in (0..rows).step_by(batch) {
        let end = (start + batch).min(rows);
        let mut values = Vec::with_capacity(end - start);
        for i in start..end {
            let ws = if i % 5 == 0 { HOT_WS } else { COLD_WS };
            let lab = if ws == HOT_WS { label_hot } else { label_cold };
            let e = emb_literal(&emb(DIM, i as f32));
            values.push(format!(
                "('fdl078-{i}', '{e}'::vector, '{ws}', ARRAY[{lab}]::smallint[])"
            ));
        }
        let sql = format!(
            "INSERT INTO {table} (id, embedding, workspace_id, labels) VALUES {} \
             ON CONFLICT (id) DO UPDATE SET embedding = EXCLUDED.embedding, \
               workspace_id = EXCLUDED.workspace_id, labels = EXCLUDED.labels",
            values.join(",")
        );
        sqlx::query(&sql)
            .execute(pool)
            .await
            .expect("study batch insert");
    }
    wall.elapsed().as_secs_f64() * 1000.0
}

#[tokio::test]
async fn e2e_spec078_filtered_diskann_labels_bakeoff() {
    let rows: u32 = std::env::var("EQ_FDL_ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_ROWS);

    let Some(base) = postgres_test_config::contract_postgres_config("fdl078") else {
        eprintln!("SKIP SPEC-078: DATABASE_URL / POSTGRES_PASSWORD not set");
        return;
    };

    std::env::set_var("EDGEQUAKE_VECTOR_STORAGE", "halfvec");
    std::env::remove_var("EDGEQUAKE_FILTERED_DISKANN_LABELS");

    let mut config = base.with_vector_index(VectorIndexType::HNSW);
    config.max_connections = 8;
    config.hnsw_m = 16;
    config.hnsw_ef_construction = 64;

    let storage =
        PgVectorStorage::with_dimension(config.clone(), DIM).with_storage_mode(VectorStorageMode::Half);
    if let Err(e) = storage.initialize().await {
        eprintln!("SKIP SPEC-078: wave2 init failed ({e})");
        return;
    }

    let seed_ms = seed_ws_split(
        &storage,
        rows as usize,
        DIM,
        400,
        "fdl078",
        TENANT,
        HOT_WS,
        COLD_WS,
    )
    .await;
    emit(
        "fdl078_seed",
        seed_ms,
        true,
        "wave2_halfvec",
        format!("rows={rows} FILTERED bake-off"),
    );

    let mf = workspace_filter(HOT_WS, TENANT);
    let q = emb(DIM, 11.0);

    let t0 = Instant::now();
    let wave2 = storage
        .query_filtered(&q, TOP_K, None, Some(&mf))
        .await
        .expect("wave2 filtered");
    let wave2_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let hang = hang_cliff_ms();
    assert!(wave2_ms < hang, "wave2 hang cliff {wave2_ms} >= {hang}");
    let wave2_ids: Vec<String> = wave2.into_iter().map(|h| h.id).collect();
    emit(
        "fdl078_cell",
        wave2_ms,
        !wave2_ids.is_empty(),
        "wave2_filtered",
        format!("hits={}", wave2_ids.len()),
    );

    let pool = postgres_test_config::contract_pg_pool(&config).await;

    // Require vectorscale for DiskANN arms.
    if let Err(e) = sqlx::query("CREATE EXTENSION IF NOT EXISTS vectorscale CASCADE")
        .execute(&pool)
        .await
    {
        emit(
            "fdl078_cell",
            0.0,
            false,
            "vectorscale",
            format!("missing vectorscale ({e}); DiskANN arms skipped"),
        );
        eprintln!("SKIP SPEC-078 DiskANN arms: vectorscale unavailable ({e})");
        emit(
            "fdl078_decision",
            0.0,
            true,
            "honesty",
            "Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; \
             promote only after full gate (not this smoke)",
        );
        return;
    }

    let mut map = WorkspaceLabelMap::new();
    let label_hot = map.label_for(HOT_WS).expect("hot label");
    let label_cold = map.label_for(COLD_WS).expect("cold label");

    let table = format!("eq_fdl078_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    sqlx::query(&format!(
        "CREATE TABLE {table} (
            id TEXT PRIMARY KEY,
            embedding vector({DIM}) NOT NULL,
            workspace_id TEXT NOT NULL,
            labels smallint[] NOT NULL
        )"
    ))
    .execute(&pool)
    .await
    .expect("create study table");

    let study_seed_ms = seed_study_table(&pool, &table, rows as usize, label_hot, label_cold).await;
    emit(
        "fdl078_seed",
        study_seed_ms,
        true,
        "study_vector_labels",
        format!("rows={rows} labels hot={label_hot} cold={label_cold}"),
    );

    // Arm 2: embedding-only DiskANN + TEXT post-filter
    let emb_idx = format!("{table}_emb_idx");
    if let Err(e) = sqlx::query(&build_diskann_embedding_only_index_sql(&table, &emb_idx))
        .execute(&pool)
        .await
    {
        eprintln!("SKIP SPEC-078: emb-only DiskANN index failed ({e})");
        return;
    }

    let q_lit = emb_literal(&q);
    let post_sql = build_postfilter_diskann_select_sql(&table, 2, 3);
    let _ = apply_diskann_recipe(&pool).await;
    let t1 = Instant::now();
    let post_rows = {
        let mut tx = pool.begin().await.expect("tx");
        for stmt in diskann_optin_recipe_statements() {
            sqlx::query(&stmt).execute(&mut *tx).await.ok();
        }
        sqlx::query("SET LOCAL enable_seqscan = off")
            .execute(&mut *tx)
            .await
            .ok();
        let rows = sqlx::query(&post_sql)
            .bind(&q_lit)
            .bind(HOT_WS)
            .bind(TOP_K as i32)
            .fetch_all(&mut *tx)
            .await;
        tx.commit().await.ok();
        rows
    };
    let post_ms = t1.elapsed().as_secs_f64() * 1000.0;
    assert!(post_ms < hang, "postfilter hang cliff {post_ms} >= {hang}");
    let post_ids: Vec<String> = match post_rows {
        Ok(rs) => rs.iter().map(|r| r.get::<String, _>("id")).collect(),
        Err(e) => {
            emit(
                "fdl078_cell",
                post_ms,
                false,
                "postfilter_diskann",
                format!("query failed: {e}"),
            );
            Vec::new()
        }
    };
    let post_recall = recall_at_k(&wave2_ids, &post_ids);
    emit(
        "fdl078_filtered_recall",
        post_recall * 1000.0,
        true,
        "postfilter_vs_wave2",
        format!(
            "FILTERED recall@20 postfilter_vs_wave2={post_recall:.4} \
             (honesty baseline) post_ms={post_ms:.1} hits={}",
            post_ids.len()
        ),
    );
    emit(
        "fdl078_cell",
        post_ms,
        true,
        "postfilter_diskann",
        format!(
            "hits={} recall_vs_wave2={post_recall:.4} (post-filter cliff archive)",
            post_ids.len()
        ),
    );

    // Arm 3: DiskANN + labels (drop emb-only index so planner picks labels index)
    let _ = sqlx::query(&format!("DROP INDEX IF EXISTS {emb_idx}"))
        .execute(&pool)
        .await;
    let lab_idx = format!("{table}_labels_idx");
    if let Err(e) = sqlx::query(&build_diskann_labels_index_sql(&table, &lab_idx))
        .execute(&pool)
        .await
    {
        eprintln!("SKIP SPEC-078: labels DiskANN index failed ({e})");
        return;
    }
    let _ = sqlx::query(&format!("ANALYZE {table}")).execute(&pool).await;

    let lab_sql = build_filtered_diskann_label_select_sql(&table, 2, 3);
    let t2 = Instant::now();
    let lab_rows = {
        let mut tx = pool.begin().await.expect("tx");
        for stmt in diskann_optin_recipe_statements() {
            sqlx::query(&stmt).execute(&mut *tx).await.ok();
        }
        sqlx::query("SET LOCAL enable_seqscan = off")
            .execute(&mut *tx)
            .await
            .ok();
        let rows = sqlx::query(&lab_sql)
            .bind(&q_lit)
            .bind(label_hot)
            .bind(TOP_K as i32)
            .fetch_all(&mut *tx)
            .await;
        tx.commit().await.ok();
        rows
    };
    let lab_ms = t2.elapsed().as_secs_f64() * 1000.0;
    assert!(lab_ms < hang, "labels hang cliff {lab_ms} >= {hang}");

    let lab_ids: Vec<String> = match lab_rows {
        Ok(rs) => rs.iter().map(|r| r.get::<String, _>("id")).collect(),
        Err(e) => {
            emit(
                "fdl078_cell",
                lab_ms,
                false,
                "filtered_diskann_labels",
                format!("query failed: {e}"),
            );
            eprintln!("WARN SPEC-078: labels query failed ({e})");
            return;
        }
    };

    let recall = recall_at_k(&wave2_ids, &lab_ids);
    let pass = recall >= RECALL_SOFT && !lab_ids.is_empty();
    emit(
        "fdl078_filtered_recall",
        recall * 1000.0,
        pass,
        "labels_vs_wave2",
        format!(
            "FILTERED recall@20 labels_vs_wave2={recall:.4} soft={RECALL_SOFT} \
             wave2_ms={wave2_ms:.1} labels_ms={lab_ms:.1} rows={rows}"
        ),
    );
    emit(
        "fdl078_cell",
        lab_ms,
        pass,
        "filtered_diskann_labels",
        format!(
            "wave2_hits={} labels_hits={} recall={recall:.4} \
             (soft-fail; Wave-2 remains default; no silent flip)",
            wave2_ids.len(),
            lab_ids.len()
        ),
    );
    emit(
        "fdl078_decision",
        0.0,
        true,
        "honesty",
        "Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; \
         EDGEQUAKE_FILTERED_DISKANN_LABELS default OFF; promote only after full gate",
    );

    let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {table}"))
        .execute(&pool)
        .await;

    if pass {
        println!("GREEN SPEC-078: filtered recall labels_vs_wave2={recall:.4}");
    } else {
        println!("WARN SPEC-078: filtered recall={recall:.4} (soft; archive only)");
    }
}
