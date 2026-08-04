# 04 — Execution Protocol (Dual-SUT Arms)

> Goal: measure **geometry (N)** and separate **M vs U** under controlled pins.  
> Reuse Acc tooling; do not change Acc publication pins.

## Arms

| Arm | SUT | Pins |
|-----|-----|------|
| **A** | EdgeQuake fair | `EDGEQUAKE_ADAPTIVE_CHUNKING=0`, `EDGEQUAKE_CHUNK_SIZE=1200`, `EDGEQUAKE_CHUNK_OVERLAP=100`, gleaning=1, caps 40/100 |
| **B** | EdgeQuake product | Adaptive **ON** (default), same gleaning/caps |
| **C** | LightRAG | `CHUNK_SIZE=1200`, `CHUNK_OVERLAP_SIZE=100`, gleaning=1, caps 40/100, strategy **R** (or Pdf when comparing PDF geometry) |

Same sample bytes across arms. Same LLM+embed when claiming extract density; geometry-only runs may use mock and must be labeled **geometry-only**.

## Samples

| ID | Path | Notes |
|----|------|-------|
| S1 | `zz_test_docs/academic_papers/lighrag_2410.05779v3.pdf` | Primary narrative (~1.1 MB → adaptive **600**) |
| S1-md | `zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md` | Text twin (~61 KB → adaptive **800**) |
| S2 | One GraphRAG-Bench medical doc from Acc freeze | Secondary when `~/.cache/edgequake/bench001/graphrag-bench/` present |

## Metrics schema (always separate columns)

| Column | Meaning |
|--------|---------|
| `arm` | A / B / C |
| `sample_id` | S1 / S1-md / S2 |
| `bytes` / `chars` | Input size |
| `chunk_size_pin` | Effective target token size |
| `chunk_count` N | Chunks produced |
| `doc_entity_count` M | EQ ProcessingStats / UI (N/A for LR unless instrumented) |
| `doc_rel_count` | EQ relationship mentions |
| `graph_unique_nodes` U | AGE Node count with source prefix / LR entity KV |
| `graph_unique_edges` | AGE edges / LR relation KV |
| `mentions_per_1k_chars` | `1000 * M / chars` |
| `nodes_per_1k_chars` | `1000 * U / chars` |
| `mode` | `geometry-only` \| `mock-extract` \| `live-llm` |

## Geometry-only probe (fast, no LLM)

Run from repo root:

```bash
python3 specs/108-extraction-compared-light-rag/measurements/geometry_probe.py
```

Writes `measurements/geometry_table.md` + `geometry_results.json`.

Implements EQ adaptive thresholds + recursive word/CJK length heuristic aligned with `recursive_token_len` (sufficient for N ratios; not byte-identical to production tiktoken embed path).

## Live / Acc dual-SUT (when services + keys available)

```bash
# EQ fair arm backend (Acc-style)
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
# start backend → ingest S1 → record document entity_count + AGE unique nodes

# EQ product arm: unset adaptive or set =1; new workspace; same file

# LightRAG arm
export CHUNK_SIZE=1200 CHUNK_OVERLAP_SIZE=100
# via tools/bench001/bench001/lightrag_runner.py or Acc stage

# Optional name Jaccard on warm workspaces
python3 tools/bench001/scripts/audit_eq_lr_ingest.py
```

Artifacts land under `measurements/<utc>/` and are summarized in [05-execution-report.md](05-execution-report.md).

## Pass / interpret rules

| Observation | Supports |
|-------------|---------|
| Same ingest: M ≫ U | **H1** |
| Arm B N ≈ 1.5–2× Arm A; M tracks N | **H2** |
| Fair A vs C: U close, Jaccard high | No H4/H5 |
| Fair A vs C: U_EQ ≫ U_LR after matched N | **H4/H5** |

## Safety

- Never overwrite Acc warm publish workspace.
- New workspace IDs per arm.
- Label mock runs so partner reply does not claim live-LLM density.
