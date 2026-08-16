# Labeled publish peer — `MID_GWC_FACT_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Pins (W1∩W3)

- `EDGEQUAKE_GRAPH_WALK_COMPRESS=1`
- `EDGEQUAKE_L2_FACT_BM25_POOL=acc`

Soft Mix / cosine / CE off.

## Result (`medical-mid-20260815T135805Z`)

| Metric | EQ | LR | vs Beat Mid |
|--------|-----|-----|-------------|
| Acc | 0.776 | 0.772 | tie CI [-0.042, +0.063] · not EQ-ahead |
| ctx_rel | 0.473 | 0.492 | **FAIL** (&lt;0.50; GWC-alone was 0.501) |
| overall ER | 0.945 | 0.944 | PASS |
| Fact ER | 0.908 | 0.950 | **FAIL** (need ≥0.920) |

**Verdict:** REJECT as Mid winner — composition tax undoes GWC ctx KEEP; Fact ER +6.1pp vs Acc SSOT but still short of Beat. Do **not** promote `publish/latest`. Do **not** run medical-full for Acc Beat on this pack.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
