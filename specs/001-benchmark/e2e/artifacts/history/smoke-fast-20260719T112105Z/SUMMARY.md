# SPEC-001 smoke-fast SUMMARY

> EQ mix vs LR mix on GraphRAG-Bench (mistral/mistral-medium-latest + mistral/mistral-embed) — publishable dual-SUT under matched top-k + L2 retrieval metrics. Not UltraDomain win-rates; Acc is not paper Table-2 comparable unless P0_paper ablation pins are used.

- **valid:** `False` (eq_failed:[Errno 61] Connection refused)
- **profile:** `P0_mistral_medium_mix_chunk1200_v1_lrlike_arms_v2`
- **judge:** `generation_eval`
- **fixture:** `smoke_fast_question_ids_v1` (n=8)
- **dataset revision:** `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`

## Model lineage

- **sut_llm:** `mistral/mistral-medium-latest`
- **sut_vision:** `mistral/mistral-small-latest`
- **sut_embed:** `mistral/mistral-embed@1024d`
- **judge_llm:** `mistral/mistral-medium-latest`
- **judge_metric_embed:** `mistral-embed`
- **llm_base_url:** `https://api.mistral.ai/v1`
- **judge_base_url:** `https://api.mistral.ai/v1`

## Overall Acc (L0) — Acc = 0.75·F1 + 0.25·cos

| SUT | Acc | F1 | cos |
|-----|-----|----|-----|
| EdgeQuake mix | 0.2334 | 0.0192 | 0.8758 |
| LightRAG mix | 0.3621 | 0.1795 | 0.9101 |
| Δ (EQ − LR) | -0.1288 | -0.1603 | -0.0343 |

- **Δ Acc 95% CI (bootstrap):** [-0.3143, +0.0235] (n=8)

## Retrieval (L2)

| SUT | evidence_recall | context_relevancy |
|-----|-----------------|-------------------|
| EdgeQuake | 0.0000 | 0.0000 |
| LightRAG | 0.1250 | 0.1250 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 0.2192
- **Complex Reasoning:** 0.2178
- **Contextual Summarize:** 0.2154
- **Creative Generation:** 0.2811

## By question_type (LR Acc)

- **Fact Retrieval:** 0.2191
- **Complex Reasoning:** 0.2257
- **Contextual Summarize:** 0.5271
- **Creative Generation:** 0.4766

## Ops

- EQ empty-answer rate: 1.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 0.000
- LR empty-context rate: 0.000
- EQ query p50/p95 ms: 0 / 0
- LR query p50/p95 ms: 11310 / 13603
- ingest wall s: 0.0

## Pins

```json
{
  "edgequake_git_sha": "936b9236",
  "dataset_id": "GraphRAG-Bench/GraphRAG-Bench",
  "dataset_revision": "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546",
  "fixture_id": "smoke_fast_question_ids_v1",
  "profile_id": "P0_mistral_medium_mix_chunk1200_v1_lrlike_arms_v2",
  "llm_provider": "mistral",
  "llm_model": "mistral-medium-latest",
  "vision_provider": "mistral",
  "vision_model": "mistral-small-latest",
  "embedding_provider": "mistral",
  "embedding_model": "mistral-embed",
  "embedding_dim": 1024,
  "llm_base_url": "https://api.mistral.ai/v1",
  "judge_provider": "mistral",
  "judge_model": "mistral-medium-latest",
  "judge_base_url": "https://api.mistral.ai/v1",
  "judge_embedding_model": "mistral-embed",
  "lineage": {
    "sut_llm": "mistral/mistral-medium-latest",
    "sut_vision": "mistral/mistral-small-latest",
    "sut_embed": "mistral/mistral-embed@1024d",
    "judge_llm": "mistral/mistral-medium-latest",
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
  "mix_fusion": "round_robin",
  "related_chunk_number": 5,
  "adaptive_chunking": false,
  "chunk_token_size": 1200,
  "chunk_overlap_token_size": 100,
  "ingest_max_chars": null,
  "eq_query_concurrency": 4,
  "lr_query_concurrency_effective": 1,
  "fairness_note": "Matched top-k=30 budgets; LR rerank off (no model); L2 Evidence Recall + Context Relevancy required for valid smoke+; EQ Mix arm gate off (LR-like always-on local+global+naive) unless EDGEQUAKE_MIX_ARM_GATE=true on the server; optional EDGEQUAKE_MIX_FUSION=round_robin ablation (default rrf); fair Acc ingest: adaptive_chunking off + chunk_token_size=1200 (LightRAG CHUNK_SIZE parity) unless explicitly ablated; smoke-fast Acc may set BENCH001_INGEST_MAX_CHARS for fast force-ingest (full corpus = 0)",
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
