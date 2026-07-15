# 036 — Phase B CORE checkpoint @ 5 docs

**Date:** 2026-07-15  
**Artifact:** `e2e/artifacts/core-checkpoints/at_5_docs/`  
**Workspace:** `1ec17559-816d-4ca7-b80c-4d0f8de3ca84`  
**Stack:** Acc #5 / `BEST_SCORE_STACK` · `P0_mm_ite` · W3-arith-v2 · document-scope · Small  
**Build:** `20260715.134954` (`5f32605a`)

---

## Scores (first 5 of 40 core docs)

| Metric | Value |
|---|---:|
| Acc | **0.4847** |
| F1 | **0.3472** |
| valid | True |
| ingest_coverage | 1.0 |
| Chart `a_in_e_long` | **0.60 PASS** (n=10) |
| Table `a_in_e_long` | **0.583 PASS** (n=12) |

Docs: political · 2311 · PIP seniors · afe620 · e79deb (smoke-overlapping prefix of core fixture).

---

## Honesty

- 5-doc prefix is **not** comparable to chart-8 Acc #5 (0.562 / 0.480). Different Q mix.
- Use this for Phase B progression tracking every 5 docs.
- Checkpoint script bug (`n` undefined) fixed; resume from max-docs=10.

---

## Next

Continue ladder: 10 → 15 → 20 → 25 → 30 → 35 → 40 with `BENCH047_RESUME=1`.
