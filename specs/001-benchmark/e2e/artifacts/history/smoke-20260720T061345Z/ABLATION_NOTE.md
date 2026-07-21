# Ablation — A1_p2b_rr_cer_v1

**Step:** a1  
**Pins:** 028 A1: P2b + CONTEXT_FORMAT=rr_cer (relation-first pack; path prune still on)  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.772 | 0.786 | −0.014 (CI includes 0) |
| Complex Acc | 0.816 | 0.845 | −0.029 |
| Fact Acc | 0.764 | 0.727 | +0.037 |
| Summarize Acc | 0.753 | 0.841 | −0.088 |
| ctx_rel | 0.438 | 0.531 | −0.094 |
| evidence_recall | 0.866 | 0.959 | −0.094 |

## A1 success gate

- Complex Δ ≤ 0.05: **PASS** (−0.029)
- Acc ≥ 0.736: **PASS** (0.772)
- ctx_rel ≥ 0.50: **FAIL** (0.438)

**Decision:** No promote. Acc/Complex improved vs P2b peer, but L2 ctx/recall regressed. Next: **A3** answer-prompt (or A2 intent) may help Acc without further packing; **B1/B2** for recall ceiling. Do not soft-Mix fish.
