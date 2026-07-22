# Ablation — P3a_intent_trunc_audit_v1

**Step:** p3a  
**Pins:** BM25 Acc (`PATH_PRUNE=0`) + intent chunk floor; audit `query_intent` on predictions  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T015129Z`

## Results

| Metric | Value | Gate | Result |
|--------|-------|------|--------|
| EQ Acc | 0.714 | audit (not Acc-win) | tax vs P0 (expected BM25) |
| Summarize evidence_recall EQ/LR | **0.950** / 0.983 | ≥0.95 or ≥LR−0.03 | **≥0.95** ✅ |
| `query_intent` on preds | **40/40** | logged | ✅ |
| path_prune_fraction | 0.0 | Acc default | ✅ |
| EQ ctx_rel | 0.381 | — | BM25 baseline |

## Verdict

- [x] Gate met — intent truncation audit OK; Summarize recall clears P3 floor on BM25
- [ ] Gate missed (do not promote)

**Note:** Acc tax is expected without S1 pack. Carry Summarize floor into P3b lexical boost; Acc promotion still P4-only.
