# SPEC-047 e2e artifacts

Generated outputs land here after `bench047` / `make bench047-*` runs.

```text
artifacts/
  smoke/                                   canonical latest (= MV-24 HEAD)
  smoke-post-mv24-chart-crops/             MV-24 Acc gate (Acc 0.433, Chart a_in_e 0.41 flat)
  # Next Acc gate after MV-26/27/28 (caption routing / soft-fail / viewer images)
  smoke-post-mv18-full-chart/              Pass A+specialize re-ingest (Acc 0.423, Chart 0.182, a_in_e 0.41)
  smoke-post-mv18-chart-prompts/           MV-18/19 1-doc Rep probe
  smoke-post-a3-acc-recovery/              Acc peak after empty-arm prune (0.429)
  smoke-post-lineage-la2/                  post L-A2/A3 query-only (0.427)
  smoke-post-q1-grounding/                 post grounding (0.436)
  smoke-post-b2-arm-gate/                  arm honesty tax (0.393)
  smoke-pre-q1-grounding/                  pre-Q1 chart dscope (0.384)
  smoke-p6-soft-resume-document-scope/     soft-resume 8-doc ingest+query
  smoke_p0_baseline/                       early P0_primary W0 (10-doc)
  smoke-post-mv32-*/                       early MV-32 / 1-doc toys
  smoke-invalid-api-down-*/                INVALID — do not score
  core/  full/                             (not yet populated)
```

## How to read `SUMMARY.md`

1. **`valid: true`?** If false, fix ops (`EMPTY_ANSWERS`, ingest, vision) — do not interpret F1.
2. **Acc / F1** — official MMLongBench short-answer metrics on the RAG pipeline.
3. **Compare to LVLM GPT-4o F1≈44.9% only as difficulty**, never as same-task ranking.
4. **Slices** — chart / cross-page / unanswerable tell you *where* to improve.
5. **Retrieval (W0)** — `ops.retrieval.page_hit@5` = gold `evidence_pages` ∩ retrieved chunk `page_start`.
6. Prefer the **locked chain** in [022](../022-reassessment-2026-07-11.md) over any single SUMMARY date.

### Locked chart-fixture Acc chain (P0_mm_ite · document-scope · n=117)

| Run | Acc | F1 | Unans | Pure-text | Chart | page_hit@5 | Notes |
|-----|-----|-----|-------|-----------|-------|------------|-------|
| Pre-Q1 | 0.384 | 0.224 | 0.691 | 0.269 | 0.136 | 0.76 | `smoke-pre-q1-grounding` |
| Post-Q1 | **0.436** | 0.255 | **0.810** | 0.192 | 0.182 | 0.73 | grounding |
| Post-B2 | 0.393 | 0.175 | 0.810 | 0.192 | 0.182 | 0.75 | arm honesty tax |
| Post-A3 | **0.429** | 0.238 | 0.810 | **0.255** | 0.136 | 0.75 | Acc recovery |
| Post-lineage | **0.427** | 0.225 | **0.833** | 0.192 | 0.136 | **0.76** | L-A2/A3; no re-ingest |
| Post-MV18 | **0.423** | 0.232 | 0.786 | **0.216** | **0.182** | 0.72 | Pass A+specialize; Chart a_in_e **0.41** |
| Post-MV24 | **0.433** | **0.262** | 0.738 | — | 0.182 | **0.80** | Crops fired 8/8; Chart a_in_e **0.41** flat (G-A FAIL) |

**Latest HEAD:** [`smoke/`](./artifacts/smoke/) ≡ [`smoke-post-mv24-chart-crops/`](./artifacts/smoke-post-mv24-chart-crops/).  
**Acc peak (query lane):** [`smoke-post-a3-acc-recovery/`](./artifacts/smoke-post-a3-acc-recovery/).  
**Scope hygiene:** [`smoke-post-lineage-la2/`](./artifacts/smoke-post-lineage-la2/).

**Next Acc lever:** **MV-26/27/28** (routing, specialize soft-fail, page-local dump) — crops alone did not lift G-A. Mix ablation ([020](../020-post-q1-first-principles-improvement-plan.md) B3) remains orthogonal.

### Older milestones (historical)

| Run | Profile | Acc | F1 | page_hit@5 | Notes |
|-----|---------|-----|-----|------------|-------|
| First valid | P0_primary | ~0.45 | ~0.29 | — | Plumbing; different fixture size |
| W0 query-only | P0_primary | 0.41 | 0.26 | **0.59** | `smoke_p0_baseline/` |
| P6 soft-resume (8-doc) | P0_mm_ite | 0.384 | 0.224 | 0.76 | Pre-Q1 ingest baseline |
| Invalid soft-resume | P0_mm_ite | — | — | — | **Do not interpret** |

**Ingest speed note:** Do **not** `force_reindex` on `--resume`. Soft-reprocess keeps markdown. Throughput defaults: see Makefile (`WORKER_THREADS=16`, `MM_IMAGE_CONCURRENCY=8`, `SOURCE_IDS KEEP=200`, etc.).

**Lineage (021, 2026-07-11):** L-A1–A4 landed (plural docs, fail-closed scope, scoped kg pick, doc-diverse KEEP). Query-only smoke sufficient; re-ingest optional for stamped `source_document_ids[]`.

Details: [022 Re-Assessment](../022-reassessment-2026-07-11.md) · [012 Acceptance](../012-acceptance-criteria-and-scorecard.md) · live [smoke/SUMMARY.md](./artifacts/smoke/SUMMARY.md)

## Rules

- **Do not commit PDFs.**
- **Do not commit API keys.**
- Scorecards/SUMMARYs may be committed selectively for progression history.
