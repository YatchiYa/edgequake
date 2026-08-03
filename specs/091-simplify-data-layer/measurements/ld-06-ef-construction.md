# LD-06 — `ef_construction` convergence (SPEC-091 IW1)

**Status:** CLOSED (policy) — 2026-07-30  
**Chosen value:** `128`  
**Proof binary:** `e2e_spec091_hnsw_policy_converged`  
**Schema:** migration **129** (`idx_chunk_embeddings_hnsw` + fleet HNSW in **130**)

## Decision

Converge the three historical values (32 in migration 071 historical DDL, 64 in older `docker/init.sql`, 128 runtime `PostgresConfig` default) onto **128**, matching the runtime SSOT that already served production ANN builds.

## Measurement honesty (LAW-I2)

A full ≥100k-vector 32/64/128 recall/size ladder was **not** re-run as a separate artifact. Closure evidence for this train:

1. Runtime + migration + `docker/init.sql` assert a single value (`e2e_spec091_hnsw_policy_converged`).
2. Typed chunk ANN uses model-scoped partial HNSW at `ef_construction = 128` (migration 129).
3. Wave-0 retrieval SLO binary (`e2e_spec091_retrieval_slo_protection`) gates filtered ANN p95 + recall on the typed path under that policy.

Historical migration **071** retains `32` as immutable applied SQL (do not edit). New indexes must not reintroduce 32/64.

## Related

- [`e2e_spec091_hnsw_policy_converged.rs`](../../edgequake/crates/edgequake-storage/tests/e2e_spec091_hnsw_policy_converged.rs)
- [`19-improvement-plan.md`](../19-improvement-plan.md) IW1 / GAP-091-25
