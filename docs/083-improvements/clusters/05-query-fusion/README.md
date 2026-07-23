# Cluster 05 — Query fusion & retrieval scores

> **Sprint**: 3–4  
> **Laws**: LAW-3, LAW-8  
> **Defects**: D-38/D-39/D-36/C-28 FIXED · D-35/D-37/D-40/X-04/X-05/X-20/X-22 CONFIRMED/PARTIAL

---

## WHY

Retrieval historically embedded polluted vectors, skipped min_score, and panicked on dim mismatch. **FIXED**: question-only `query_vec` (D-38), min_score on fused paths (D-39), sparse fusion naming honesty for weighted mode (D-36), cosine `Result` (C-28). Still open: Mix “weighted sum” docs vs max (D-35), score-scale mixing (D-37), stream stats parity (D-40), BM25/L2 docs, citation stability, Thinking SSE.

## ROOT CAUSE → STATUS

```
  D-38 history in query_vec     FIXED
  D-39 min_score skipped        FIXED
  D-36 weighted=sparse-first    FIXED (named)
  C-28 cosine panic             FIXED
  D-35/D-37/D-40/X-04/X-05…     CONFIRMED backlog
  X-20 citations positional     PARTIAL
```

## SOLUTION

| Primitive | Status |
|-----------|--------|
| `embed_query(question_only)` | FIXED |
| min_score always applied | FIXED |
| Fusion mode naming (D-36) | FIXED |
| cosine → Result | FIXED |
| ScoreScale / stream stats / citations / Thinking | open |

## E2E

`e2e_query_vec_matches_question_only_embedding`, `e2e_min_score_enforced_on_rrf`, `contract_fusion_mode_names`, `unit_cosine_dim_mismatch_is_err`  
Backlog: `unit_score_scale_no_cross_compare`, `contract_stream_stats_superset`, `e2e_fts_language_config`, `contract_citation_stable_ids`
