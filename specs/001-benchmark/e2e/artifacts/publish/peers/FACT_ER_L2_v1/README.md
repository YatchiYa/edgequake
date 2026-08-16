# Labeled publish peer — `FACT_ER_L2_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## Pin

`EDGEQUAKE_L2_FACT_BM25_POOL=acc` only (088 W3) — FactReplace BM25 over Acc-admitted prompt chunks (not Mix pre-CE). Soft Mix / cosine / CE / GWC off.

## Result (`medical-mid-20260815T135129Z`)

| Metric | Baseline T110218Z | FACT_ER_L2 | Gate |
|--------|-------------------|------------|------|
| Acc | 0.792 / 0.786 tie | 0.775 / 0.773 tie CI | Acc CI ≥tie KEEP |
| ctx_rel | 0.471 | 0.471 | flat · FAIL Beat |
| overall ER | 0.932 | 0.949 | PASS |
| Fact ER | 0.847 | **0.896** vs LR 0.930 | ≥base−0.01 KEEP; Beat need ≥0.900 (miss −0.004) |

**Verdict:** KEEP on Fact ER progress (+4.9pp). Beat STOP (Fact ER −0.4pp vs LR−0.03; ctx unmet). Next Mid candidate = GWC + pool=acc combined pack.

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
