# 001 — First Principles (Fair Dual-SUT HybridRAG Eval)

**Cross-ref:** [000 INDEX](./000-index.md) · [002 Selection](./002-benchmark-selection.md) · [003 Protocol](./003-fair-evaluation-protocol.md)

---

## 1. What are we measuring?

```text
  Question (GraphRAG-Bench gold)
           |
           +------------------+
           v                  v
  +-----------------+  +-----------------+
  | EdgeQuake mix   |  | LightRAG mix    |
  | (SUT A)         |  | (SUT B)         |
  +--------+--------+  +--------+--------+
           |                    |
           v                    v
     predictions_eq.json   predictions_lr.json
           |                    |
           +---------+----------+
                     v
           Official generation_eval
                     |
                     v
              scorecard.json (side-by-side)
```

**Axiom A1 — Task identity must be named honestly.**  
We measure:

> *Given the same GraphRAG-Bench corpus and questions, with the same LLM/embed pins, how does EdgeQuake `mix` compare to LightRAG `mix` under the official GraphRAG-Bench generation metrics?*

Call this **`GraphRAG-Bench/EQ-vs-LR`**, never “UltraDomain win-rate” or “MMLongBench LVLM score.”

---

## 2. Irreducible requirements

| ID | Principle | Violation = invalid run |
|----|-----------|-------------------------|
| P1 | **Same corpus** | Both SUTs ingest the identical GraphRAG-Bench text (medical / novel) |
| P2 | **Same questions** | Frozen fixture IDs; no post-hoc cherry-picking |
| P3 | **Same judge** | Official `generation_eval` (or documented dry-run proxy labeled `valid:false` for plumbing) |
| P4 | **Mode map explicit** | Headline = EQ `mix` ↔ LR `mix` only; other modes = labeled ablations |
| P5 | **Pinned system profile (parameterized)** | Defaults = Mistral Small + mistral-embed; LLM/vision/embed/**judge** are CLI/env parameters; full `pins.lineage` required in scorecard |
| P6 | **Isolation** | Separate EQ workspace + LR working dir; no cross-contamination |
| P7 | **Reproducibility** | Seeded smoke IDs, dataset revision, command log, artifact hashes |
| P8 | **Fail closed** | Ingest failure, empty answers, missing keys → `valid: false` |
| P9 | **Retrieved context exported** | Predictions must carry non-empty `context` used at generation time — required for Creative Faithfulness and `retrieval_eval`. Empty context → `valid: false` |
| P10 | **Progression is observable** | Smoke → core ladder archived under `e2e/artifacts/history/` + `PROGRESS.md`; each run writes `progress.json` phase ticks |
| P11 | **Matched retrieval budget** | Both SUTs use the same top-k / max_results pin (default **30**, paper H.2). Scorecard records `retrieve_topk` / `lr_top_k` / `eq_max_results`. |
| P12 | **L2 retrieval required (publish)** | Official `retrieval_eval` (Evidence Recall + Context Relevancy) must succeed for both SUTs on smoke/core when `BENCH001_PUBLISH_FAIRNESS=1` (default). Acc alone is not a publishable RAG claim (2026 RAG Triad). |

---

## 3. Why dual-SUT on GraphRAG-Bench

1. **LightRAG is an official published baseline** on this suite — numbers are comparable, not invented.
2. **Gold answers exist** — Acc / ROUGE are more reliable than UltraDomain pairwise LLM-judge win-rates.
3. **Four difficulty levels** stress when graphs help (Fact → Reasoning → Summarize → Creative) — aligns with SPEC-046 routing lessons.
4. **Smokeable** — stratified 40 questions on medical keeps the loop quick.

---

## 4. Fairness constraints

```text
FORBIDDEN                                              REQUIRED
---------                                              --------
• Different corpora per SUT                            • Byte-identical context ingest
• Mixing EQ hybrid vs LR mix in one headline           • Explicit mode map (005)
• Feeding gold evidence into the retriever             • Blind query: question text only
• Editing questions / answers                          • Exact official strings
• Softening empty answers into scored zeros            • Fail closed → valid:false
• Publishing smoke as release score                    • Smoke → medical-mid → core ladder
• Claiming UltraDomain win-rates from this SPEC        • Task name GraphRAG-Bench/EQ-vs-LR
• Publishing n=40 Acc as the stakeholder score         • Publish Acc = medical-mid n=200 (`make bench`)
```

---

## 5. Metric layers

| Layer | Metrics | Role |
|-------|---------|------|
| **L0 Headline** | Acc (`answer_correctness`) by `question_type` + overall | Dual-SUT comparison |
| **L1 Generation** | ROUGE-L, Coverage, Faithfulness (per type) | Official GraphRAG-Bench dims |
| **L2 Retrieval (required for publish)** | Context relevancy, evidence recall | Official `retrieval_eval` |
| **L3 Ops** | Ingest wall, query p50/p95, empty-answer rate, $/run | Operability |

Never optimize L3 at the expense of L0/L2 validity.

**Publish claim law:** A scorecard may say “publishable dual-SUT” only when `valid: true`, profile `P0_mistral_mix_v2` (or labeled ablation), L0+L1+L2 present, and Acc is **not** compared to paper Table-2 without a `P0_paper` pin set.

---

## 6. Larger model + judgment parallelism (P12 / P13)

| Principle | Rule |
|-----------|------|
| **P12 Fair larger SUT** | Upsize LLM on **both** EQ and LR with the same pin (`mistral-large-latest`). Embed stays `mistral-embed`. Label profile `P0_mistral_large_mix_v2`. Do not mix small SUT + large judge in a headline Δ. |
| **P13 Judgment wall-time** | Acc/L2 are post-hoc on frozen predictions — parallelize freely: EQ∥LR scoring, `generation_eval`∥`retrieval_eval`, samples via `--eval-concurrency` (default **16**), question types via `asyncio.gather`. Never change metric definitions for speed. |

Target: `make bench001-smoke-fast-large` (large SUT+judge, eval concurrency 24).

---

## 7. Acc-lift protocol (P14) — fair magnitude under Mistral

| Principle | Rule |
|-----------|------|
| **P14 Acc shape then content** | Acc ≈ 0.75·F1 + 0.25·cos. Fix shape first (`--answer-style gold`, no citation markers), then same-family stronger SUT+judge (`mistral-medium-latest` on **both** SUTs). Keep embed/top-k/L2 fixed. |
| **Parallel judgment** | Use `make bench001-smoke-fast-acc` / `make bench001-smoke-acc` (eval∥=24, gen∥retrieval, qtypes∥). Stale `BENCH001_EVAL_CONCURRENCY=4` in shell must not win. |
| **Read Acc honestly** | Flat Acc ≈ 0.24 with rising ROUGE means F1≈0 (wrong facts or strict statement match) and score ≈ 0.25·cos. Do not claim paper Table-2 without `P0_paper`. |

Empirically (2026-07-19): baseline small/concise smoke Acc 0.241; medium/gold Acc 0.241 with ROUGE↑ and score wall 136s→51s.

---

## 8. Acc metric adaptation (P15) — ML engineering

**Law:** Adapt the *measurement stack* to the new pins. Do **not** retarget Acc weights, gold labels, or L2 gates to inflate the scalar.

| Principle | Practice |
|-----------|----------|
| **Construct validity** | Acc = factual equivalence to gold under official formula; style/citations are SUT pins, not metric edits |
| **Decompose** | Always report Acc **and** statement-F1 **and** embed-cos (plus ROUGE, L2). Flat Acc≈0.24 + high cos ⇒ F1≈0 |
| **Dual track** | **A** `P0_mistral_*` for product decisions · **B** `P0_paper` (GPT-4o-mini + BGE) for Table-2 — never mix claims |
| **Embed coherence** | Cosine pin declared: `mistral-embed` with Mistral SUT; BGE only on paper track |
| **Calibrate judge** | Human labels on stratified smoke; agreement / sensitivity-specificity; optional bias-corrected Acc + CI |
| **One confound** | Named `profile_id` + `pins.lineage`; change one of {style, judge, SUT LLM, embed} per ablation |
| **Stats** | n=8 gate; n=40+ for claims; bootstrap CI on Δ; judge temperature=0 |

**Adapted Acc checklist:** (1) freeze formula, (2) fair pins + gold style, (3) export F1/cos components, (4) calibrate judge, (5) dual headline. See canvas `acc-metric-adaptation`.

**Publishable Acc requirements (harness-enforced when `BENCH001_PUBLISH_FAIRNESS=1`):**

| Gate | Command / field |
|------|-----------------|
| Acc components | Scorecard must include `overall_f1` + `overall_cos` (else `acc_components_missing`) |
| Instrument canary | `make bench001-acc-canary` — paraphrase Acc≥0.7 / wrong-fact Acc≤0.4 |
| Δ uncertainty | Bootstrap 95% CI on Δ Acc when per-sample detailed scores exist |
| Paper track | `make bench001-smoke-paper` (rescore frozen preds with GPT-4o-mini + BGE) — labeled `P0_paper` only |
