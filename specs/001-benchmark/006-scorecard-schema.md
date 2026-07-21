# 006 — Scorecard Schema

**Cross-ref:** [003](./003-fair-evaluation-protocol.md) · [012 SPEC-047 pattern](../047-rag-evaluation/012-acceptance-criteria-and-scorecard.md)

---

## 1. Normative `scorecard.json`

```json
{
  "spec": "001",
  "stage": "smoke",
  "valid": true,
  "invalid_reason": null,
  "task_name": "GraphRAG-Bench/EQ-vs-LR",
  "banner": "EQ mix vs LR mix on GraphRAG-Bench — not UltraDomain win-rates, not MMLongBench LVLM.",
  "profile_id": "P0_mistral_mix",
  "pins": {
    "edgequake_git_sha": "...",
    "dataset_id": "GraphRAG-Bench/GraphRAG-Bench",
    "dataset_revision": "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546",
    "fixture_id": "smoke_question_ids_v1",
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
    "eq_query_mode": "mix",
    "lr_query_mode": "mix",
    "chunk_size": 1200,
    "retrieve_topk": 5,
    "judge": "generation_eval"
  },
  "metrics": {
    "eq": {
      "overall_acc": 0.0,
      "overall_f1": 0.0,
      "overall_cos": 0.0,
      "by_type": {
        "Fact Retrieval": {
          "answer_correctness": 0.0,
          "factuality_f1": 0.0,
          "embed_cosine": 0.0,
          "rouge_score": 0.0
        },
        "Complex Reasoning": {"answer_correctness": 0.0, "factuality_f1": 0.0, "embed_cosine": 0.0, "rouge_score": 0.0},
        "Contextual Summarize": {"answer_correctness": 0.0, "factuality_f1": 0.0, "embed_cosine": 0.0, "coverage_score": 0.0},
        "Creative Generation": {"answer_correctness": 0.0, "factuality_f1": 0.0, "embed_cosine": 0.0, "coverage_score": 0.0, "faithfulness": 0.0}
      }
    },
    "lr": { "overall_acc": 0.0, "overall_f1": 0.0, "overall_cos": 0.0, "by_type": {} },
    "delta_eq_minus_lr": {
      "overall_acc": 0.0,
      "overall_f1": 0.0,
      "overall_cos": 0.0,
      "overall_acc_delta_ci": { "mean": 0.0, "ci_low": 0.0, "ci_high": 0.0, "n": 0 }
    }
  },
  "ops": {
    "n_questions": 40,
    "eq_empty_answer_rate": 0.0,
    "lr_empty_answer_rate": 0.0,
    "eq_empty_context_rate": 0.0,
    "lr_empty_context_rate": 0.0,
    "eq_query_p50_ms": 0,
    "eq_query_p95_ms": 0,
    "lr_query_p50_ms": 0,
    "lr_query_p95_ms": 0,
    "ingest_wall_s": 0,
    "acc_components_present": true
  },
  "created_at_utc": "2026-07-18T00:00:00Z"
}
```

---

## 2. Required keys

`spec`, `stage`, `valid`, `task_name`, `banner`, `profile_id`, `pins`, `metrics`, `ops`, `created_at_utc`

`pins` must include SUT + judge lineage: `llm_*`, `vision_*`, `embedding_*`, `judge_provider`, `judge_model`, `judge_base_url`, `judge_embedding_model`, `lineage`, `eq_query_mode`, `lr_query_mode`, `dataset_revision`, `fixture_id`, `profile_id`.

---

## 3. Acceptance gates

### Smoke

- Schema-valid `scorecard.json` + `SUMMARY.md`
- Fixture = `smoke_question_ids_v1` (n=40)
- Dry-run allowed for plumbing with `valid: false`
- Live dual-SUT: `valid: true` only with both SUTs + official judge

### Core

- `--i-accept-cost`
- Fixture = `core_question_ids_v1`
- Both corpora ingested
- Side-by-side Acc deltas reported

---

## 4. `SUMMARY.md` must include

1. Honesty banner  
2. Pins table  
3. Overall Acc EQ vs LR + delta  
4. Per-`question_type` Acc  
5. Ops (empty-answer, empty-context, latency)  
6. Path to raw `predictions_*.json` / `eval_*.json`  
7. Pointer to progression ledger (`PROGRESS.md` / `history/`)
