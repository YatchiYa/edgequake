# 004 — Dataset & Fixtures

**Cross-ref:** [002](./002-benchmark-selection.md) · [fixtures/](./fixtures/)

---

## 1. Source pin

| Field | Value |
|-------|-------|
| HF dataset | `GraphRAG-Bench/GraphRAG-Bench` |
| Snapshot revision | `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546` |
| Questions (medical) | `Datasets/Questions/medical_questions.json` (2062) |
| Questions (novel) | `Datasets/Questions/novel_questions.json` (2010) |
| Corpus (medical) | `Datasets/Corpus/medical.json` (1 context, ~1.05M chars) |
| Corpus (novel) | `Datasets/Corpus/novel.json` (20 contexts) |

Local cache root: `~/.cache/edgequake/bench001/` (override with `EDGEQUAKE_BENCH001_CACHE`).

---

## 2. Question schema (official)

```json
{
  "id": "Medical-73586ddc",
  "source": "Medical",
  "question": "...",
  "answer": "...",
  "question_type": "Fact Retrieval",
  "evidence": ["..."],
  "evidence_relations": "..."
}
```

`question_type` ∈ {`Fact Retrieval`, `Complex Reasoning`, `Contextual Summarize`, `Creative Generation`}.

---

## 3. Smoke fixture (`smoke_question_ids_v1.txt`)

| Property | Value |
|----------|-------|
| Corpus | medical only |
| n | **40** |
| Stratification | 10 per `question_type` |
| Seed | **42** |
| Role | Daily / CI plumbing + Acc signal |

File: [`fixtures/smoke_question_ids_v1.txt`](./fixtures/smoke_question_ids_v1.txt)

---

## 4. Core fixture (`core_question_ids_v1.txt`)

| Property | Value |
|----------|-------|
| Corpus | medical full + novel sample 100 |
| n | **2162** (2062 medical + 100 novel) |
| Novel sample | 25 per type × 4, seed 42 |
| Role | Publishable scorecard |
| Gate | `--i-accept-cost` |

File: [`fixtures/core_question_ids_v1.txt`](./fixtures/core_question_ids_v1.txt)

---

## 5. What is committed vs downloaded

| Asset | In git? |
|-------|---------|
| Question ID lists | Yes |
| Full corpus JSON | No (HF download) |
| Full questions JSON | No (HF download) |
| Predictions / scorecards | Artifacts only under `e2e/artifacts/` |

---

## 6. Refresh procedure

```bash
make bench001-freeze-smoke   # re-download + verify fixture IDs still exist
# If upstream IDs change: regenerate with seed 42, bump fixture to _v2, update this pin.
```
