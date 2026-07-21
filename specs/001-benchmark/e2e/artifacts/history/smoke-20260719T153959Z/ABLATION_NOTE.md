# Acc-win E2 — query-conditioned entity ranking

**Date:** 2026-07-19  
**Archive:** `smoke-20260719T153959Z`  
**Confound vs S1:** `EDGEQUAKE_ENTITY_RANK=query_score` (was degree) · path stays **off**  
**Pins:** CE `qwen3-rerank` · `PROTECT_FIRST=12` · path 0 · `entity_rank=query_score` · prune off · `top_k=30`  
**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Code:** `entity_rank.rs` + `query_pipeline` postprocess

## Results

| Metric | S1 `T151125Z` | E1 path `T153436Z` | **E2 this run** | Gate |
|--------|---------------|--------------------|-----------------|------|
| EQ Acc | 0.760 | 0.742 | **0.734** | drop vs S1 −0.026 (tax) |
| LR Acc | 0.780 | 0.774 | 0.751 | — |
| Δ Acc 95% CI | includes 0 | includes 0 | [−0.108, +0.076] | includes 0 |
| EQ ctx_rel | 0.519 | 0.519 | **0.519** | ≥0.48 floor ✅ |
| EQ recall | 0.928 | 0.928 | **0.941** | ↑ |
| Complex Acc EQ→LR | 0.752→0.835 | 0.757→0.836 | 0.744→0.813 | still ~−7pp |
| Complex F1 EQ→LR | 0.681→0.791 (Δ−0.110) | 0.687→0.793 (Δ−0.106) | 0.669→0.763 (**Δ−0.094**) | need \|Δ\|≤0.03 ❌ |

## Verdict

1. **Code shipped:** `EDGEQUAKE_ENTITY_RANK=query_score|degree` (default degree). Scorecard pin `entity_rank=query_score` confirmed.
2. **Complex gate missed:** ΔF1 vs LR improved slightly (−0.110 → −0.094) but far from ≤0.03. Prompt entity order alone is insufficient.
3. **L2:** ctx_rel held at 0.519; recall improved (+1.3pp).
4. **Acc:** point estimate worse than S1; CI still includes 0. **Do not promote.**
5. **Next (E3):** Summarize coverage (naive/global weight or related_chunk audit) — separate confound. Consider relation ordering / path-serialized blocks for Complex (research, not silent stack).

See [017](../../../../001-edgquake-improvements/017-beat-lightrag.md).
