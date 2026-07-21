# Ablation — B2 force-ingest smoke (flat context pack)

**Profile:** `B2_md_glean_force_ingest_v1_lrlike_arms_v2`  
**Workspace:** `e0270f5f-0b6c-4e90-882f-5f9b0eac8cff`  
**Pins:** markdown + gleaning=1 · chunk 1200/100 · query pack = **flat** + bm25 (not A1)

## Result (n=40)

| Metric | EQ | LR | Δ |
|--------|----|----|---|
| Acc | 0.721 | 0.780 | −0.059 |
| ctx_rel | 0.381 | 0.525 | −0.144 |
| evidence_recall | 0.867 | 0.963 | −0.096 |

## Verdict

Cold-ingest dual-SUT smoke under default (non-A1) query pins. Headline B2 Acc is the A1 query-only on this WS: [`T071732Z`](../smoke-20260720T071732Z/).
