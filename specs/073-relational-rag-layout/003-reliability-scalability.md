# SPEC-073 — Reliability and scalability mechanisms

Mechanisms, not slogans. Each claim ties to a first principle ([`001`](001-first-principles.md)) and EdgeQuake evidence ([`002`](002-edgequake-mapping.md), SPEC-063–072).

## Reliability

### 1. Cascade / retract integrity

| Mechanism | Ideal | EdgeQuake |
|-----------|-------|-----------|
| Document delete removes chunks | `ON DELETE CASCADE` on `chunks.document_id` | Relational rows cascade; **RAG** requires saga retract of KV + vectors + AGE (SPEC-058/059) |
| Orphan prevention | Single transaction | Best-effort saga + orphan janitor (`EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER`) |

**Reliability tax:** dual-SSOT improves ANN/FTS layout but **weakens** single-transaction delete. Incomplete retract = ghost embeddings / graph nodes. Treat retract completeness as a first-class reliability surface (checklist in [`004`](004-recommendations.md)).

### 2. Isolation by construction

- `workspace_id` on documents **and** denormalized on vector rows enables:
  - App filters that match index predicates
  - RLS on relational tables (tenant context)
  - Fail-closed isolation if app omits a filter (RLS) — vectors still need explicit workspace predicates + Wave-2 columns-only policy
- Dedicated `*_ws_*` tables = physical isolation for dimension / DiskANN recipes

### 3. Queryable truth

Relational sidecar holds status, content_hash, PDF linkage, lineage (`chunk_entity_links`). Operators debug ingest without a second product database. CQRS `entities`/`relationships` are read models — not ANN SSOT.

### 4. Filter–index implication (plan reliability)

Reliability includes **correct EXPLAIN**, not only correct rows:

| Failure | Symptom | Cause | Evidence |
|---------|---------|-------|----------|
| Cold cliff | ~1.5 s @100k cold; warm ~50–70 ms | btree `(tenant,ws)` → exact distance on ~20% slice | SPEC-063 / SPEC-064 |
| Planner skip | Concurrent p95 multi-hundred ms / Sort | Partial HNSW not chosen | SPEC-067 session bias |
| JSONB-only filter | Partial index not used | `metadata->>'workspace_id'` OR shape | Wave-2 columns-only policy |
| Recall underfill | Fast ANN, few/wrong hits | Global HNSW + selective post-filter | Industry filter trap; DiskANN q_list=100 @150k (SPEC-070/072) |

**Fix class that works in EdgeQuake:** denorm columns + Wave-2 partial HNSW (+ residency) or dedicated DiskANN with tuned `query_search_list_size`.

### 5. Ops unity

One Postgres instance for KV + vectors + AGE + relational:

- One PITR / backup story for embeddings with product data
- No second vector-DB consistency protocol for the default path
- Industry default through ~10M vectors/node; EdgeQuake proven floors are lower and measured ([`docs/product-limits.md`](../../docs/product-limits.md))

## Scalability

**Clear playbook (industry July 2026 + EdgeQuake evidence):** [`005-industry-scale-playbook.md`](005-industry-scale-playbook.md).

### 1. Selectivity physics

Workspace linkage turns multi-tenant search into a **bounded subgraph**:

\[
\text{cost} \propto \text{embedding\_bytes} \times \text{rows\_in\_workspace\_slice} \times (1 + \text{I/O\_miss})
\]

Without workspace-shaped indexes, shared-table growth makes every tenant pay for the global corpus (exact scan or post-filter underfill).

### 2. Industry ladder vs EdgeQuake measured ladder

| Industry step (2026) | What it fixes | EdgeQuake measured |
|----------------------|---------------|--------------------|
| Denorm workspace on embedding rows | Filter/RLS/partition without join | Required (Wave-2 columns) |
| HNSW default | ANN while graph fits RAM | Shared / partial HNSW |
| `halfvec` first | ~2× density | Wave-2 greenfield |
| Partial HNSW / partition | Filter trap (recall/latency) | Wave-2 **100k** supported |
| `iterative_scan` | Underfill on shared HNSW | Future bake-off (not productized) |
| Residency / warmup | Cold I/O cliff | SPEC-067/071 |
| DiskANN (pgvectorscale) | HNSW RAM ceiling + concurrent | Opt-in **150k** @ `q_list≥400` |
| Hybrid FTS + rerank | Lexical gaps / precision | FTS↔KV; rerank product-dependent |
| External vector DB | Extreme QPS / multi-region / huge N | Out of default path |

| EdgeQuake shape | Scale outcome | Spec |
|-----------------|---------------|------|
| Shared HNSW, no Wave-2 | Cold cliff @100k | 063/064 |
| Wave-2 halfvec + partial HNSW | **Supported 100k** filtered ANN | 064/068/071 |
| Dedicated HNSW `*_ws_*` | Single-query OK; **concurrent wall** @clients=16 from 100k | 069 |
| Dedicated DiskANN, `q_list≥400` | **Opt-in 150k** full-gate green | 070/072 |
| Wave-2 above 100k | Mid-scale wall; 250k first_fail | 068 |

**Document filter is secondary:** useful for scoped search and delete-by-doc; **workspace** chooses the ANN index shape.

### 3. Residency and quantization (same order as industry)

1. `halfvec` (Wave-2 greenfield) — industry “turn on first”
2. `shared_buffers` / host class (≥2 GB tip; ≥16 GB host for proven 100k)
3. Partial HNSW on hot workspaces (warmup)
4. Opt-in DiskANN when concurrent + recall gates demand it
5. Only then: iterative_scan bake-off, partitions, external ANN

### 4. When layout is not enough

Beyond measured walls, evaluate only with full gate (single Q1-d ∧ recall@20≥0.99 ∧ concurrent abs @clients=16):

- pgvector `hnsw.iterative_scan` on shared HNSW (future bake-off candidate)
- Declarative partitions by workspace
- Higher DiskANN lists / rebuild params
- External ANN — last resort for EdgeQuake’s product posture

## Synthesis

```text
Reliability  ≈  FK ownership + denorm filters + retract completeness + plan≡index
Scalability  ≈  bytes↓ + workspace-shaped ANN + residency + DiskANN when RAM cliffs
               (industry order; EdgeQuake floors = measured only)
```

Workspace→document→chunk→embedding is the **control plane** that makes both equations hold. Splitting text (KV) from embeddings (vectors) is a performance optimization; it must not erase the control plane denorm columns or the retract story.
