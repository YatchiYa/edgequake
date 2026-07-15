# 038 — Phase B CORE checkpoint @ 5 docs (VALID)

**Date:** 2026-07-15  
**Artifact:** `e2e/artifacts/core-checkpoints/at_5_docs/`  
**Workspace:** `b994167c-180e-4708-96a1-a1778b450f15`  
**Stack:** Acc #5 `BEST_SCORE_STACK` · `P0_mm_ite` · W3-arith-v2 · document-scope · Small  
**Fixture:** `core_doc_ids_v1.txt` first 5 (not chart-8)  
**n_scored:** 79

---

## Scores

| Metric | Value |
|---|---:|
| Acc | **0.5494** |
| F1 | **0.4910** |
| valid | True |
| ingest_coverage | 1.0 |
| Chart `a_in_e_long` | **1.00 PASS** (n=5) |
| Table `a_in_e_long` | **0.70 PASS** (n=10) |

### Docs

1. `e79deb…`  
2. `afe620…`  
3. `germanwings…`  
4. `2311.16502v3.pdf`  
5. `measuringsuccessonfacebooktwitterlinkedin…`

---

## Honesty

- This is the **first valid Phase B CORE** milestone (fixture-correct).
- Prior `at_5_docs_CHART8_FIXTURE_MISRUN` Acc 0.485 is **void**.
- CORE@5 Acc/F1 **≠** chart-8 Acc #5 (0.562 / 0.480) — different Q mix; do not claim Acc win from this alone.
- Chart long n=5 is small; gate PASS is informative but not a 40-doc claim.

---

## Follow

- Ladder continuing: **max-docs=10** with `--resume` / `force_reindex=False` (already started).
- Next checkpoints: 10 → 15 → 20 → 25 → 30 → 35 → 40.
