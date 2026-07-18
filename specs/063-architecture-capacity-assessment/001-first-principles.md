# SPEC-063 — First principles cost model

## Law

Capacity claims require: **(1)** a hard cap in code, **or (2)** a physics formula with RAM/disk assumptions, **or (3)** a measured gate under published SLOs. Marketing numbers without one of these are **aspirational**.

## Vector storage (dominant for RAG corpora)

For dimension \(D = 1536\) and `vector` (float32):

| Component | Bytes / row (order) |
|-----------|---------------------|
| Embedding payload | \(D \times 4 = 6144\) |
| Row/TOAST/metadata | ~100–500 |
| **Table** | ≈ **6.2–6.6 KB** |

At \(N\) chunk vectors:

\[
\text{table\_GB} \approx N \times 6.5 \times 10^{-6}
\]

So **1M chunks ≈ 6–7 GB** table alone.

### HNSW index residency

With default `m=16`, HNSW graph size is typically **~0.5–1×** the vector table for healthy recall. Practical rule for EdgeQuake:

\[
\text{RAM\_effective\_GB} \approx 10\text{–}14 \times (N / 10^6) \quad\text{at }D=1536\text{ full vector}
\]

`halfvec` (float16) ≈ **0.5×** payload → roughly **doubles** \(N\) for the same RAM budget. Index still wants to stay hot; when it spills, p95 cliffs (industry + pgvector issue history).

Industry band (vanilla pgvector HNSW, memory-resident): comfortable **~1–5M** @1536; **~5–10M** with halfvec/tuning; beyond that DiskANN/shard — not EdgeQuake’s default path today.

## Documents and pages (not 1:1 with vectors)

| Quantity | Relation |
|----------|----------|
| Pages (PDF) | UX thresholds: large PDF ≥100 pages; gleaning disable ≥500 pages |
| Chunks / page | ~1–3 after extraction (strategy-dependent) |
| Entities / doc | ~5–50 typical; dense papers higher |
| **Documents** | \(N_{\text{docs}} \approx N_{\text{chunks}} / \text{chunks\_per\_doc}\) |

Example: 50k chunks @ ~100 chunks/doc ⇒ **~500 documents** — not 50k docs. FAQ “100k documents” implies **millions** of chunks unless docs are tiny.

## Graph (AGE + native `eq_*`)

| Path | Binding constraint |
|------|--------------------|
| Request-path expand / degrees | Native `eq_source_id` / `eq_target_id` (SPEC-062); measured at 1k–20k edges |
| Community Louvain | **Hard** API gate at **50k** nodes (`graph_scan_threshold`); ingest soft-skips above `EDGEQUAKE_COMMUNITY_MAX_NODES` |
| Graph API response | Clamp **500** nodes / request |
| Forbidden | Unbounded `get_all_nodes` / `get_all_edges` on request path |

Larger graphs can be **stored**; community detection and full materialization are the gates, not AGE’s absolute row limit.

## Workspace GB (honesty)

There is **no** enforced per-workspace storage quota. `storage_bytes` undercounts embeddings/graph (SPEC-021). Size from the formula above; do not trust a UI “X MB of Y MB” unless backend exposes `storage_limit_bytes`.

## Upload / concurrency (throughput, not corpus)

| Cap | Default | Role |
|-----|---------|------|
| Upload body | **50 MiB**/file | Hard |
| PDF vision jobs | 2 process-wide | OOM guard |
| Extractions | 16 concurrent | LLM/GPU |
| DB pool | 32 | Connection budget |

These limit **ingest rate**, not maximum stored corpus.
