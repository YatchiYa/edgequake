# Ablation — P4_acc_ci_decision_v1

**Step:** p4  
**Pins:** S1 CE+protect + `GRAPH_WALK_COMPRESS=1` + `ENTITY_RANK=retrieval` + `KEYWORD_LEXICAL_BOOST=1` + `PATH_PRUNE=0`  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T015647Z`

## Results

| Metric | Value | Gate | Result |
|--------|-------|------|--------|
| EQ Acc | **0.677** | — | **regress** vs P0/P2b |
| LR Acc | 0.788 | — | — |
| Δ Acc 95% CI | **[−0.216, −0.007]** | excludes 0 **EQ** | **fail** (excludes 0 **LR**) |
| EQ ctx_rel | **0.500** | ≥0.50 | ✅ |
| evidence_recall EQ/LR | 0.899 / 0.969 | ≥LR−0.03 | **miss** (−0.070) |

## Verdict

- [ ] Gate met → promote Acc headline
- [x] Gate missed — **do not promote**

**Decision (023):** Keep Acc headline BM25 / `PATH_PRUNE=0` / `PROTECT=0` / `PRUNE=0`. Honest framing: improved peer on P2b (`T014814Z` Acc 0.752, ctx_rel 0.50, Complex Δ −0.023) but **no** CI win; P4 stacked package is Acc-toxic vs P2b lr_pack.

**Carry-forward:** Prefer P2b knobs for labeled L2 ablations; do not ship P4 stack as Acc default.
