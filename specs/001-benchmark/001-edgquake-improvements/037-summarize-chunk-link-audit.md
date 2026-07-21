# 037 — Horizon B Summarize chunk-link audit (First Principles)

**Status:** Closed — audit complete · law **SELECT** (not LINK starvation)  
**Cross-ref:** [036](./036-a1fp-recall-without-dual-list.md) · [029](./029-ingest-parity-audit.md) · [028](./028-first-principles-beat-roadmap.md) · [001](./001-first-principles.md)  
**Warm WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Acc peer:** [`T095809Z`](../e2e/artifacts/history/smoke-20260720T095809Z/) `a1fp`  
**Artifact:** [`ingest-audit/summarize-20260720T103652Z`](../e2e/artifacts/ingest-audit/summarize-20260720T103652Z/)  
**Tool:** [`tools/bench001/scripts/audit_summarize_chunk_links.py`](../../../tools/bench001/scripts/audit_summarize_chunk_links.py)

---

## 1. Assess (split peers — unchanged)

| Package | Acc | ctx | recall | Fact ER | Sum ER |
|---------|----:|----:|-------:|--------:|-------:|
| **a1fp** T095809Z | **0.775** | **0.500** | 0.926 ✗ | **0.85** | 0.86 |
| a1lrl2 T093152Z | 0.718 | 0.525 | 0.933 ✓ | 0.85 | 0.88 |
| LR (a1fp run) | 0.787 | — | 0.965 | 0.90 | **0.983** |

Binding without dual-list tax: **Contextual Summarize ER 0.86 vs LR 0.98**.  
Disk at audit time: free space varies; **no densify-all re-ingest** until SELECT confound is designed (law below).

---

## 2. First-principles chain

```text
q → entity|relation hit → source_chunk_ids → Mix C → Â → Summarize ER
```

| Law | Necessary condition |
|-----|---------------------|
| **LINK** | Exact-name topic entity: LR has source chunks, EQ has none |
| **SELECT** | Topic entities linked both sides; EQ Mix hits fewer of **q's own content bigrams** than LR |
| **GEN_OR_EVAL** | EQ Mix question-bigram hits not below LR |

**Probes (only):**
1. Exact normalized entity-name pairs from the question text  
2. Verbatim **question content bigrams** inside admitted Mix (e.g. `bone cancers`)

**Forbidden:** `+N` / `%` gates, domain needle bags, token-overlap “gold” fishing.  
GraphRAG-Bench `evidence[]` strings are often **paraphrases** — not corpus probes.

LightRAG keeps `source_id` on entities/relations and expands with `RELATED_CHUNK_NUMBER` (VECTOR/WEIGHT pick). EQ mirrors via `source_chunk_ids` → Mix. Missing links ⇒ LINK; wrong neighborhood in C ⇒ SELECT.

---

## 3. Global hygiene (observables — not a promote gate)

| Side | Entities | Mean chunks/entity | Zero-chunk |
|------|---------:|-------------------:|-----------:|
| EQ (B3b WS) | 4559 | 2.228 | **344** |
| LR smoke | 3580 | 2.204 | **0** |

Identity (B1): AGE/vectors ≈ 1.08. Mean density ≈ LR.  
Zero-chunk count is hygiene debt; it **does not** decide this Summarize miss unless LINK fires on topic names.

---

## 4. Binding miss — `Medical-0002d2de`

**Q:** How are bone cancers staged and what factors are considered in determining the stage?

| Observable | EQ | LR |
|------------|---:|---:|
| Mix parts | **6** | **14** |
| Question bigram `bone cancers` in Mix | **0** | **≥1** |
| `BONE_CANCER` / Bone cancer source chunks | **5** | **6** |
| Exact-pair LINK (EQ empty, LR linked) | **0** | — |

**Law: SELECT**

- `BONE_CANCER` is **BOTH_LINKED** (5 vs 6) — not source_id starvation.  
- EQ Mix heads are off-topic (cervical / anal / AML / bone marrow); **zero** hits on `bone cancers`.  
- LR Mix contains `bone cancer` / TNM neighborhood.

Therefore: **do not** “densify all entity links” as the next Acc lever. The pool for the right entity exists; **Mix admission chose the wrong neighborhood**.

---

## 5. Decision / next confound

| Do | Don't |
|----|-------|
| Next labeled confound: [038](./038-topic-entity-admit-exploratory.md) `TOPIC_ENTITY_ADMIT` / `a1fpsel` | Blind `source_chunk_ids`↑ re-ingest as Acc headline |
| Keep `a1fp` Acc peer + `a1lrl2` L2 Parity (dual-list tax) | Dual-list / LR-budget as Acc headline |
| Optionally fix zero-chunk hygiene on a **separate** ingest workspace when free disk allows | Force-ingest under Acc pins without a SELECT design |

**Promote gates unchanged** (028 / 000): Beat = CI excludes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03; Parity = CI includes 0 ∧ same L2.

---

## 6. Reproduce

```bash
eval "$(rg '^export DATABASE_URL=' /tmp/edgequake-start.sh)"
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
python3 tools/bench001/scripts/audit_summarize_chunk_links.py \
  --predictions-eq specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_eq.json
```
