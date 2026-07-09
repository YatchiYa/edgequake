# EdgeQuake Benchmarks

This directory contains performance benchmarks for EdgeQuake components.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench chunking_bench
cargo bench --bench storage_bench
cargo bench --bench graph_performance
cargo bench --bench graphrag_bench

# CI-friendly smoke (fewer samples)
cargo bench --bench graphrag_bench -- --quick
```

## Benchmarks

### Chunking Benchmark (`chunking_bench.rs`)

Measures text chunking performance:
- Small text (< 1KB)
- Medium text (~10KB)
- Large text (~100KB)

### GraphRAG Benchmark (`graphrag_bench.rs`) — SPEC-046

GraphRAG-Bench-style Hybrid RAG physics (in-memory, no Postgres/LLM):
- L1–L4 adaptive routing table (`spec046_synthetic_bench`)
- Path prune throughput
- PPR vs BFS neighborhood expand
- Dynamic truncation remainder

Also covered by e2e: `cargo test -p edgequake-query --test e2e_spec046_hybrid_rag`

### Storage Benchmark (`storage_bench.rs`)

Measures storage operations:
- Vector upsert and query
- Graph node and edge operations
- KV storage operations

### Graph Performance (`graph_performance.rs`)

Graph scan / neighborhood microbenchmarks.

## Interpreting Results

Criterion outputs statistics including:
- Mean execution time
- Standard deviation
- Throughput (for iterative benchmarks)

Results are saved in `target/criterion/` with HTML reports.

## Environment

Benchmarks use in-memory storage to measure pure algorithm performance.
For production-like benchmarks, configure external storage backends.
