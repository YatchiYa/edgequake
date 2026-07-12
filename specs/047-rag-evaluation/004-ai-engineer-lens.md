# 004 — AI Engineer Lens

**Cross-ref:** [001](./001-first-principles.md) · [003](./003-fair-evaluation-protocol.md) · SPEC-046 query compare · `edgequake-query` hybrid path

---

## 1. System under test (as a retrieval physicist)

```text
  PDF bytes
     │
     ▼
  Vision parse (mistral-small-latest) ──▶ markdown + page structure
     │
     ▼
  Chunk + embed (mistral-embed 1024-d) + entity/rel extract (mistral-small)
     │
     ▼
  pgvector chunks/entities/rels + AGE graph
     │
     ▼
  Query hybrid: local ∥ global ∥ naive ──▶ context ──▶ LLM answer
```

**Hypothesis H1:** Hybrid GraphRAG beats naive-only on cross-page slices.  
**Hypothesis H2:** Vision ingest beats text-only OCR on chart/image evidence sources.  
**Hypothesis H3:** Unanswerable Acc correlates with refusal prompt + retrieval emptiness, not with model size alone.

Every primary run must be able to support or refute H1–H3 via slices + optional ablations.

---

## 2. Failure mode taxonomy (design the harness to catch these)

| ID | Failure | Symptom on MMLongBench-Doc | Probe |
|----|---------|----------------------------|-------|
| F1 | Vision silent drop | Chart/image Acc ≈ 0; text Acc OK | Compare vision vs forced EdgeParse |
| F2 | Chunk boundary loss | Single-page table Acc low | Inspect retrieved chunks vs gold page |
| F3 | Embedding mismatch | Global nonsense retrieval | Confirm 1024-d + same model at ingest/query |
| F4 | Graph under-extraction | Cross-page Acc << single-page | Entity count / relation density per doc |
| F5 | Hybrid merge dilution | Naive-only better than hybrid | Ablation modes: naive, local, global, hybrid, mix |
| F6 | Hallucinated answerable | Unanswerable Acc low | Rate of non-“Not answerable” preds on neg Qs |
| F7 | Extractor bias | Acc swings when swapping GPT-4o ↔ Mistral judge | Dual-extractor smoke |
| F8 | Context truncation | Long answers missing numbers | Log context token counts |

---

## 3. Ablation matrix (ordered)

Run after smoke is green; label each `profile_id`:

| Profile | Change | Purpose |
|---------|--------|---------|
| `P0_primary` | hybrid + vision Small + embed | Headline |
| `P1_naive` | mode=`naive` | Vector-only floor |
| `P2_local` | mode=`local` | Entity-centric |
| `P3_global` | mode=`global` | Relation-centric |
| `P4_mix` | mode=`mix` | EQ hybrid+naive |
| `P5_text_parse` | vision off / EdgeParse | Vision value |
| `P6_oracle_pages` | filter to gold pages | Retrieval ceiling |

Primary publication uses **P0 only**. Ablations go in `artifacts/{stage}/ablations/`.

---

## 4. Context & limits (Mistral)

From [Mistral Vision docs](https://docs.mistral.ai/studio-api/conversations/vision):

- Max **8 images / request** — cannot naively send 47 page images to Small for LVLM-style eval.  
- Max resolution ~1540×1540 for Small 3.2 family.  
- `mistral-embed`: **1024-d**, ~8192 token input budget per request (EdgeQuake already batches).

These limits **justify** RAG adaptation rather than full-page LVLM replay.

---

## 5. Retrieval diagnostics worth logging

Per question JSONL row:

```json
{
  "doc_id": "...",
  "question_id": "...",
  "mode": "hybrid",
  "answer_long": "...",
  "answer_short": "...",
  "score": 0.0,
  "gold_pages": [3, 5],
  "retrieved_sources": [{"chunk_id": "...", "page": 3, "score": 0.42}],
  "page_hit": true,
  "latency_ms": 1820,
  "prompt_tokens": 3500
}
```

`page_hit` is best-effort (only if page metadata exists post-ingest).

---

## 6. AI Engineer acceptance

- [ ] H1–H3 can be tested from artifacts without re-querying  
- [ ] Ablation profiles are one flag away  
- [ ] No gold leakage in P0  
- [ ] Vision fail-closed enforced  

Next: [005 Expert Developer](./005-expert-developer-lens.md).
