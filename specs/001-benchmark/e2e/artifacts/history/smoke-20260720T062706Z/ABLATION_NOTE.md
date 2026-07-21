# Ablation — A4_acc_ci_a1_rr_cer_v1

**Step:** a4 (`BENCH001_A4_PACKAGE=a1`)  
**Pins:** 028 A4 Acc CI on A1 pack (P2b + `CONTEXT_FORMAT=rr_cer`)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | **0.767** | 0.756 | **+0.011** (CI includes 0) |
| Complex Acc | 0.775 | 0.792 | **−0.017** |
| Fact Acc | 0.730 | 0.677 | +0.053 |
| Summarize Acc | 0.808 | 0.826 | −0.018 |
| ctx_rel | 0.494 | 0.550 | −0.056 |
| evidence_recall | 0.914 | 0.947 | −0.032 |

## Promote gates

| Gate                    | Result                          |
| -------------------------| ---------------------------------|
| Beat (CI excludes 0 EQ) | **FAIL** (CI includes 0)        |
| Parity (CI includes 0)  | PASS                            |
| ctx_rel ≥ 0.50          | **FAIL** (0.494)                |
| recall ≥ LR−0.03        | **FAIL** (0.914 vs need ≥0.917) |

## Verdict

**No promote.** Best Horizon A Acc pack is A1/`rr_cer`: Acc can lead point-estimate and Complex Δ is near-zero, but L2 gates still miss. Next: **Horizon B** (ingest) for recall/ctx ceiling — not more soft Mix fishing. A2/A3 stacks do not beat A1 Acc.
