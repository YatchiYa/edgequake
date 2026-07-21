# Ablation — P1b_gw_compress_s1_v1

**Step:** p1b  
**Pins:** `GRAPH_WALK_COMPRESS=1` on S1 CE+protect (`PATH_PRUNE=0`, protect=12)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T014115Z`

## Results

| Metric | P1b | Gate | Result |
|--------|-----|------|--------|
| EQ Acc | 0.739 | not worse than P0 (0.744) by >0.02 | **−0.005 ok** |
| EQ ctx_rel | **0.494** | ≥0.48 | **ok** |
| Complex Acc EQ/LR | 0.751 / 0.883 | Δ≤0.05 | **−0.132 miss** |
| Fact Acc | 0.766 | drop≤0.02 vs P0 | **ok** (ahead) |
| Δ Acc 95% CI | [−0.131, +0.060] | — | tie |

## Verdict

- [ ] Gate met (full P1)
- [x] Partial — L2 ctx_rel cleared with S1+gw; **Complex packing still open** → P2

**Compare:** P1a BM25+gw failed L2; P1b S1+gw recovers ctx_rel≈0.49 (near S1 T151125Z 0.519).
