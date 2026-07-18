# SPEC-061 — First principles

1. **Every OK path is measured or FORBIDDEN.** Catalog ops without a proof test are a documentation lie.
2. **Same SLO on every supported major.** A pg16/AGE 1.6 regression blocks release if we claim multi-major support.
3. **Plan shape + wall time + concurrency.** Prove with `EXPLAIN (ANALYZE, BUFFERS)` (Index/HNSW/GIN), p95 at fixed scale, then N concurrent clients. Soft-skip is not a gate.
4. **Artifacts are the product.** JSON per profile (`op`, `p95_ms`, `plan_class`, `pass`) — we cannot improve what we do not measure.
5. **PR stays fast.** Full matrix is nightly / `battle=true` only.

Industry alignment (pgvector 0.8.x): always verify ANN/FTS with `EXPLAIN (ANALYZE, BUFFERS)`; enable `hnsw.iterative_scan` for filtered RAG; measure recall separately (SPEC-059 halfvec gate).
