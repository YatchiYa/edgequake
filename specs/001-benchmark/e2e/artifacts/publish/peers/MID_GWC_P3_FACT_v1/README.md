# Labeled publish peer — `MID_GWC_P3_FACT_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Pins

- `EDGEQUAKE_GRAPH_WALK_COMPRESS=1`
- `EDGEQUAKE_GRAPH_WALK_COMPRESS_NAIVE_PROTECT=3` (hard compress floor)
- `EDGEQUAKE_L2_FACT_BM25_POOL=acc`

## Result (`medical-mid-20260815T142806Z`)

| Metric | EQ | LR | vs Beat Mid |
|--------|-----|-----|-------------|
| Acc | 0.794 | 0.786 | CI [-0.094, +0.086] · tie |
| ctx_rel | 0.478 | 0.494 | **FAIL** (<0.50) |
| overall ER | 0.937 | 0.949 | PASS |
| Fact ER | 0.894 | 0.950 | **FAIL** (need ≥0.920) |

Fact ctx ~115k — protect=3 did **not** shrink the blob vs protect=8 (k=30 chunk budget dominates).

**Verdict:** REJECT. Full lever grid exhausted — no single-confound or coupled composition clears both `ctx_rel ≥ 0.50` and `Fact ER ≥ LR−0.03` on one mid pack. Do not promote `publish/latest`; no medical-full Acc Beat scale.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
