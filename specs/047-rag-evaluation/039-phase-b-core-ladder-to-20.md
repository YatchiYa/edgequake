# 039 — Phase B CORE complete (@5→@40)

**Date:** 2026-07-16  
**Finished:** `DONE_OK 2026-07-16T04:34:07Z` (~3.4 h resume)  
**Workspace:** `b994167c-180e-4708-96a1-a1778b450f15`  
**Stack:** Acc #5 / `P0_mm_ite` · W3-arith-v2 · document-scope · mistral-small + mistral-embed  
**Ingest:** **40/40 completed** · exit 0

---

## Verdict

Phase B CORE ladder **complete**. Full-ingest Acc settles ~**0.458** at @35–@40.  
Wave-1 Chart/Table long still **PASS** at full-n.  
Do **not** cite @20/@25 Acc (partial ingest). Chart-8 Acc/F1 SOTA remains **0.562 / 0.480**.

---

## Final ladder

| Docs | Acc | F1 | cov | honest? | Cite? |
|---:|---:|---:|---:|---|---|
| 5 | **0.549** | **0.491** | 1.00 | YES | yes |
| 10 | **0.529** | **0.422** | 1.00 | YES | yes |
| 15 | **0.533** | **0.440** | 1.00 | YES | yes |
| 20 | 0.503 | 0.407 | 0.90 | NO | no |
| 25 | 0.471 | 0.373 | 0.96 | NO | no |
| 30 | **0.467** | **0.359** | 1.00 | YES | yes |
| 35 | **0.459** | **0.356** | 1.00 | YES | yes |
| **40** | **0.458** | **0.356** | 1.00 | YES | **final** |

Honest Acc: 0.549 → 0.529 → 0.533 → 0.467 → 0.459 → **0.458**. Softens then plateaus.

---

## Wave-1 @40 (full-n)

| Metric | Value | Gate |
|---|---:|---|
| a_in_e_long | 0.667 | — |
| Chart long | **0.600** | ≥0.50 PASS |
| Table long | **0.585** | ≥0.55 PASS |

---

## Honesty

Questions only for completed docs. Core requires cov==1.0.  
@20/@25 tainted by pool-era misses (earlybird/t480). Optional re-score later; not required for Phase B close.

---

## Follow (optional)

1. Re-score max-docs=20/25 for honest mid-ladder fill-in.  
2. Next Acc work stays chart-8 (operand / %×N), not CORE Acc chasing.  
3. Archive snapshot under `e2e/artifacts/core-phase-b-*` if desired.
