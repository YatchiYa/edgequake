# Ablation — B3b A1 after label-FTS (032 follow-up)

**Archive:** `smoke-20260720T085257Z` · query-only on B3b WS `2a7bcb2f-…`  
**Change:** FTS/trigram/prefix search bare `label` (not scoped `node_id`); keyword validation passes `workspace_id`.

## Result

| Metric | EQ | LR |
|--------|-----:|-----:|
| Acc | **0.749** | 0.759 |
| evidence_recall | 0.941 | 0.962 |
| context_relevancy | **0.519** | 0.538 |

Statistical tie (CI includes 0). Acc ↑ vs pre-FTS A1 (0.727) but still **below B2 Acc 0.785** — no Beat promote.

## Next (first principles, no FAQ)

Denser visible graph (4k nodes) needs Mix context packing / ranking under full WS identity — not extract-density fishing.
