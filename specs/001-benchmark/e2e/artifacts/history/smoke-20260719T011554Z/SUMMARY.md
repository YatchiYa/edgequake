# SPEC-001 smoke SUMMARY

> EQ mix vs LR mix on GraphRAG-Bench (Mistral Small + mistral-embed) — not UltraDomain win-rates, not MMLongBench LVLM.

- **valid:** `False` (empty_context_rate;pre_fix_context_export;faithfulness_not_trustworthy)
- **profile:** `P0_mistral_mix`
- **judge:** `generation_eval`
- **fixture:** `smoke_question_ids_v1` (n=40)
- **dataset revision:** `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546`

## Overall Acc

| SUT | overall_acc |
|-----|-------------|
| EdgeQuake mix | 0.2289 |
| LightRAG mix | 0.2311 |
| Δ (EQ − LR) | -0.0023 |

## By question_type (EQ Acc)

- **Fact Retrieval:** 0.2261
- **Complex Reasoning:** 0.2303
- **Contextual Summarize:** 0.2300
- **Creative Generation:** 0.2291

## By question_type (LR Acc)

- **Fact Retrieval:** 0.2311
- **Complex Reasoning:** 0.2313
- **Contextual Summarize:** 0.2317
- **Creative Generation:** 0.2305

## Ops

- EQ empty-answer rate: 0.000
- LR empty-answer rate: 0.000
- EQ empty-context rate: 1.000
- LR empty-context rate: 1.000
- EQ query p50/p95 ms: 10922 / 24747
- LR query p50/p95 ms: 1817 / 2692
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
  "eq_query_mode": "mix",
  "lr_query_mode": "mix",
  "chunk_size": 1200,
  "retrieve_topk": 5,
  "query_concurrency": 8,
  "eval_concurrency": 8,
  "judge": "generation_eval"
}
```

## Progression

- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`
- This run archives under `specs/001-benchmark/e2e/artifacts/history/`

## Accuracy audit

Prior smoke exported **empty retrieved context** for both SUTs (EQ=100%, LR=100%). Creative `faithfulness=0.0` is not trustworthy. Re-run `--query-only` after harness fix to restore `valid: true`.
