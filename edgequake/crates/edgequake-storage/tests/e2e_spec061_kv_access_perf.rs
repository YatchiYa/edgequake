//! SPEC-061 — KV DataAccess p95 + Index EXPLAIN.
#![cfg(feature = "postgres")]

#[path = "support/perf_harness.rs"]
mod perf_harness;
#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::KVStorage;
use edgequake_storage::PostgresKVStorage;
use perf_harness::{assert_plan_uses_index, finish_report, join_plan_rows, PlanKind};
use std::time::{Duration, Instant};

const N: usize = 1_000;
const SAMPLES: usize = 10;

#[tokio::test]
async fn e2e_spec061_kv_get_prefix_delete_count() {
    let Some(config) = postgres_test_config::require_or_skip_postgres("perf061_kv") else {
        return;
    };
    let kv = PostgresKVStorage::new(config.clone());
    kv.initialize().await.expect("kv init");

    let batch: Vec<_> = (0..N)
        .map(|i| {
            (
                format!("perf061-prefix-{i}"),
                serde_json::json!({"content": format!("body {i}"), "type": "chunk"}),
            )
        })
        .collect();

    // Upsert wall → p95 of repeated small batches is expensive; measure one full upsert + report.
    let mut upsert_samples = Vec::new();
    for _ in 0..SAMPLES.min(3) {
        let start = Instant::now();
        kv.upsert(&batch).await.expect("upsert");
        upsert_samples.push(start.elapsed());
    }
    finish_report(
        "kv_upsert",
        &upsert_samples,
        100.0,
        "pk_upsert",
        false,
        format!("N={N}"),
    );

    let ids: Vec<String> = (0..N).map(|i| format!("perf061-prefix-{i}")).collect();
    let mut get_samples = Vec::new();
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let got = kv.get_by_ids(&ids).await.expect("get_by_ids");
        get_samples.push(start.elapsed());
        assert_eq!(got.len(), N);
    }
    finish_report(
        "kv_get_by_ids",
        &get_samples,
        50.0,
        "pk_index",
        false,
        format!("N={N}"),
    );

    let mut prefix_samples = Vec::new();
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let keys = kv
            .keys_with_prefix("perf061-prefix-")
            .await
            .expect("prefix");
        prefix_samples.push(start.elapsed());
        assert!(keys.len() >= N);
    }
    finish_report(
        "kv_keys_with_prefix",
        &prefix_samples,
        100.0,
        "text_pattern",
        false,
        format!("N={N}"),
    );

    let start = Instant::now();
    let c = kv.count().await.expect("count");
    let count_elapsed = start.elapsed();
    assert!(c >= N);
    finish_report(
        "kv_count",
        &[count_elapsed],
        20.0,
        "stats",
        false,
        format!("count={c}"),
    );

    // EXPLAIN probes: selective PK ANY (K=2) + prefix with seqscan disabled so the
    // text_pattern index is proven even when LIKE matches the whole 1k-row table.
    let pool = postgres_test_config::contract_pg_pool(&config).await;
    let table = config.qualified_kv_table();
    sqlx::query(&format!("ANALYZE {table}"))
        .execute(&pool)
        .await
        .expect("ANALYZE kv");

    let pk_sql = format!(
        r#"EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
           SELECT key FROM {table} WHERE key = ANY($1::text[])"#
    );
    let selective: Vec<String> = ids.iter().take(2).cloned().collect();
    let plan_rows: Vec<(String,)> = sqlx::query_as(&pk_sql)
        .bind(&selective)
        .fetch_all(&pool)
        .await
        .expect("EXPLAIN kv get_by_ids");
    let plan = join_plan_rows(plan_rows);
    assert_plan_uses_index(&plan, &[PlanKind::Index, PlanKind::Btree, PlanKind::Bitmap]);
    eprintln!("OK SPEC-061 KV get_by_ids EXPLAIN:\n{plan}");

    let mut tx = pool.begin().await.expect("tx");
    sqlx::query("SET LOCAL enable_seqscan = off")
        .execute(&mut *tx)
        .await
        .expect("disable seqscan");
    let prefix_sql = format!(
        r#"EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
           SELECT key FROM {table} WHERE key LIKE $1 LIMIT 100"#
    );
    let prefix_rows: Vec<(String,)> = sqlx::query_as(&prefix_sql)
        .bind("perf061-prefix-%")
        .fetch_all(&mut *tx)
        .await
        .expect("EXPLAIN kv prefix");
    tx.commit().await.ok();
    let prefix_plan = join_plan_rows(prefix_rows);
    assert_plan_uses_index(
        &prefix_plan,
        &[PlanKind::Index, PlanKind::Btree, PlanKind::Bitmap],
    );
    eprintln!("OK SPEC-061 KV prefix EXPLAIN:\n{prefix_plan}");

    let mut del_samples = Vec::new();
    for s in 0..SAMPLES.min(3) {
        let slice: Vec<String> = (0..100)
            .map(|i| format!("perf061-prefix-{}", (s * 100 + i) % N))
            .collect();
        // Re-upsert slice then delete for timing
        let re: Vec<_> = slice
            .iter()
            .map(|id| (id.clone(), serde_json::json!({"content": "x"})))
            .collect();
        kv.upsert(&re).await.ok();
        let start = Instant::now();
        kv.delete(&slice).await.expect("delete");
        del_samples.push(start.elapsed());
    }
    finish_report(
        "kv_delete",
        &del_samples,
        100.0,
        "pk_delete",
        false,
        "batch=100",
    );

    let _ = Duration::from_millis(0);
}
