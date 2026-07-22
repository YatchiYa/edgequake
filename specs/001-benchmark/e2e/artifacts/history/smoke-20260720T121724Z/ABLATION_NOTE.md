# Ablation — 045 a1fpcmat CONTENT-gated materialize

**Archive:** `smoke-20260720T121724Z`  
**Workspace:** `8e990410-…` (B5 peer, query-only)  
**Profile:** `A1FPCMAT_…_topic_mat_content_v1`  
**Confound:** `TOPIC_ENTITY_ADMIT` + `TOPIC_MATERIALIZE` + `TOPIC_MATERIALIZE_CONTENT=1`

## vs peer B5+a1fp [`T120315Z`](../smoke-20260720T120315Z/)

| Metric | Peer a1fp | a1fpcmat | Gate |
|--------|----------:|---------:|:-----|
| Acc | **0.801** | 0.733 | ✗ Acc≥0.755 |
| Fact ER | **0.85** | 0.80 | ✗ ≥0.83 |
| Sum ER | 0.863 | **0.963** | ✓ ≥0.90 |
| Overall recall | 0.926 | **0.933** | ~Parity (LR−0.03=0.937) |
| ctx | 0.519 | 0.513 | ✓ ≥0.50 |
| Probe `bone cancers` | ✗ | **✓** | nice-to-have |

Δ Acc vs LR **−0.052** · CI **[-0.132, +0.027]** — no Beat.

## Verdict

**REJECT** as Acc headline — CONTENT gate proves CE_GAP SELECT (Sum ER↑, probe✓) but Acc/Fact tax persists (same class as blind 042).

**STOP** topic-SELECT Acc fishing (043). Keep **B5+a1fp** Acc peer. Optional labeled L2/Sum package only — not promote.
