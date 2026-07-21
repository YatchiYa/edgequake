# 003 — Fair Evaluation Protocol

**Cross-ref:** [001](./001-first-principles.md) · [005](./005-mode-map-and-pins.md) · [010](./010-smoke-then-core-runbook.md)

---

## 1. Stages

```text
prepare → freeze fixtures → doctor
  → ingest same corpus into EQ + LR (isolated)
  → query both with pinned modes (mix / mix)
  → write predictions_eq.json + predictions_lr.json
  → run official generation_eval (L0/L1) + retrieval_eval (L2)
  → emit scorecard.json + SUMMARY.md
```

| Step | Owner | Artifact |
|------|-------|----------|
| Download HF snapshot | `bench001 freeze-smoke` | `~/.cache/edgequake/bench001/` |
| Freeze question IDs | fixtures (committed) | `fixtures/smoke_question_ids_v1.txt` |
| Ingest EQ | REST `POST /api/v1/documents` | workspace `bench001-{stage}` |
| Ingest LR | LightRAG `insert` / example runner | `results/lightrag_workspace/` |
| Query EQ | `POST /api/v1/query` mode=`mix` | `predictions_eq.json` |
| Query LR | LightRAG `aquery` mode=`mix` | `predictions_lr.json` |
| Score (L0/L1) | `generation_eval` or local rouge Acc | `eval_eq.json`, `eval_lr.json` |
| Score (L2) | official `retrieval_eval` | `retrieval_eq.json`, `retrieval_lr.json` |
| Report | `bench001 report` | `scorecard.json`, `SUMMARY.md` |

---

## 2. Prediction record (normative)

Matches GraphRAG-Bench eval input (normalize field names in harness):

```json
{
  "id": "Medical-73586ddc",
  "question": "...",
  "source": "medical",
  "context": ["retrieved context string(s)"],
  "evidence": ["gold evidence span"],
  "question_type": "Fact Retrieval",
  "generated_answer": "...",
  "ground_truth": "...",
  "gold_answer": "..."
}
```

`ground_truth` and `gold_answer` must be identical copies of the official `answer` field.

---

## 3. Blind query rules

1. Query string = official `question` only.
2. Never pass gold `evidence` / `evidence_relations` into the retriever.
3. Never filter documents to evidence-only pages (there are none — text corpora).
4. Temperature 0 for query + judge when supported.

---

## 4. Resume & cache

- **Index once:** smoke/core reuse workspaces unless `--force-ingest`.
- **`--query-only`:** skip ingest; fail if workspace empty.
- **`--dry-run`:** synthetic answers from gold tokens for harness plumbing; scorecard `valid: false`, `invalid_reason: dry_run`. Writes to `e2e/artifacts/<stage>-dry-run/` — **never** overwrites live `smoke/` / `core/` artifacts.
- **`--max-questions N`:** debug truncate; writes to `e2e/artifacts/<stage>-debug/` and forces `valid: false`.

---

## 5. Validity gates

A run is `valid: true` only if all hold:

1. Dataset revision matches pin in [004](./004-dataset-and-fixtures.md).
2. Both SUTs completed ingest (or query-only with prior ingest).
3. Empty-answer rate ≤ **5%** per SUT when publish fairness is on (default); else ≤ 10% smoke / ≤ 5% core.
4. Empty-context rate ≤ **5%** per SUT when publish fairness is on — retrieved context required for Faithfulness + L2.
5. Judge is `generation_eval` (not dry-run rouge-only).
6. Mode pins are EQ `mix` + LR `mix` for headline profile `P0_mistral_mix_v2`.
7. **L2** `retrieval_eval` succeeded for both SUTs (`metrics.*.retrieval` present) when `BENCH001_PUBLISH_FAIRNESS=1`.
8. Matched retrieval budget pinned (`retrieve_topk` / `lr_top_k` / `eq_max_results`, default **30**).

### Parallelism (speed, fair pins)

| Layer | Mechanism | Pin |
|-------|-----------|-----|
| Query EQ | `ThreadPoolExecutor` | `BENCH001_QUERY_CONCURRENCY` / `--query-concurrency` (default 8) |
| Query LR | `asyncio.Semaphore` | same pin (fair EQ↔LR) |
| Query dual-SUT | EQ ∥ LR processes | always when both live |
| Judge | official `--max_concurrent` | `BENCH001_EVAL_CONCURRENCY` / `--eval-concurrency` (default 8) |
| Score dual-SUT | EQ ∥ LR `generation_eval` | always |

Never raise concurrency by dropping pins or mixing different top-k / modes per SUT.

### Progression

- Live phase ticks: `e2e/artifacts/<stage>/progress.json`
- Ladder ledger: `e2e/artifacts/PROGRESS.md`
- Immutable archives: `e2e/artifacts/history/<stage>-<utc>/`
- Compare: `python3 -m bench001.cli report smoke --compare history/<run>`

---

## 6. Cost gates

| Stage | Flag | Default |
|-------|------|---------|
| smoke | none | allowed |
| core | `--i-accept-cost` | required |
