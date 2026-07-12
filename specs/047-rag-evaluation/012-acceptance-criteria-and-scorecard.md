# 012 — Acceptance Criteria & Scorecard Schema

**Cross-ref:** [003](./003-fair-evaluation-protocol.md) · [009](./009-implementation-plan.md) · [010](./010-smoke-then-full-runbook.md) · [022](./022-reassessment-2026-07-11.md)

---

## 1. Spec-pack acceptance (this delivery)

| Criterion | Status |
|-----------|--------|
| Specs under `specs/047-rag-evaluation/` with `NNN-*.md` pattern | ✅ |
| First principles + fair protocol documented | ✅ |
| Multi-lens coverage (AI Eng, Dev, MLOps, ML, Product/SRE) | ✅ |
| Smoke → core → full progression defined | ✅ |
| Mistral Small LLM+vision + mistral-embed + hybrid locked | ✅ |
| Real documents + official scoring required | ✅ |
| Complementary benchmarks + methodology | ✅ |
| Easy run / easy eval UX specified | ✅ |
| Implementation tickets listed | ✅ |
| Harness code implemented (`tools/bench047`, `make bench047-*`) | ✅ |
| First valid smoke scorecard (Postgres + Small + hybrid) | ✅ 2026-07-10 |
| Query-lane Acc band locked (chart fixture dscope) | ✅ ~0.43 · see [022](./022-reassessment-2026-07-11.md) |

---

## 2. Implementation acceptance

### Smoke gate (`bench047-smoke`)

- [x] `valid: true`
- [x] `ingest_coverage >= 0.9`
- [x] Scorecard on chart fixture (8 docs) **or** classic 10-doc smoke — label fixture in meta
- [x] `scorecard.json` schema-valid
- [x] `SUMMARY.md` contains RAG banner + Acc + F1 + slices
- [x] Vision / empty-answer misconfig → `valid: false` (fail closed)
- [x] Scorer unit tests green

### How to read Acc / F1

**Authoritative locked chain (2026-07-11):** [022](./022-reassessment-2026-07-11.md) — Acc **0.384 → 0.436 → 0.429 → 0.427**, Unans **≥0.81**, Chart **~0.14**.

#### Historical early smoke (2026-07-10, different fixture)

| Metric | First valid | W0 query-only re-score | How to interpret |
|--------|-------------|------------------------|------------------|
| Acc | **0.45** | **0.41** | LLM variance; not a regression gate alone |
| F1 | **0.29** | **0.26** | Same |
| Unanswerable Acc | **0.89** | **0.78** | Protect ≥~0.70 under dscope HEAD (held 0.83) |
| Chart Acc | **0.05** | **0.05** | Still weakest slice → **015** |
| **page_hit@5** (answerable) | — | **0.59** | W0 retrieval law (G2); dscope HEAD ~0.76 |
| **answer_in_evidence** | — | **0.51** | W1a fidelity; Chart **0.36** |
| False refusal | ~50% | **47%** | Calibrate; never ban NA |
| LVLM GPT-4o F1 | ≈**0.45** | — | **Difficulty reference only** |

**Causal split (code is law):** ~half of answerable golds never appear in evidence-page markdown (W1). Of the rest, some miss retrieve (W2) and some miss generate despite hit (W3). Do not prompt-patch Acc. Query lane closed most of the Gen/scope gap; **Chart Rep remains the Acc ceiling.**

### Core / Full gates

- [ ] Same schema
- [ ] Cost acknowledgement flag used
- [ ] Resume works
- [ ] Dataset revision pinned in meta

---

## 3. `scorecard.json` schema (normative)

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "EdgeQuakeBench047Scorecard",
  "type": "object",
  "required": [
    "spec", "stage", "valid", "task_name", "banner",
    "pins", "metrics", "slices", "ops", "created_at_utc"
  ],
  "properties": {
    "spec": { "const": "047" },
    "stage": { "enum": ["smoke", "core", "full", "ablation"] },
    "valid": { "type": "boolean" },
    "invalid_reason": { "type": ["string", "null"] },
    "task_name": {
      "const": "MMLongBench-Doc/RAG-adaptation"
    },
    "banner": {
      "type": "string",
      "description": "Mandatory honesty note about RAG vs LVLM"
    },
    "pins": {
      "type": "object",
      "required": [
        "edgequake_version", "edgequake_git_sha",
        "dataset_id", "dataset_revision",
        "fixture_id", "llm_provider", "llm_model",
        "vision_provider", "vision_model",
        "embedding_provider", "embedding_model",
        "embedding_dim", "query_mode",
        "extractor_model", "eval_score_sha"
      ],
      "properties": {
        "edgequake_version": { "type": "string" },
        "edgequake_git_sha": { "type": "string" },
        "dataset_id": { "type": "string" },
        "dataset_revision": { "type": "string" },
        "fixture_id": { "type": "string" },
        "llm_provider": { "type": "string" },
        "llm_model": { "type": "string" },
        "vision_provider": { "type": "string" },
        "vision_model": { "type": "string" },
        "embedding_provider": { "type": "string" },
        "embedding_model": { "type": "string" },
        "embedding_dim": { "type": "integer", "const": 1024 },
        "query_mode": { "const": "hybrid" },
        "extractor_model": { "type": "string" },
        "eval_score_sha": { "type": "string" },
        "system_prompt_sha": { "type": "string" },
        "profile_id": { "type": "string" }
      }
    },
    "metrics": {
      "type": "object",
      "required": ["n_docs", "n_questions", "n_scored", "accuracy", "f1"],
      "properties": {
        "n_docs": { "type": "integer" },
        "n_questions": { "type": "integer" },
        "n_scored": { "type": "integer" },
        "n_skipped_ingest_failed": { "type": "integer" },
        "accuracy": { "type": "number" },
        "f1": { "type": "number" }
      }
    },
    "slices": {
      "type": "object",
      "properties": {
        "single_page_accuracy": { "type": "number" },
        "cross_page_accuracy": { "type": "number" },
        "unanswerable_accuracy": { "type": "number" },
        "by_evidence_source": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "properties": {
              "accuracy": { "type": "number" },
              "n": { "type": "integer" }
            }
          }
        },
        "by_doc_type": {
          "type": "object",
          "additionalProperties": {
            "type": "object",
            "properties": {
              "accuracy": { "type": "number" },
              "n": { "type": "integer" }
            }
          }
        }
      }
    },
    "ops": {
      "type": "object",
      "required": ["ingest_coverage"],
      "properties": {
        "ingest_coverage": { "type": "number" },
        "cost_usd_total": { "type": ["number", "null"] },
        "p50_query_latency_ms": { "type": ["number", "null"] },
        "p95_query_latency_ms": { "type": ["number", "null"] },
        "answer_empty_rate": { "type": ["number", "null"] },
        "extractor_fail_rate": { "type": ["number", "null"] },
        "page_hit_rate": { "type": ["number", "null"] }
      }
    },
    "created_at_utc": { "type": "string", "format": "date-time" }
  }
}
```

---

## 4. Example (illustrative only — not real scores)

```json
{
  "spec": "047",
  "stage": "smoke",
  "valid": true,
  "invalid_reason": null,
  "task_name": "MMLongBench-Doc/RAG-adaptation",
  "banner": "EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to LVLM leaderboard without caveats. GPT-4o LVLM F1≈44.9% is difficulty reference only.",
  "pins": {
    "edgequake_version": "0.16.0",
    "edgequake_git_sha": "abc12345",
    "dataset_id": "yubo2333/MMLongBench-Doc",
    "dataset_revision": "main@TBD",
    "fixture_id": "smoke_doc_ids_v1",
    "llm_provider": "mistral",
    "llm_model": "mistral-small-latest",
    "vision_provider": "mistral",
    "vision_model": "mistral-small-latest",
    "embedding_provider": "mistral",
    "embedding_model": "mistral-embed",
    "embedding_dim": 1024,
    "query_mode": "hybrid",
    "extractor_model": "gpt-4o",
    "eval_score_sha": "TBD",
    "system_prompt_sha": "TBD",
    "profile_id": "P0_primary"
  },
  "metrics": {
    "n_docs": 10,
    "n_questions": 72,
    "n_scored": 70,
    "n_skipped_ingest_failed": 2,
    "accuracy": 0.0,
    "f1": 0.0
  },
  "slices": {
    "single_page_accuracy": 0.0,
    "cross_page_accuracy": 0.0,
    "unanswerable_accuracy": 0.0,
    "by_evidence_source": {},
    "by_doc_type": {}
  },
  "ops": {
    "ingest_coverage": 0.9,
    "cost_usd_total": null,
    "p50_query_latency_ms": null,
    "p95_query_latency_ms": null,
    "answer_empty_rate": null,
    "extractor_fail_rate": null,
    "page_hit_rate": null
  },
  "created_at_utc": "2026-07-10T00:00:00Z"
}
```

---

## 5. Progression tracking table (keep in SUMMARY)

| Stage | Date | EQ version | F1 | Acc | ingest_cov | Notes |
|-------|------|------------|----|-----|------------|-------|
| smoke | 2026-07-10 | 0.16.0 | 0.294 | 0.452 | 1.00 | First **valid** P0 (Small+embed+hybrid+Postgres). Prior crash run INVALID. |
| smoke | 2026-07-10 | 0.16.0 | 0.154 | 0.417 | 1.00 | **P0_mm_ite** chart-doc (1/8), MV-23+32, `page_hit@5=1.0`, Chart fidelity **0.50** (was 0.36). 10 query workers. |
| core | | | | | | |
| full | | | | | | |

Update this table after every valid run — this is how “we see the progression.”

**Improvement priorities from smoke slices:** chart → cross-page → figure → pure-text/table. Unanswerable is already strong.

---

## 6. Citations (required in every published SUMMARY)

```text
Ma et al., MMLongBench-Doc: Benchmarking Long-context Document Understanding
with Visualizations, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
```
