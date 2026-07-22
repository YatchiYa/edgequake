# Ablation — 051 a1fprw RELATION_SELECT=lightrag

**Step:** a1fprw  
**Pins:** a1fp + `EDGEQUAKE_RELATION_SELECT=lightrag` (one confound)  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5 Acc peer)  
**Archive:** `smoke-20260720T154525Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥ 0.781 (peer ≥ 0.801) | **0.761** ✗ |
| ctx_rel | ≥ 0.50 | **0.525** ✓ |
| recall | ≥ LR−0.03 | 0.927 vs 0.964 ✗ |
| Fact Acc | (vs B5 0.765) | **0.666** ↓ |
| Complex Acc | (vs B5 0.813) | **0.824** ↑ |
| Δ Acc 95% CI | Beat excludes 0 EQ | includes 0 |

## Verdict

- [x] Law implemented and pinned (`relation_select=lightrag`)
- [x] Acc gate missed — **do not promote**; keep B5+`a1fp` Acc peer
- Keep code; Acc headline stays `RELATION_SELECT=default`
