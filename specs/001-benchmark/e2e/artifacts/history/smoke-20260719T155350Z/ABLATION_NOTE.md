# Acc-win E3b — Mix naive-weight boost (`MIX_NAIVE_WEIGHT=2`)

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T155350Z`  
**Confound vs S1:** `EDGEQUAKE_MIX_NAIVE_WEIGHT` 1 → **2** (local=1, global=1 → normalized ~0.25/0.25/0.50)  
**Pins:** CE `qwen3-rerank` · `PROTECT_FIRST=12` · path off · related_chunk **5** · entity_rank degree · prune off · `top_k=30`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Code:** `mix_arm_weight_from_env` + Acc harness pin fields

## Results

| Metric | S1 `T151125Z` | E3 related=8 `T154427Z` | **E3b this run** | Gate |
|--------|---------------|-------------------------|------------------|------|
| EQ Acc | 0.760 | 0.752 | **0.734** | Acc tax −0.026 vs S1 |
| LR Acc | 0.780 | 0.743 | 0.786 | — |
| Δ Acc 95% CI | includes 0 | includes 0 | [−0.132, +0.032] | includes 0 |
| EQ ctx_rel | 0.519 | 0.506 | **0.500** | bare ≥0.50 |
| EQ overall recall | 0.928 | 0.926 | 0.933 | slight ↑ |
| **Summarize evidence_recall** | 0.863 | 0.863 | **0.882** | need ≥**0.95** ❌ (+1.8pp) |
| LR Summarize recall | — | 0.983 | 0.983 | — |
| Fact Acc (EQ) | 0.709 | 0.715 | **0.743** | not↓ ✅ |

## Verdict

1. **Code shipped:** `EDGEQUAKE_MIX_{LOCAL,GLOBAL,NAIVE}_WEIGHT` (default 1). Scorecard confirms `mix_naive_weight=2.0`.
2. **Gate missed:** Naive RRF weight ×2 lifts Summarize recall only **+1.8pp** (0.863→0.882) — still **~10pp below** 0.95 and LR 0.983.
3. **Side effects:** Fact Acc improved; overall Acc and ctx_rel slipped vs S1.
4. **Do not promote** naive_weight=2 to Acc headline (defaults stay 1/1/1).
5. **Hypothesis update:** Soft Mix reweighting cannot close Summarize coverage. Remaining candidates (labeled, one confound each): stronger naive (3+) with Acc budget check; **truncation / chunk token budget** for summarize-length contexts; keyword/global arm audit; ingest-side coverage — not more CE.

See [017](../../../../001-edgquake-improvements/017-beat-lightrag.md).
