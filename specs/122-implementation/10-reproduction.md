# 10 — Reproduction

## Hypotheses

| ID | Claim | Accept if | Reject if |
|----|-------|-----------|-----------|
| H1 | Local near-serial tenant lane dominates Arm A | ≤1 overlapping Processing docs under make local | Multi-doc Processing overlap common |
| H2 | PDF vision dominates PDF bulk wall clock | PDF t_all ≫ text for similar token counts; convert stage largest | Extract/embed dominate PDF too |
| H3 | Chunk inflation from PDF markdown | PDF chunks ≫ text peers; extract time scales with chunks | Chunk counts similar; vision still dominates |
| H4 | SPEC-090 DB contention dominates | High store_contention / lock waits; LLM idle | LLM busy; contention low |
| H5 | WebUI×3 transfer is the bottleneck | admit_ms ≈ t_all | admit_ms ≪ t_all |

## Fixtures

| Set | Files | Purpose |
|-----|-------|---------|
| Text | `zz_test_docs/test_injection.txt`, `test_injection.md`, `test-document.md`, `large_test.txt` (+ copies/variants) | Arm A/B text bulk |
| PDF | `zz_test_docs/CCN_Auto_MAJ_Janvier_2026.pdf` and/or `academic_papers/*` small PDFs | H2/H3 |
| Control | Single smallest text file | Arm C |

Prefer small PDFs for first pass; huge manuals only for EC-12.

## Procedure

### Preconditions

```bash
make status
curl -s http://localhost:8080/health   # or auth-aware health
curl -s http://localhost:11434/api/tags
# Mistral: echo ${MISTRAL_API_KEY:+set}
```

Auth: obtain JWT if `DEV_AUTH` requires it; harness uses env `EDGEQUAKE_TOKEN` / magic sign-in.

### Arm C — Control (single doc)

1. Snapshot `GET /api/v1/pipeline/queue-metrics`.
2. Upload 1 text file; record `t_admit`, `t_complete`.
3. Save metrics after complete.

### Arm A — Local Ollama bulk

1. `make stop && make dev-bg` (Ollama profile clamps).
2. Confirm env: `MAX_TASKS_PER_TENANT=1`, extract=1.
3. Upload N∈{3,5,10} text files via WebUI or harness (bounded 3 admits).
4. Record: `t_admit_all`, `t_first_complete`, `t_all_complete`, max concurrent Processing observed, queue-metrics mid-run.
5. Optional: PDF subset for H2/H3.

### Arm B — Mistral bulk

1. Restart with `EDGEQUAKE_LLM_PROVIDER=mistral` + `MISTRAL_API_KEY` and cloud-like or Docker caps (`MAX_TASKS_PER_TENANT≥6`).
2. Same N fixtures as Arm A.
3. Compare docs/min and Processing overlap vs Arm A.

### Metrics to capture

```ascii
  admit_ms_total
  t_first_complete_s
  t_all_complete_s
  docs_per_min = (N / t_all_complete_s) * 60
  max_concurrent_processing
  queue_metrics.pending_depth
  queue_metrics.tenant_park_waiters*
  per_doc: chunks, status, error
```

## Measurement results

Artifacts: [`measurements/20260811-summary.json`](measurements/20260811-summary.json), per-arm dirs under `measurements/`.

### Platform

| Field | Value |
|-------|-------|
| Date | 2026-08-11 |
| EdgeQuake | v0.24.3 (debug binary) |
| Host | local darwin; API `:8090` |
| Arm A provider | Ollama (`gemma4` / `embeddinggemma`) |
| Arm B provider | Mistral (`mistral-small-latest` / `mistral-embed`) |

### Arm C (Ollama, N=1)

| Metric | Value |
|--------|-------|
| File | unique fixture from `test_injection.txt` |
| t_admit_s | 0.056 |
| t_complete_s | 14.245 |
| docs/min | 4.212 |
| tenant | 1 |

### Arm A (Ollama)

| N | t_admit_s | t_first_s | t_all_s | docs/min | max_proc* | park_waiters | tenant |
|---|-----------|-----------|---------|----------|-----------|--------------|--------|
| 5 | 0.159 | 6.311 | 59.237 | 5.064 | 0† | 0 | 1 |

\*Harness polls every 2s — often misses fleeting `processing` display. Completion counts advanced **1→2→3→4→5** (serial).  
†`max_tasks_per_tenant=1` confirmed via queue-metrics.

### Arm B (Mistral)

| N | t_admit_s | t_first_s | t_all_s | docs/min | park_waiters | tenant | workers |
|---|-----------|-----------|---------|----------|--------------|--------|---------|
| 5 | 0.184 | 8.364 | 45.016 | 6.664 | 0 | 6 | 4 |

### PDF sample (H2/H3, Mistral)

| Metric | Value |
|--------|-------|
| File | `zz_test_docs/academic_papers/Qwen.pdf` (1 page, 833 KiB) |
| Convert duration | 11521 ms (`extraction_method=vision`) |
| Markdown len | 3028 |
| Chunks after insert | 5 |
| Text control (short) | ~8.7 s to completed |
| Note | Vision tax real; not serial root for text bulk |

### Hypothesis outcomes

| ID | Outcome | Evidence |
|----|---------|----------|
| H1 | **ACCEPT** | Arm A stepwise completes under tenant=1; t_all≈4× Arm C |
| H2 | **PARTIAL** | 1-page convert ≈11.5 s vision tax; scales with pages; text bulk still serial without PDF |
| H3 | **REJECT** | 5 chunks / 3 KB md — no inflation on this fixture |
| H4 | **REJECT** | `store_contention.level=normal`; LLM-bound path |
| H5 | **REJECT** | admit_s ≪ t_all on all arms (0.16–0.18 s vs 45–59 s) |

## Harness

See [scripts/measure-bulk-ingest.sh](scripts/measure-bulk-ingest.sh).

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- PDF: [12-pdf-quality-hypothesis.md](12-pdf-quality-hypothesis.md)
