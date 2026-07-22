# SPEC-063 — Operating envelope

**Claim SSOT for operators:** [`docs/product-limits.md`](../../docs/product-limits.md) (SPEC-065/066). This envelope is the capacity-pack detail view. Ceiling ladder: [`../066-ceiling-proof/e2e/artifacts/RUN_NOTES.md`](../066-ceiling-proof/e2e/artifacts/RUN_NOTES.md) (`highest_green_N=100k`, `first_fail_N=250k`).

Docs estimates assume ~50–500 chunks/document. Exact: \(N_{\text{docs}} = N_{\text{chunks}} / \text{chunks\_per\_doc}\).

| Envelope | Docs (est.) | Chunks / vectors | Nodes | Edges | Disk / RAM (order) | Status |
|----------|-------------|------------------|-------|-------|--------------------|--------|
| **Proven** | tens–low hundreds | **50k @1536** (SPEC-061/062 prod matrix) | ≤50k community | ~5k–20k expand stress | &lt;10 GB corpus; laptop-class OK | **Measured** 2026-07-18 |
| **L1 Q1-d (battle)** | — | **100k @1536** single p95 **&lt;500ms** with **halfvec + workspace partial HNSW** (opt-in); warm full path also &lt;500ms | — | — | Keep filtered working set hot | **SPEC-064** [`../064-filtered-ann-scale-battle/e2e/artifacts/RUN_NOTES.md`](../064-filtered-ann-scale-battle/e2e/artifacts/RUN_NOTES.md) |
| **L1 cold cliff** | — | Same N: **btree filter → exact distance on ~20% rows**; cold p95 ~**1.5s** | — | — | shared_buffers / host RAM | Still real without residency or Wave2 shape ([`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md)) |
| **Supported (design)** | ~1k–10k | **100k** at Q1-d when Wave2 shape (or proven warm residency); **L2/500k not promoted** (SPEC-066 recall/concurrent cliffs) | ≤50k auto Louvain; **G1 100k** nodes+degrees proven (SPEC-066) | proportional | tens of GB disk; RAM ≥ working set (`shared_buffers` ≥2–4GB) | Prefer `halfvec` greenfield + `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1` for hot WS |
| **Aspirational** | 100k+ | ~1M+ entities / chunks | 1M+ | — | 100s GB + large RAM or DiskANN | **Unproven** — do not state as fact |

## Binding constraints by axis

| Axis              | Binding limit today                                                      |
| -------------------| --------------------------------------------------------------------------|
| Documents         | Upload **50 MiB**/file; `max_documents` fail-closed when set; no storage-GB quota |
| Pages             | UX/timeouts (100 / 500 page thresholds), not a hard store cap            |
| Vectors / ANN     | Proven **50k@1536**; supported **100k** with Wave-2 + residency; beyond not promoted (SPEC-066). Physics-only M-scale figures are **not** product floors |
| Nodes (community) | **50k** hard API / soft ingest                                           |
| Nodes (store)     | No hard max; migrations defer helpers at **500k** vertices               |
| Edges             | No hard max; request-path expand bounded                                 |
| Concurrent users  | Pool **32** + arm concurrency **4** — not a 100-user proof               |

## What operators should promise

1. **Proven:** latency SLOs at ≤50k @1536 under `make data-access-perf-matrix-prod`.
2. **Supported:** size RAM with [`001-first-principles.md`](001-first-principles.md); keep community ≤50k or raise env deliberately; prefer halfvec for new workspaces after SPEC-059 recall gate.
3. **Aspirational:** only after [`004-proof-ladder.md`](004-proof-ladder.md) L1 (100k) / L2 (500k) / L3 (1M) artifacts are green.
