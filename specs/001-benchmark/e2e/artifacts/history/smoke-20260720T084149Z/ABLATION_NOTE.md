# Ablation — B3b workspace-scoped AGE identity (valid ingest)

**Archive:** `smoke-20260720T084149Z` · WS `2a7bcb2f-…`  
**STRUCTURE_INDUCE:** off · A1 concurrency ≤4 · settle gate OK (4215 entity vectors)

## Identity (first principles)

- `age_over_vectors` = **1.0819** (gate [0.90, 1.10] PASS)
- Entity vectors **4215** ≈ LR entities 3580 (density not starved)
- AGE nodes now WS-owned via `{ws}::NAME` — closes global node_id collision

## Acc (do not Beat-promote)

| Metric | EQ | LR |
|--------|-----:|-----:|
| Acc | 0.734 | 0.785 |
| evidence_recall | **0.960** | 0.961 |
| context_relevancy | 0.394 | 0.550 |

Recall parity achieved; ctx/Acc still miss promote (Acc≥0.775, ctx≥0.50).

## Not FAQ

B3a FAQ induce remains closed. No structure-induce heuristics on Acc.
