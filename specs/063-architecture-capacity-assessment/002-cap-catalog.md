# SPEC-063 — Cap catalog (code is law)

Sources: [`budget.rs`](../../edgequake/crates/edgequake-core/src/resource/budget.rs), [`capabilities.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/capabilities.rs), [`community_persist.rs`](../../edgequake/crates/edgequake-storage/src/community_persist.rs).

## Enforced (hard)

| Cap | Default | Env / note |
|-----|---------|------------|
| Upload / body | **50 MiB** | `EDGEQUAKE_MAX_UPLOAD_BYTES` (min 1 MiB) |
| Graph nodes / API response | **500** | `MAX_GRAPH_NODES` |
| Graph depth | **5** | `MAX_GRAPH_DEPTH` |
| List page size | **100** | `MAX_PAGE_SIZE` |
| Query string | **10_000** chars | `MAX_QUERY_CHARS` |
| Community / full-graph scan admission | **50_000** nodes | `EDGEQUAKE_GRAPH_SCAN_THRESHOLD` (min 1k); API hard reject |
| HNSW dim `vector` | **≤2000** | else no HNSW |
| HNSW dim `halfvec` | **≤4000** | else no HNSW |
| Graph materialize concurrent | **4** (pool-aware ≤8) | `EDGEQUAKE_GRAPH_MATERIALIZE_CONCURRENT` |
| PDF vision jobs concurrent | **2** (1–8) | `EDGEQUAKE_PDF_VISION_JOBS` |
| Tenant `max_workspaces` | plan defaults | Enforced in quota ops |

## Soft-skip / soft-gate

| Cap | Default | Behavior |
|-----|---------|----------|
| Community auto (ingest/backfill) | **50_000** | `EDGEQUAKE_COMMUNITY_MAX_NODES`; skip + warn above |
| Mem headroom ratio | **0.75** | Soft / ops (not hard OOM kill of uploads) |

## Declared but not enforced as corpus quota

| Field | Status |
|-------|--------|
| `Workspace.max_documents` | Declared; **fail-closed** at upload / new PDF mint when set (SPEC-066/067; committed + staging) |
| Workspace / tenant **storage GB** | **None** — no `storage_limit_bytes` |
| FAQ 100k docs / 1M entities | Aspirational until proof ladder green |

## Batch / ops knobs (throughput)

| Knob | Default | Clamp |
|------|---------|-------|
| `EDGEQUAKE_VECTOR_UPSERT_CHUNK` | 1000 | 100…10_000 |
| `EDGEQUAKE_GRAPH_UPSERT_CHUNK` | 500 | 50…2000 |
| `EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS` | 16 | 1…256 |
| `DATABASE_POOL_SIZE` | 32 | — |
| Neighbor expand `LIMIT` | 500 | depth 1…3 |
