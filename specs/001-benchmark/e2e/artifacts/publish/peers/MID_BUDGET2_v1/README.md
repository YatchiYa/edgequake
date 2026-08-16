# Labeled publish peer — `MID_BUDGET2_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Pin

`EDGEQUAKE_RELATED_CHUNK_NUMBER=2` only (prompt/chunk-admission budget probe, LightRAG formula `rcn × n_entities / 2`). GWC off, pool=mix.

## Result (`medical-mid-20260815T143947Z`)

| Metric | EQ | LR | vs Beat Mid |
|--------|-----|-----|-------------|
| Acc | 0.782 | 0.764 | CI [-0.022, +0.077] · tie |
| ctx_rel | 0.482 | 0.495 | **FAIL** (<0.50) |
| overall ER | 0.942 | 0.951 | PASS |
| Fact ER | 0.883 | 0.938 | **FAIL** (need ≥0.920) |
| EQ query p50 | **4180 ms** | 708 ms | latency improved vs 5715 baseline |

Fact ctx ~152k — **budget lever did not shrink the prompt** (bounded by k=30 token budget, not per-entity admission).

**Verdict:** REJECT for Beat. Third orthogonal lever (after compress, citation-pool) hits the same SNR ceiling. Confirms the binding constraint is the k=30 prompt budget / `min_chunk_budget_ratio`, and lowering it would cost evidence recall below gate.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
