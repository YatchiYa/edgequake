# 10 — Postgres / pgvector / AGE Performance (PG16 · PG17 · PG18)

**Date:** 2026-07-10  
**Lens:** Database Expert (primary) · Systems Engineer  
**Law:** Pins and DDL in repo beat marketing slides.

---

## First Principles (Storage Physics)

### S1 — One engine, three access paths

```text
KV (documents/chunks)     → B-tree / JSONB
Vectors (chunk/entity/rel)→ HNSW / halfvec / FTS GIN
Graph (entities/edges)    → AGE vertex/edge heaps + Cypher
```

Cross-path joins (agtype ↔ jsonb) cost CPU. Prefer **batch SQL** over per-node Cypher round-trips.

### S2 — ANN recall is a GUC, not a constant

```text
recall ≈ f(ef_search, iterative_scan, filter selectivity, index quality)
latency ≈ g(ef_search, shared_buffers residency, m, dimension)
```

Default `hnsw.ef_search=40` is often too low for production RAG (AWS / pgvector docs 2025–2026).

### S3 — AGE does not index for you

Microsoft Learn (Jan 2026): **Apache AGE creates no indexes on new graphs by default.**  
Without BTREE on `id`/`start_id`/`end_id` and GIN on `properties`, traversals degrade to seq scans → **O(N)**.

### S4 — O(N) is forbidden on the hot path

Any ingest or query path that calls `MATCH (n:Node) RETURN n` without workspace filter + LIMIT is a **scalability bug**, not a feature.

---

## Official Support Matrix (Code Pins)

Source: `edgequake/docker/extension-pins.sh` (SSOT)

| Profile | Postgres | pgvector | AGE | Role |
|---------|----------|----------|-----|------|
| **pg18** (recommended) | 18 | **0.8.3** | **1.7.0** (`PG18/v1.7.0-rc0`) | New installs |
| **pg17** | 17 | **0.8.3** | **1.7.0** (`PG17/v1.7.0-rc0`) | Managed PG17 |
| **pg16** (legacy) | 16 | **0.8.3** | **1.6.0** (`PG16/v1.6.0-rc0`) | Existing deployments |

**Capability gates in code** (`capabilities.rs`):

| Capability | Gate |
|------------|------|
| `iterative_scan` | pgvector ≥ 0.8.0 |
| AGE RLS / COPY loader | AGE ≥ 1.7.0 |
| `uuidv7` | PG ≥ 18 |

**Implication:** PG16 lacks AGE RLS. Feature-flag paths must not assume 1.7 APIs on pg16.

---

## What EdgeQuake Already Does Well

### pgvector

| Knob | Value | Path |
|------|-------|------|
| Index type | HNSW default | `PostgresConfig::default` |
| `m` | 16 | `hnsw_m` |
| `ef_construction` | **32** (SPEC-034: ~35% smaller index, <2% recall loss vs 64) | `config.rs:96` |
| Runtime `ef_search` | `clamp(k*4, 40, 1000)` via `SET LOCAL` | `search_tuning.rs` |
| Filtered iterative scan | `strict_order` + `max_scan_tuples=20000` when ≥0.8 | `search_tuning.rs` |
| halfvec | dims (2000,4000] → halfvec column/index | `AnnIndexPolicy`, M071/M080 |
| FTS | `content_tsv` + GIN | `ensure_content_fts` |
| O(1) counts | trigger stats table | `row_count_stats.rs` |

Aligned with official guidance:

- [pgvector 0.8.0](https://www.postgresql.org/about/news/pgvector-080-released-2952/) — iterative scans for filtered ANN
- AWS Aurora pgvector production notes — raise `ef_search`; size `maintenance_work_mem` for builds
- halfvec for memory (≈50% footprint)

### AGE

| Practice | EdgeQuake | Official |
|----------|-----------|----------|
| BTREE on vertex `id` | ✅ `ensure_indexes` | Azure AGE guide |
| BTREE on edge `start_id`/`end_id` | ✅ | Azure AGE guide |
| GIN on `properties` | ✅ | Azure AGE guide |
| Expr indexes tenant/workspace | ✅ | Multi-tenant RAG |
| CONCURRENT bootstrap >10k rows | ✅ | Avoid lock storms |
| INVALID index repair | ✅ | Ops hygiene |
| Prefer batch edge fetch over Cypher `[*..N]` | ✅ query expand | Avoid variable-length path tax |

---

## Performance Gaps & Code Smells

### P0 — Unbounded full-graph load

```text
community.rs:158-159
  nodes = graph.get_all_nodes()
  edges = graph.get_all_edges()
→ nodes_ops.rs:394  MATCH (n:Node) RETURN n   # no LIMIT
```

**Impact:** Community refresh after ingest is **O(V+E)** memory + I/O. Breaks large workspaces.

**Fix physics:** Workspace-scoped sample or incremental Louvain on dirty subgraph; hard cap (e.g. 50k nodes) with metric + skip.

### P0 — Silent HNSW DDL failure

```63:63:edgequake/crates/edgequake-storage/src/adapters/postgres/vector/ddl.rs
            sqlx::query(&index_sql).execute(&pool).await.ok();
```

**Impact:** Table exists, ANN index missing → seq scan. Latency/recall cliff with **no readiness failure**.

**Fix:** Propagate error OR record `ann_index_ready=false` and block `/ready`.

### P1 — `ef_construction=32` vs industry default 64

SPEC-034 traded size for recall. Valid **if** measured. Re-validate on GraphRAG-Bench subset:

```text
Target: recall@k ≥ baseline(ef_construction=64) − 2%
If miss → raise to 64 for production profiles; keep 32 for dev.
```

### P1 — Filtered scan uses `strict_order`

pgvector docs: `relaxed_order` often better recall/latency for RAG filters.  
EdgeQuake uses `strict_order` for HNSW filtered queries.

**Action:** A/B `relaxed_order` as default for workspace-filtered chunk search; keep strict for eval harness.

### P1 — AGE RLS only on PG17/18 + opt-in

PG16 tenants rely on application filters. Defense-in-depth incomplete on legacy profile.

### P2 — agtype ↔ jsonb cast tax

Known industry pain (unified Postgres Graph-RAG writeups 2026). Prefer property projection in Cypher RETURN and typed Rust parse; avoid double cast in hot loops.

### P2 — Pool sizing

Default `max_connections: 32`. Mix runs 3 arms → multiple DB holders. Document:

```text
PG max_connections ≥ Σ(app pools) + admin + autovacuum headroom
```

---

## Recommended Postgres Runtime Settings (Ops Runbook)

Grounded in AWS/Azure/pgvector production notes — **not** cargo-cult; tune per host.

| Setting | Starting point | Why |
|---------|----------------|-----|
| `shared_buffers` | ~25% RAM | Keep HNSW pages resident |
| `maintenance_work_mem` | ≥ 2GB on index build hosts | Avoid HNSW build thrash |
| `work_mem` | sized for sort/hash of FTS+ANN plans | Per-query |
| `hnsw.ef_search` | app sets LOCAL (already) | Don't rely on global 40 |
| `hnsw.iterative_scan` | `relaxed_order` for filtered RAG (proposed) | Completeness under filters |
| Checksums | ON (PG18 default for new clusters) | Corruption detection |
| Autovacuum | Aggressive on high-churn vector tables | Bloat kills ANN |

---

## Query-Path Complexity Contract

| Path | Allowed complexity | Enforcement |
|------|-------------------|-------------|
| Vector ANN | O(log N) | Require HNSW; readiness check |
| FTS BM25 | O(log N) inverted | GIN `content_tsv` |
| Local/Global expand | O(k · degree · hops) batched | `get_incident_edges_batch` |
| Mix | 3 × above (parallel) | Router should skip arms on L1 |
| Community | O(sample) or incremental | **Must not** full scan |
| Admin reconcile | O(N) OK offline | Never on request path |

---

## Major-Version Migration Notes

| Transition | Risk | Mitigation in-repo / required |
|------------|------|-------------------------------|
| PG16 → PG17/18 | AGE 1.6 → 1.7 upgrade script slow on large graphs | `scripts/migrate_postgres_major.sh`; schedule maintenance window |
| AGE 1.7 first install on PG18 | No prior upgrade path (upstream note) | Fresh graph create OK; dump/restore for data move |
| pgvector <0.8 → 0.8.3 | iterative_scan becomes available | M042 readiness blocker |
| halfvec conversion | Rewrite embeddings column | M080 reconcile; verify ANN opclass |
| sqlx checksum drift | Boot fail | M071/M078 repair hooks |

**Rollback law:** sqlx is **forward-only**. True rollback = PITR / `pg_basebackup` restore, not `migrate down`.

---

## Benchmark Plan (must run before claiming perf)

```text
1. Synthetic: 100k / 1M chunk vectors — p50/p95 ANN @ k=10,50 with/without workspace filter
2. Graph: 100k nodes / 500k edges — 2-hop expand p95; community refresh wall time
3. Mix query: L1 vs L2 arm cost with adaptive router on/off
4. Matrix: pg16 / pg17 / pg18 images from extension-pins.sh
```

Artifacts: pin smoke via `make ops17-smoke` / `e2e/run_ops17_perf_smoke.sh` (DONE).  
Large-scale ANN/graph wall-time benches (`eval/postgres-perf/`) remain **optional post-0.16** measurement work — not a blocker for fail-closed correctness.

---

## Verdict (v0.16.0 code-is-law)

EdgeQuake's **pgvector + AGE configuration is ahead of most OSS GraphRAG stacks** (iterative_scan, halfvec policy, AGE indexes, triple PG track).

**Shipped in v0.16.0:**
- ✅ Community ingest path is **O(sample)-safe** (`load_graph_bounded`)
- ✅ ANN readiness is **fail-closed** (`missing_hnsw_index` on `/ready`)
- ✅ Multi-major pin smoke (`make ops17-smoke` + nightly workflow)

**Still open (measurement, not correctness):** 100k/1M ANN + graph wall-time artifact suite.

→ Implementation: [12-IMPLEMENTATION-PLAN-OPS.md](./12-IMPLEMENTATION-PLAN-OPS.md) (all OPS tickets DONE)
