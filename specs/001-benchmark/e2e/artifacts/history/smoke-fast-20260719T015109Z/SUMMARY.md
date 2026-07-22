# SPEC-001 smoke-fast SUMMARY

> EQ mix vs LR mix on GraphRAG-Bench (Mistral Small + mistral-embed) — not UltraDomain win-rates, not MMLongBench LVLM.

- **valid:** `True`
- **profile:** `P0_mistral_mix`
- **judge:** `generation_eval`
- **fixture:** `smoke_fast_question_ids_v1` (n=8)
- **dataset revision:** `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`

## Model lineage

- **sut_llm:** `mistral/mistral-small-latest`
- **sut_vision:** `mistral/mistral-small-latest`
- **sut_embed:** `mistral/mistral-embed@1024d`
- **judge_llm:** `mistral/mistral-small-latest`
- **judge_metric_embed:** `mistral-embed`
- **llm_base_url:** `https://api.mistral.ai/v1`
- **judge_base_url:** `https://api.mistral.ai/v1`

## Overall Acc

| SUT | overall_acc |
|-----|-------------|
| EdgeQuake mix | 0.2412 |
| LightRAG mix | 0.2375 |
| Δ (EQ − LR) | +0.0037 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 0.2411
- **Complex Reasoning:** 0.2419
- **Contextual Summarize:** 0.2446
- **Creative Generation:** 0.2373

## By question_type (LR Acc)

- **Fact Retrieval:** 0.2367
- **Complex Reasoning:** 0.2386
- **Contextual Summarize:** 0.2421
- **Creative Generation:** 0.2325

## Ops

- EQ empty-answer rate: 0.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 0.000
- LR empty-context rate: 0.000
- EQ query p50/p95 ms: 5219 / 9306
- LR query p50/p95 ms: 5769 / 9173
- ingest wall s: 0.0

## Pins

```json
{
  "edgequake_git_sha": "936b9236",
  "dataset_id": "GraphRAG-Bench/GraphRAG-Bench",
  "dataset_revision": "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546",
  "fixture_id": "smoke_fast_question_ids_v1",
  "profile_id": "P0_mistral_mix",
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
  "answer_style": "concise",
  "acc_formula": "0.75*F1 + 0.25*embed_cosine (weights overridable)",
  "eq_query_mode": "mix",
  "lr_query_mode": "mix",
  "chunk_size": 1200,
  "retrieve_topk": 5,
  "query_concurrency": 4,
  "eval_concurrency": 4,
  "judge": "generation_eval"
}
```

## Progression

- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`
- This run archives under `specs/001-benchmark/e2e/artifacts/history/`
