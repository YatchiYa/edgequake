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
- **judge_metric_embed:** `BAAI/bge-large-en-v1.5`
- **llm_base_url:** `https://api.mistral.ai/v1`
- **judge_base_url:** `https://api.mistral.ai/v1`

## Overall Acc

| SUT | overall_acc |
|-----|-------------|
| EdgeQuake mix | 0.2280 |
| LightRAG mix | 0.2303 |
| Δ (EQ − LR) | -0.0023 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 0.2288
- **Complex Reasoning:** 0.2292
- **Contextual Summarize:** 0.2315
- **Creative Generation:** 0.2223

## By question_type (LR Acc)

- **Fact Retrieval:** 0.2351
- **Complex Reasoning:** 0.2293
- **Contextual Summarize:** 0.2327
- **Creative Generation:** 0.2239

## Ops

- EQ empty-answer rate: 0.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 0.000
- LR empty-context rate: 0.000
- EQ query p50/p95 ms: 11446 / 14233
- LR query p50/p95 ms: 2930 / 3272
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
  "judge_embedding_model": "BAAI/bge-large-en-v1.5",
  "lineage": {
    "sut_llm": "mistral/mistral-small-latest",
    "sut_vision": "mistral/mistral-small-latest",
    "sut_embed": "mistral/mistral-embed@1024d",
    "judge_llm": "mistral/mistral-small-latest",
    "judge_metric_embed": "BAAI/bge-large-en-v1.5",
    "llm_base_url": "https://api.mistral.ai/v1",
    "judge_base_url": "https://api.mistral.ai/v1"
  },
  "eq_query_mode": "mix",
  "lr_query_mode": "mix",
  "chunk_size": 1200,
  "retrieve_topk": 5,
  "query_concurrency": 12,
  "eval_concurrency": 12,
  "judge": "generation_eval"
}
```

## Progression

- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`
- This run archives under `specs/001-benchmark/e2e/artifacts/history/`
