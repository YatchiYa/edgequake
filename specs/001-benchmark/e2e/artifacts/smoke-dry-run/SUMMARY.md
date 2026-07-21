# SPEC-001 smoke-dry-run SUMMARY

> EQ mix vs LR mix on GraphRAG-Bench (Mistral Small + mistral-embed) — not UltraDomain win-rates, not MMLongBench LVLM.

- **valid:** `False` (dry_run;judge:rouge_proxy;max_questions_truncate)
- **profile:** `P0_mistral_mix`
- **judge:** `rouge_proxy`
- **fixture:** `smoke_question_ids_v1` (n=1)
- **dataset revision:** `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`

## Model lineage

- **sut_llm:** `mistral/mistral-small-latest`
- **sut_vision:** `mistral/mistral-small-latest`
- **sut_embed:** `mistral/mistral-embed@1024d`
- **judge_llm:** `openai/gpt-4o-mini`
- **judge_metric_embed:** `BAAI/bge-large-en-v1.5`
- **llm_base_url:** `https://api.mistral.ai/v1`
- **judge_base_url:** `https://api.openai.com/v1`

## Overall Acc

| SUT | overall_acc |
|-----|-------------|
| EdgeQuake mix | 1.0000 |
| LightRAG mix | 1.0000 |
| Δ (EQ − LR) | +0.0000 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 1.0000
- **Complex Reasoning:** 0.0000
- **Contextual Summarize:** 0.0000
- **Creative Generation:** 0.0000

## By question_type (LR Acc)

- **Fact Retrieval:** 1.0000
- **Complex Reasoning:** 0.0000
- **Contextual Summarize:** 0.0000
- **Creative Generation:** 0.0000

## Ops

- EQ empty-answer rate: 0.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 0.000
- LR empty-context rate: 0.000
- EQ query p50/p95 ms: 0 / 0
- LR query p50/p95 ms: 0 / 0
- ingest wall s: 0.0

## Pins

```json
{
  "edgequake_git_sha": "936b9236",
  "dataset_id": "GraphRAG-Bench/GraphRAG-Bench",
  "dataset_revision": "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546",
  "fixture_id": "smoke_question_ids_v1",
  "profile_id": "P0_mistral_mix",
  "llm_provider": "mistral",
  "llm_model": "mistral-small-latest",
  "vision_provider": "mistral",
  "vision_model": "mistral-small-latest",
  "embedding_provider": "mistral",
  "embedding_model": "mistral-embed",
  "embedding_dim": 1024,
  "llm_base_url": "https://api.mistral.ai/v1",
  "judge_provider": "openai",
  "judge_model": "gpt-4o-mini",
  "judge_base_url": "https://api.openai.com/v1",
  "judge_embedding_model": "BAAI/bge-large-en-v1.5",
  "lineage": {
    "sut_llm": "mistral/mistral-small-latest",
    "sut_vision": "mistral/mistral-small-latest",
    "sut_embed": "mistral/mistral-embed@1024d",
    "judge_llm": "openai/gpt-4o-mini",
    "judge_metric_embed": "BAAI/bge-large-en-v1.5",
    "llm_base_url": "https://api.mistral.ai/v1",
    "judge_base_url": "https://api.openai.com/v1"
  },
  "eq_query_mode": "mix",
  "lr_query_mode": "mix",
  "chunk_size": 1200,
  "retrieve_topk": 5,
  "query_concurrency": 8,
  "eval_concurrency": 8,
  "judge": "rouge_proxy"
}
```

## Progression

- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`
- This run archives under `specs/001-benchmark/e2e/artifacts/history/`
