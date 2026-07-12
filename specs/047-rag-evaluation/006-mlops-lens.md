# 006 — MLOps Lens

**Cross-ref:** [003](./003-fair-evaluation-protocol.md) · [010](./010-smoke-then-full-runbook.md) · [012](./012-acceptance-criteria-and-scorecard.md)

---

## 1. Reproducibility contract

```text
  same dataset revision
+ same fixture list
+ same EdgeQuake VERSION/git SHA
+ same provider profile
+ same extractor
≈ same Acc/F1 within noise band (see 007)
```

Store everything needed to re-run in `meta.json` (no secrets).

---

## 2. Cost & time envelopes (planning)

Rough order-of-magnitude (update after first smoke):

| Stage | Docs | Pages (≈) | Dominant cost | Wall time (order) |
|-------|------|-----------|---------------|-------------------|
| Smoke | 10 | ~475 | vision ingest + embed + extract | hours |
| Core | ~40 | ~1,900 | same | ~1 day |
| Full | 135 | ~6,400 | same | multi-day |

Controls:

- `EDGEQUAKE_EMBEDDING_BATCH_SIZE=16`  
- ingest concurrency ≤ 2  
- cache PDFs forever locally  
- skip re-ingest on `--resume`  
- optional: store markdown once; do not re-vision unless `--force-reingest`

**Budget gate:** `doctor` prints estimated page count × configured $ / page heuristic before starting; require `--i-accept-cost` for core/full.

---

## 3. Caching layers

| Layer | Key | Invalidate when |
|-------|-----|-----------------|
| PDF cache | dataset revision | revision changes |
| Ingest | workspace + doc sha + vision model | model/profile change |
| Predictions | question_id + mode + model | prompt/mode change |
| Extractions | (question, long_answer_hash, extractor) | extractor change |

---

## 4. CI / Nightly policy

| Job | Trigger | Scope |
|-----|---------|-------|
| `bench047-unit` | PR | scorer parity tests, schema validate, no API |
| `bench047-doctor` | PR (if secrets) | health + vision flag |
| `bench047-smoke-nightly` | nightly + `MISTRAL_API_KEY` | 10 docs |
| `bench047-full` | manual workflow_dispatch | 135 docs |

Do **not** block every PR on smoke (cost). Block PRs on unit + schema.

Secrets: `MISTRAL_API_KEY`, optional `OPENAI_API_KEY` for official extractor.

---

## 5. Observability during runs

- Tail `/tmp/edgequake-backend.log` for vision/embed errors.  
- Emit harness progress: `docs_done/docs_total`, `qs_done/qs_total`, ETA.  
- Heartbeat JSON every N minutes for long full runs.  
- On rate-limit (429): exponential backoff, record in meta.

---

## 6. Data governance

- NC license → research CI only; document in workflow.  
- No PDF upload to public artifacts / GitHub releases.  
- Scorecards + SUMMARY.md are OK to commit **without** raw document bytes.  
- Redact API keys from logs (scan before upload).

---

## 7. Environment promotion

```text
  local smoke  →  nightly smoke  →  release candidate core  →  versioned full
```

Tag artifacts: `bench047-full-edgequake-0.X.Y-mmlongbench-{dataset_rev}.json`

---

## 8. MLOps acceptance

- [ ] Resume works after kill -9  
- [ ] Cost confirmation flag for core/full  
- [ ] Nightly smoke workflow sketched  
- [ ] Artifacts exclude PDFs  

Next: [007 ML Scientist](./007-ml-scientist-lens.md).
