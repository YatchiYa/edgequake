//! SPEC-046 GraphRAG-Bench subset Criterion harness.
//!
//! Measures Hybrid RAG retrieval physics (no Postgres / no live LLM):
//! - L1–L4 adaptive routing table (GraphRAG-Bench levels)
//! - Path prune throughput
//! - PPR neighborhood expand
//! - Dynamic truncation remainder
//!
//! Run:
//! ```bash
//! cargo bench --bench graphrag_bench
//! # quick smoke (CI-friendly):
//! cargo bench --bench graphrag_bench -- --quick
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tokio::runtime::Runtime;

use edgequake_query::eval::{assert_case_routing, spec046_synthetic_bench};
use edgequake_query::graph_expand::expand_neighborhood_edges;
use edgequake_query::graph_ppr::GraphWalkMode;
use edgequake_query::path_prune::{prune_relationships, PathPruneConfig};
use edgequake_query::tokenizer::MockTokenizer;
use edgequake_query::truncation::{balance_context, TruncationConfig};
use edgequake_query::{
    context::{RetrievedChunk, RetrievedEntity, RetrievedRelationship},
    keywords::QueryIntent,
};
use edgequake_storage::adapters::memory::MemoryGraphStorage;
use edgequake_storage::traits::{GraphReadView, GraphStorage, GraphStorageMutateOps};

fn create_runtime() -> Runtime {
    Runtime::new().expect("tokio runtime")
}

fn bench_routing_table(c: &mut Criterion) {
    let cases = spec046_synthetic_bench();
    let mut group = c.benchmark_group("graphrag_routing");
    group.throughput(Throughput::Elements(cases.len() as u64));
    group.bench_function("synthetic_l1_l4_table", |b| {
        b.iter(|| {
            for case in &cases {
                assert_case_routing(black_box(case));
            }
        })
    });
    for case in &cases {
        group.bench_with_input(BenchmarkId::new("classify", case.id), case.query, |b, q| {
            b.iter(|| {
                let intent = QueryIntent::classify_heuristic(black_box(q));
                black_box(intent.recommended_mode())
            })
        });
    }
    group.finish();
}

fn bench_path_prune(c: &mut Criterion) {
    let mut group = c.benchmark_group("graphrag_path_prune");
    let cfg = PathPruneConfig {
        drop_fraction: 0.4,
        min_keep: 3,
        min_input: 5,
        ..Default::default()
    };
    for n in [20usize, 60, 200] {
        let rels: Vec<_> = (0..n)
            .map(|i| {
                RetrievedRelationship::new("A", format!("T{i}"), "REL")
                    .with_score(i as f32 * 0.01)
                    .with_description(if i % 3 == 0 { "rich evidence path" } else { "" })
            })
            .collect();
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &rels, |b, rels| {
            b.iter(|| prune_relationships(black_box(rels.clone()), black_box(&cfg)))
        });
    }
    group.finish();
}

fn bench_ppr_expand(c: &mut Criterion) {
    let rt = create_runtime();
    let graph = rt.block_on(async {
        let g = Arc::new(MemoryGraphStorage::new("graphrag-ppr-bench"));
        g.initialize().await.unwrap();
        // Small dense neighborhood around SEED
        for i in 0..32 {
            let n = format!("N{i}");
            g.upsert_edge("SEED", &n, HashMap::new()).await.unwrap();
            if i + 1 < 32 {
                let n2 = format!("N{}", i + 1);
                g.upsert_edge(&n, &n2, HashMap::new()).await.unwrap();
            }
        }
        g
    });

    let mut group = c.benchmark_group("graphrag_ppr_expand");
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("ppr_depth2_top10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let view = GraphReadView::new(graph.as_ref());
                expand_neighborhood_edges(
                    &view,
                    black_box(&["SEED".into()]),
                    2,
                    10,
                    GraphWalkMode::Ppr,
                    None,
                    None,
                )
                .await
                .unwrap()
            })
        })
    });
    group.bench_function("bfs_depth2_top10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let view = GraphReadView::new(graph.as_ref());
                expand_neighborhood_edges(
                    &view,
                    black_box(&["SEED".into()]),
                    2,
                    10,
                    GraphWalkMode::Bfs,
                    None,
                    None,
                )
                .await
                .unwrap()
            })
        })
    });
    group.finish();
}

fn bench_dynamic_truncation(c: &mut Criterion) {
    let tokenizer = MockTokenizer::with_rate(1.0);
    let config = TruncationConfig {
        max_entity_tokens: 2000,
        max_relation_tokens: 2000,
        max_total_tokens: 8000,
        buffer_tokens: 200,
        min_chunk_budget_ratio: 0.40,
    };
    let entities: Vec<_> = (0..40)
        .map(|i| RetrievedEntity::new(format!("E{i}"), "T", "entity description text"))
        .collect();
    let rels: Vec<_> = (0..40)
        .map(|i| {
            RetrievedRelationship::new("A", format!("B{i}"), "REL")
                .with_description("relation evidence")
        })
        .collect();
    let chunks: Vec<_> = (0..40)
        .map(|i| {
            RetrievedChunk::new(
                format!("c{i}"),
                "chunk body ".repeat(40),
                1.0 - (i as f32 * 0.01),
            )
        })
        .collect();

    let mut group = c.benchmark_group("graphrag_truncation");
    group.bench_function("balance_context_40x40x40", |b| {
        b.iter(|| {
            balance_context(
                black_box(entities.clone()),
                black_box(rels.clone()),
                black_box(chunks.clone()),
                black_box(&config),
                black_box(&tokenizer),
            )
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_routing_table,
    bench_path_prune,
    bench_ppr_expand,
    bench_dynamic_truncation
);
criterion_main!(benches);
