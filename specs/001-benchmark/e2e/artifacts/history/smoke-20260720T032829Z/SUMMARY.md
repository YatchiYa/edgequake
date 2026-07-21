# SPEC-001 smoke SUMMARY

> EQ mix vs LR mix on GraphRAG-Bench (mistral/mistral-small-latest + mistral/mistral-embed) — publishable dual-SUT under matched top-k + L2 retrieval metrics. Not UltraDomain win-rates; Acc is not paper Table-2 comparable unless P0_paper ablation pins are used.

- **valid:** `True`
- **profile:** `T0_p2b_fact_bm25_v1_lrlike_arms_v2`
- **judge:** `generation_eval`
- **fixture:** `smoke_question_ids_v1` (n=40)
- **dataset revision:** `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`

## Model lineage

- **sut_llm:** `mistral/mistral-small-latest`
- **sut_vision:** `mistral/mistral-small-latest`
- **sut_embed:** `mistral/mistral-embed@1024d`
- **judge_llm:** `mistral/mistral-small-latest`
- **judge_metric_embed:** `mistral-embed`
- **llm_base_url:** `https://api.mistral.ai/v1`
- **judge_base_url:** `https://api.mistral.ai/v1`

## Overall Acc (L0) — Acc = 0.75·F1 + 0.25·cos

| SUT | Acc | F1 | cos |
|-----|-----|----|-----|
| EdgeQuake mix | 0.7196 | 0.6401 | 0.9581 |
| LightRAG mix | 0.7835 | 0.7227 | 0.9662 |
| Δ (EQ − LR) | -0.0640 | -0.0826 | -0.0081 |

- **Δ Acc 95% CI (bootstrap):** [-0.1678, +0.0371] (n=40)

## Retrieval (L2)

| SUT | evidence_recall | context_relevancy |
|-----|-----------------|-------------------|
| EdgeQuake | 0.9336 | 0.4938 |
| LightRAG | 0.9708 | 0.5000 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 0.6535
- **Complex Reasoning:** 0.6747
- **Contextual Summarize:** 0.8025
- **Creative Generation:** 0.7477

## By question_type (LR Acc)

- **Fact Retrieval:** 0.6846
- **Complex Reasoning:** 0.8635
- **Contextual Summarize:** 0.8510
- **Creative Generation:** 0.7351

## Ops

- EQ empty-answer rate: 0.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 0.000
- LR empty-context rate: 0.000
- EQ query p50/p95 ms: 10921 / 16164
- LR query p50/p95 ms: 1803 / 2478
- ingest wall s: 0.0
- EQ/LR p50 ratio: 6.057 (SLO ≤1.5×: FAIL/WAIVE)
- EQ stage p50 ms: embed=2385, retrieve=813, rerank=1066, generate=2131

## Pins

```json
{
  "edgequake_git_sha": "936b9236",
  "dataset_id": "GraphRAG-Bench/GraphRAG-Bench",
  "dataset_revision": "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546",
  "fixture_id": "smoke_question_ids_v1",
  "profile_id": "T0_p2b_fact_bm25_v1_lrlike_arms_v2",
  "llm_provider": "mistral",
  "llm_model": "mistral-small-latest",
  "vision_provider": "mistral",
  "vision_model": "mistral-small-latest",
  "embedding_provider": "mistral",
  "embedding_model": "mistral-embed",
  "embedding_dim": 1024,
  "llm_base_url": "https://api.mistral.ai/v1",
  "judge_provider": "mistral",
  "judge_model": "mistral-small-latest",
  "judge_base_url": "https://api.mistral.ai/v1",
  "judge_embedding_model": "mistral-embed",
  "lineage": {
    "sut_llm": "mistral/mistral-small-latest",
    "sut_vision": "mistral/mistral-small-latest",
    "sut_embed": "mistral/mistral-embed@1024d",
    "judge_llm": "mistral/mistral-small-latest",
    "judge_metric_embed": "mistral-embed",
    "llm_base_url": "https://api.mistral.ai/v1",
    "judge_base_url": "https://api.mistral.ai/v1"
  },
  "judge_temperature": 0.0,
  "judge_acc_factuality_weight": 0.75,
  "judge_embed_backend": "auto",
  "answer_style": "gold",
  "recommended_mistral_judge": "mistral-medium-latest",
  "acc_formula": "0.75*F1 + 0.25*embed_cosine (weights overridable)",
  "acc_lift_note": "Closer Acc under Mistral judge: answer_style=gold (SUT) + judge_model=mistral-medium-latest; keep L2 gates",
  "publish_fairness": true,
  "retrieve_topk": 30,
  "lr_top_k": 30,
  "lr_chunk_top_k": 30,
  "lr_enable_rerank": false,
  "eq_max_results": 30,
  "eq_rerank_top_k": 30,
  "l2_retrieval_required": true,
  "mix_arm_gate": false,
  "eq_mix_arm_gate_env": "false",
  "mix_fusion": "rrf",
  "related_chunk_number": 5,
  "kg_chunk_occurrence_sort": false,
  "kg_chunk_pick_lr_budget": false,
  "mix_relevancy_prune": false,
  "mix_relevancy_keep": 12,
  "mix_relevancy_score": "rrf",
  "mix_graph_soft_prune": false,
  "eq_reranker": "cross_encoder",
  "eq_reranker_provider": "aliyun",
  "path_prune_fraction": 0.4,
  "path_prune_orphan_entities": false,
  "rerank_protect_first": 12,
  "min_rerank_score": 0.1,
  "entity_rank": "retrieval",
  "mix_local_weight": 1.0,
  "mix_global_weight": 1.0,
  "mix_naive_weight": 1.0,
  "context_format": "path",
  "passage_pack": false,
  "graph_walk_compress": false,
  "popular_node_fallback": false,
  "keyword_lexical_boost": false,
  "content_headings": true,
  "l2_sources_union": false,
  "l2_sources_mix_top_k": 30,
  "fact_reranker": "bm25",
  "intent_rerank": true,
  "min_chunk_budget_ratio": 0.4,
  "query_arm_concurrency": 16,
  "adaptive_chunking": false,
  "chunk_token_size": 1200,
  "chunk_overlap_token_size": 100,
  "ingest_max_chars": null,
  "eq_query_concurrency": 4,
  "lr_query_concurrency_effective": 2,
  "fairness_note": "Matched top-k=30 budgets; LR rerank off (no model); L2 Evidence Recall + Context Relevancy required for valid smoke+; EQ Mix arm gate off (LR-like always-on local+global+naive) unless EDGEQUAKE_MIX_ARM_GATE=true on the server; optional EDGEQUAKE_MIX_FUSION=round_robin ablation (default rrf); Acc PATH_PRUNE=0 (022 P0; soft path only with CE+protect); Phase-1 EDGEQUAKE_MIX_RELEVANCY_PRUNE Acc default off; fair Acc ingest: adaptive_chunking off + chunk_token_size=1200 (LightRAG CHUNK_SIZE parity) unless explicitly ablated; smoke-fast Acc may set BENCH001_INGEST_MAX_CHARS for fast force-ingest (full corpus = 0)",
  "eq_query_mode": "mix",
  "lr_query_mode": "mix",
  "chunk_size": 1200,
  "query_concurrency": 4,
  "eval_concurrency": 24,
  "judge": "generation_eval"
}
```

## Progression

- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`
- This run archives under `specs/001-benchmark/e2e/artifacts/history/`
