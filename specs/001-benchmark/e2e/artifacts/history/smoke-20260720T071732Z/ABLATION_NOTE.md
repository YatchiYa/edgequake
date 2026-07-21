# Ablation — B2 re-ingest + A1 `rr_cer` (clean retry)

**Step:** B2 (markdown + gleaning re-ingest) → A1 query-only  
**Pins:** chunk 1200/100 · adaptive off · `chunk_strategy=markdown` · gleaning=1 · P2b + `CONTEXT_FORMAT=rr_cer`  
**Workspace:** `e0270f5f-0b6c-4e90-882f-5f9b0eac8cff` (new; warm peer `8b359190-…` preserved)  
**Note:** Prior A1 on this WS [`T071121Z`](../smoke-20260720T071121Z/) was **invalid** (`empty_answer_rate=0.125`) from Postgres pool timeouts under concurrency=8. This retry used `BENCH001_ACC_QUERY_CONCURRENCY=4`.

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | **0.785** | 0.780 | **+0.006** (CI includes 0) |
| Complex Acc | 0.844 | 0.850 | **−0.006** |
| Fact Acc | 0.687 | 0.702 | −0.016 |
| Summarize Acc | 0.822 | 0.842 | −0.020 |
| ctx_rel | 0.494 | 0.538 | −0.044 |
| evidence_recall | 0.928 | 0.963 | −0.034 |

## Ingest audit ([`20260720T070838Z`](../../ingest-audit/20260720T070838Z/))

| Signal | B2 WS | Pre-B2 warm (`8b359190`) |
|--------|-------|--------------------------|
| EQ nodes | 392 | 429 |
| Soft-overlap | 0.640 | ~0.66 |
| Zero-chunk | 44 (11.2%) | 62 |

## Promote gates

| Gate | Result |
|------|--------|
| Beat (CI excludes 0 EQ) | **FAIL** (CI includes 0) |
| Parity (CI includes 0) | PASS on Acc CI |
| ctx_rel ≥ 0.50 | **FAIL** (0.494) |
| recall ≥ LR−0.03 | **FAIL** (0.928 vs need ≥0.932) |
| B2 soft-overlap ≥0.75 or ents≥0.5×LR | **FAIL** |
| B2 zero-chunk ≤5% | **FAIL** |
| Acc ≥ A4−0.02 (0.747) | **PASS** (0.785) |

## Verdict

**No Beat/Parity promote** (L2 still miss by a hair). **Best Acc point-estimate + best Complex Δ** on the Acc ladder so far — B2 markdown+glean helps packing/Acc even though entity **count** did not close vs LR. Keep warm peer `8b359190-…` as pre-B2 baseline; treat `e0270f5f-…` as B2 candidate WS. Next: **B3** structure-aware / paragraph chunking for recall/ctx + entity density.
