# Ablation — P1a_gw_compress_bm25_v1

**Step:** p1a  
**Pins:** `EDGEQUAKE_GRAPH_WALK_COMPRESS=1` on BM25 Acc base (`PATH_PRUNE=0`)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T013827Z`

## Results

| Metric | P1a | P0 restore | Gate |
|--------|-----|------------|------|
| EQ Acc | 0.721 | 0.744 | not worse by >0.02 → **−0.023 miss** |
| EQ ctx_rel | 0.375 | 0.381 | ≥0.48 → **miss** |
| Complex Acc EQ/LR | 0.661 / 0.887 | 0.658 / 0.830 | Δ≤0.05 → **−0.226 miss** |
| Fact Acc | 0.701 | 0.715 | drop≤0.02 → **ok** |
| Δ Acc 95% CI | [−0.161, +0.034] | includes 0 | tie |

## Verdict

- [ ] Gate met
- [x] Gate missed — gw_compress on BM25 alone does **not** close Complex/ctx_rel; Acc tax vs P0

**Next:** P1b (same compress on S1 CE+protect) — one confound change = S1 base.
