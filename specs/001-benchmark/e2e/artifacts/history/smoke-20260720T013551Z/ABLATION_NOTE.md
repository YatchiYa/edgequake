# Ablation — P0_path_off_bm25_restore_v1

**Step:** p0  
**Pins:** Acc headline `PATH_PRUNE=0` BM25 restore (fix T011703Z BM25+path=0.4 confound)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T013551Z`

## Results

| Metric | Value |
|--------|-------|
| EQ Acc | **0.744** |
| LR Acc | **0.794** |
| Δ Acc 95% CI | **[-0.100, +0.006]** (includes 0 → tie) |
| EQ / LR ctx_rel | 0.381 / 0.538 |
| EQ / LR evidence_recall | 0.924 / 0.963 |
| path_prune_fraction pin | **0.0** |
| vs T124903Z EQ Acc (0.765) | Δ −0.021 (within ±0.03 gate) |
| vs T011703Z EQ Acc (0.699) | recovered +0.045 |

## Gates

| Gate | Target | Result |
|------|--------|--------|
| path_prune_fraction pin | 0 | **0.0** ✅ |
| Δ Acc 95% CI | includes 0 | **includes 0** ✅ |
| EQ Acc vs T124903Z | within ±0.03 | **−0.021** ✅ |

## Verdict

- [x] Gate met — Acc-tie class restored (not −9pp LR-win publish)
- [ ] Gate missed (do not promote)

**Note:** Still point-estimate behind LR; Complex Acc 0.658 vs LR 0.830 remains the binding gap → continue P1 graph-walk compress.
