# Acc-win E3 — Summarize coverage (`RELATED_CHUNK_NUMBER=8`)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T154427Z`  
**Confound vs S1:** `EDGEQUAKE_RELATED_CHUNK_NUMBER` 5 → **8**  
**Pins:** CE `qwen3-rerank` · `PROTECT_FIRST=12` · path off · `entity_rank=degree` · related_chunk **8** · prune off · `top_k=30`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Results

| Metric | S1 `T151125Z` | **E3 this run** | Gate |
|--------|---------------|-----------------|------|
| EQ Acc | 0.760 | **0.752** | — (CI still includes 0) |
| LR Acc | 0.780 | 0.743 | — |
| Δ Acc 95% CI | includes 0 | [−0.085, +0.111] | includes 0 |
| EQ ctx_rel | 0.519 | 0.506 | ≥0.50 (bare) |
| EQ overall recall | 0.928 | 0.926 | flat |
| **Summarize evidence_recall** | **0.863** | **0.863** | need ≥**0.95** ❌ |
| LR Summarize recall | — | 0.983 | — |
| Fact Acc (EQ) | 0.709 | **0.715** | not↓ ✅ |

## Verdict

1. **Gate missed:** Raising `related_chunk_number` 5→8 did **not** move Summarize evidence_recall (stuck at 0.863 vs LR 0.983).
2. **Fact Acc** held / slightly up — no Fact tax from the wider KG→chunk take.
3. **Hypothesis update:** Summarize miss is not “too few chunks per entity.” Likely **naive/global fusion weight**, post-fuse truncation, or keyword/global arm under-coverage for long-form types — not `related_chunk` take alone.
4. **Do not promote** related_chunk=8 to Acc headline (fairness pin stays 5).
5. **Next confound (E3b or E4 path):** labeled Mix naive-weight boost (`EDGEQUAKE_MIX_NAIVE_WEIGHT` env — needs code if missing) **or** Summarize-focused global weight; keep one confound. Then E4 Acc CI only after a coverage win.

See [017](../../../../001-edgquake-improvements/017-beat-lightrag.md) · [011 R3](../../../../001-edgquake-improvements/011-lens-evidence-coverage.md).
