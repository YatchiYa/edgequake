# SPEC-060 — First principles

1. **Prove the plan, not the story.** Every OK request-path op has EXPLAIN (Index/HNSW/GIN) or a FORBIDDEN contract.
2. **Complexity ≠ wall time.** Catalog tags asymptotic class; scale ladders (2k → 50k) + p95 prove budgets.
3. **ANN needs recall + latency.** halfvec recall@20 ≥ 0.99; filtered HNSW uses `iterative_scan` + `EXPLAIN (ANALYZE, BUFFERS)`.
4. **Pipeline stages are separate budgets.** KV / vector / AGE / compensate each timed — not one blob.
5. **Soft-skip is not a gate.** Nightly sets `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1`.
6. **Criterion memory benches are informational only** — never a Postgres release gate.
