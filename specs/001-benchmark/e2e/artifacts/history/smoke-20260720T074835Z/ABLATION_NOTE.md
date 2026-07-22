# Ablation — B3a FAQ induce + A1 `rr_cer`

**Step:** B3a (`EDGEQUAKE_STRUCTURE_INDUCE=faq` + markdown + gleaning=1) → A1  
**Workspace:** `951bb6fa-0fa0-4986-8c9b-527e616f613a`  
**Chunks:** 396 (B2 was ~188) · nodes: 533 (B2: 392)

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.663 | 0.774 | **−0.112** (CI excludes 0 → LR) |
| Complex Acc | 0.755 | 0.807 | −0.052 |
| ctx_rel | 0.488 | 0.544 | −0.056 |
| evidence_recall | 0.916 | 0.963 | −0.047 |

## Promote gates

| Gate | Result |
|------|--------|
| Acc ≥ B2 A1−0.01 (0.775) | **FAIL** (0.663) |
| ctx ≥ 0.50 | **FAIL** |
| recall ≥ LR−0.03 | **FAIL** |
| Soft-overlap ≥ 0.70 | **FAIL** (0.636) |

## Verdict

**Acc tax — do not promote.** FAQ induction over-fragmented the Acc medical blob (396 chunks) and **hurt** Acc vs B2 A1 [`T071732Z`](../smoke-20260720T071732Z/) (0.785). Keep B2 WS `e0270f5f-…` as Acc candidate. Next: B3b extract-density (gleaning/merge limits) **without** FAQ spam, or tighten induction (min section tokens / fewer cues).
