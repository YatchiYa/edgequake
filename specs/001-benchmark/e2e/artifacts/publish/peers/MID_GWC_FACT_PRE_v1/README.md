# Labeled publish peer — `MID_GWC_FACT_PRE_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Pins (coupled lever)

- `EDGEQUAKE_GRAPH_WALK_COMPRESS=1` (compress reasoning context)
- `EDGEQUAKE_L2_FACT_BM25_POOL=acc` (Fact BM25 over Acc-admitted chunks)
- `EDGEQUAKE_L2_FACT_BM25_POOL_PRE_COMPRESS=1` (pool = pre-compress snapshot — decouple judge sources from GWC)

## Result (`medical-mid-20260815T141834Z`)

| Metric | EQ | LR | vs Beat Mid |
|--------|-----|-----|-------------|
| Acc | 0.768 | 0.782 | CI [-0.074, +0.024] · LR point ahead |
| ctx_rel | 0.485 | 0.491 | **FAIL** (<0.50) |
| overall ER | 0.940 | 0.947 | PASS |
| Fact ER | 0.905 | 0.940 | **FAIL** (need ≥0.910) |

**Verdict:** REJECT. Pre-compress pool kept Fact ER ≈ W3 (0.905) but ctx recovered only partway (0.485 vs GWC-alone 0.501). Composition still does not clear both Beat gates on one mid pack. Do not promote `publish/latest`; do not scale to medical-full for Acc Beat.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
