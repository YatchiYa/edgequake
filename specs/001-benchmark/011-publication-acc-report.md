# 011 — Publication Acc Report: EQ vs LightRAG (First Principles + July 2026 SOTA)

**Date:** 2026-07-22 (medical-mid ladder)  
**Smoke archive (peer reference):** [`e2e/artifacts/history/smoke-20260721T022103Z`](./e2e/artifacts/history/smoke-20260721T022103Z/)  
**Publish claim:** `make bench` → medical-mid **n=200** → [`e2e/artifacts/publish/latest/`](./e2e/artifacts/publish/latest/)  
**Cross-ref:** [001 First Principles](./001-first-principles.md) · [002 Selection](./002-benchmark-selection.md) · [003 Protocol](./003-fair-evaluation-protocol.md) · [004 Fixtures](./004-dataset-and-fixtures.md) · [010 Runbook](./010-smoke-then-core-runbook.md) · [019 Business brief](./019-business-eq-vs-lightrag-and-rag.md) · [Improvements pack](./001-edgquake-improvements/000-index.md) · [017 Beat LightRAG](./001-edgquake-improvements/017-beat-lightrag.md)

---

> **Business / GTM readers:** start with [019 — How EdgeQuake compares to LightRAG and other RAG](./019-business-eq-vs-lightrag-and-rag.md) (plain language). This document is the technical Acc publication record.

---

## 0. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Publish Acc ladder (first principles):                                      │
│    smoke n=40  = daily / CI gate only (CI too wide for release claims)       │
│    medical-mid n=200 = defendable publish Acc (make bench)                   │
│    core n=2162 = ultimate ladder (--i-accept-cost)                           │
│                                                                              │
│  Smoke peer snapshot (T022103Z, n=40, cold fair):                            │
│    EQ Acc 0.731  ·  LR Acc 0.760  ·  Δ −0.029  ·  CI includes 0 ⇒ TIE        │
│    L2 ctx_rel EQ 0.381 vs LR 0.494 · evidence recall EQ 0.936 vs LR 0.951    │
│                                                                              │
│  External claim: peer / statistical tie under named pins — not Acc Beat.     │
│  After make bench (n=200), prefer publish/latest/BUSINESS_REPORT.md.         │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 1. What we measured (honest task identity)

Per [001](./001-first-principles.md) Axiom A1:

> *Given the same GraphRAG-Bench medical corpus and frozen smoke questions, with the same LLM/embed/judge pins, how does EdgeQuake `mix` compare to LightRAG `mix` under official `generation_eval` + `retrieval_eval`?*

| Pin | Value |
|-----|-------|
| Task | `GraphRAG-Bench/EQ-vs-LR` |
| Publish fixture | `medical_publish_question_ids_v1` (n=200, 50/type, seed 42) |
| Smoke fixture (gate) | `smoke_question_ids_v1` (n=40; subset of publish) |
| Corpus | FULL medical (~1.05M chars, uncapped) |
| Chunk | 1200 / overlap 100, adaptive **off** |
| Text / vision / judge | `mistral-small-latest` |
| Embed (SUT + Acc cosine) | `mistral-embed` @ 1024-d |
| Retrieval budget | top-k = 30 both SUTs |
| Fusion | RRF; Mix arm gate off (LR-like always-on arms) |
| Judge | Official `generation_eval` (temp=0) |
| Acc formula | **0.75·statement-F1 + 0.25·embed_cosine** |

**Not claimed:** UltraDomain win-rates · paper Table-2 with GPT-4o-mini+BGE · MMLongBench · full medical+novel core.

---

## 2. Result snapshot (`smoke-20260719T124903Z`)

### L0 — Acc

| SUT | Acc | F1 | cos |
|-----|-----|----|-----|
| EdgeQuake mix | **0.7645** | 0.6993 | 0.9603 |
| LightRAG mix | **0.7539** | 0.6831 | 0.9662 |
| Δ (EQ − LR) | **+0.0106** | +0.0161 | −0.0059 |

- **Δ Acc 95% bootstrap CI:** **[−0.0607, +0.0830]** — includes 0 ⇒ **tie** at α≈0.05.
- Empty answer / empty context rates: **0** both SUTs → validity gates passed.

### L2 — Retrieval (the binding constraint)

| SUT | evidence_recall | context_relevancy |
|-----|-----------------|-------------------|
| EdgeQuake | 0.9282 | **0.3750** |
| LightRAG | **0.9905** | **0.5437** |
| Gap (EQ − LR) | **−0.062** | **−0.169** |

### Acc by question type

| Type | EQ Acc | LR Acc | Who leads |
|------|--------|--------|-----------|
| Fact Retrieval | **0.752** | 0.654 | EQ |
| Complex Reasoning | 0.715 | **0.776** | LR |
| Contextual Summarize | 0.836 | **0.866** | LR |
| Creative Generation | **0.755** | 0.720 | EQ |

### Ops (L3)

| Metric | EQ | LR |
|--------|----|----|
| Query p50 | ~9.6 s | ~3.0 s |
| Query p95 | ~12.9 s | ~6.8 s |
| Ingest wall | ~637 s (~10.6 min full corpus) | (parallel / cached path) |

---

## 3. First principles: what Acc and L2 actually mean

```text
  Question
     │
     ▼
  Retriever  ──►  Context C   ──►  Generator  ──►  Answer Â
     │                 │                              │
     │                 │                              ▼
     │                 │                     Acc = 0.75·F1(Â, gold) + 0.25·cos
     │                 │
     ├─ Evidence Recall: does C cover gold evidence spans?
     └─ Context Relevancy: is C on-topic for the question (low noise)?
```

### Decomposition laws

| Observation | First-principles reading |
|-------------|--------------------------|
| High cos (~0.96) both SUTs | Answers are **semantically near gold**; style pin (`gold`) is working. |
| Acc ≈ tied while L2 relevancy lags | Generation can **compensate** for noisy context when gold-style prompting is strong — Acc alone **hides** retrieval quality. |
| EQ Fact Acc > LR, Reasoning Acc < LR | EQ is competitive on **local fact lookup**; LR wins when **multi-hop / synthesis** needs cleaner context. |
| Recall high (≥0.93) but relevancy mid | EQ **finds** the evidence but **also packs noise** (graph neighbors, redundant entities/relations). Classic GraphRAG failure mode (ICLR 2026 paper Obs.4–6). |

**Publish claim law (P12):** Acc without L2 is not a RAG quality claim. This run is publishable dual-SUT **because** L0+L1+L2 are present and `valid=true`.

---

## 4. What is missing in EdgeQuake to *exceed* LightRAG

Exceed means: **Δ Acc > 0 with CI excluding 0**, and preferably **L2 relevancy ≥ LR**, without breaking fairness pins.

### Gap rank (priority order)

| Priority | Gap | Evidence | First-principles fix direction |
|----------|-----|----------|--------------------------------|
| **P0** | **Context relevancy −17pp** | 0.375 vs 0.544 | **Noise pruning** after fusion: drop off-topic entities/relations/chunks before prompt; path-aware keep (PathRAG-style); query-conditioned chunk re-rank |
| **P1** | **Evidence recall −6pp** | 0.928 vs 0.991 | Close remaining miss: related-chunk expansion, keyword dual-level (LR-like), ensure naive vector arm always contributes gold spans |
| **P2** | **Complex Reasoning Acc −6pp** | 0.715 vs 0.776 | Multi-hop: denser/cleaner entity linking; personalized PageRank / passage+phrase nodes (HippoRAG2 lesson); structured “Entities → Relations → Chunks” already on — improve *selection* not *volume* |
| **P3** | **Latency ~3×** | p50 9.6s vs 3.0s | Parallel arm execution, cache keyword extraction, tighter context token budget (HippoRAG2 ~1k tokens vs LightRAG ~10⁵ in paper) |
| **P4** | Summarize Acc −3pp | 0.836 vs 0.866 | Broader but relevant coverage — same as P0 (relevancy↑ without recall↓) |

### What is *not* the bottleneck (this run)

| Hypothesis | Verdict |
|------------|---------|
| Wrong LLM/embed pin | **Rejected** — lineage is mistral-small + mistral-embed both sides |
| Empty context / failed ingest | **Rejected** — rates 0; full corpus uncapped |
| Acc formula / gold style | **Rejected** — F1≈0.70, cos≈0.96; Acc is healthy |
| Chunk size mismatch | **Rejected** — fair 1200/100 both |
| Need larger judge to “win” | **Wrong goal** — upsizing judge without fixing L2 inflates both SUTs; use for calibration only |

### Concrete EQ engineering backlog (one confound each)

1. **Relevancy gate (P0):** After RRF merge, score each candidate chunk vs query (embed cosine or small cross-encoder); keep top-m with floor; measure L2 ctx_rel before Acc.
2. **Graph soft-prune:** Cap entities/relations in prompt; prefer high-degree *on-query* nodes (not global hubs).
3. **LR-parity keyword path:** Ensure local+global keyword extraction quality matches LightRAG mix (already arms-on; audit keyword LLM failures).
4. **Latency SLO:** Target EQ p50 ≤ 1.5× LR under same concurrency before claiming product parity.
5. **Ablation discipline:** Change one of {prune, rerank, related_chunk_number, fusion} per run; keep Acc pins fixed.

**Stop rule:** Do not stack Acc ablations until **EQ ctx_rel ≥ 0.50** on n=40 smoke under these pins (with Acc/recall companion budgets), or Δ Acc CI excludes 0.  
**Status (2026-07-19):** S1 package labeled (`T151125Z`); Phase 2 Acc+CI = **persistent Acc tie** under same pins (`T151836Z` confirm) — see [020 §2b](./001-edgquake-improvements/020-roadmap.md). Headline stays BM25/`PRUNE=0`.

**How to beat LightRAG (post Phase 2):** Acc-win soft ladder **closed** — persistent Acc **tie** ([018](./001-edgquake-improvements/018-e4-acc-tie-close.md)). Soft Mix knobs (entity_rank / related_chunk / naive×2) did not yield CI win. Remaining hard path: truncation/budget or Phase 3 latency. Architecture + ladder: **[017](./001-edgquake-improvements/017-beat-lightrag.md)**.

---

## 5. Comparison to SOTA RAG (July 2026)

### Landscape (ICLR 2026 GraphRAG-Bench era)

Primary reference: *When to use Graphs in RAG* ([arXiv:2506.05690](https://arxiv.org/abs/2506.05690), ICLR 2026) · leaderboard [graphrag-bench.github.io](https://graphrag-bench.github.io/).

Consensus mid-2026:

1. **GraphRAG is not universally better than vanilla RAG.** Graphs help on multi-hop / summarize / creative; fact lookup often favors lean vector RAG (less noise).
2. **Headline SOTA systems on this suite (paper pins: GPT-4o-mini + BGE):** HippoRAG2, LightRAG, MS-GraphRAG, RAPTOR, Fast-GraphRAG — trade Acc vs token cost.
3. **HippoRAG2** often leads dense retrieval (high Evidence Recall + Context Relevancy) with **compact prompts (~1k tokens)**.
4. **LightRAG** is strong on medical creative faithfulness and developer UX; **prompt-heavy (~10⁴–10⁵ tokens)** in paper measurements.
5. Product practice: **hybrid routing** — vector for facts, graph/path for multi-hop — not “always GraphRAG.”

### Paper medical Acc (full set, GPT-4o-mini judge) — directional only

| Method | Fact Acc | Reason Acc | Summarize Acc | Creative Acc |
|--------|----------|------------|---------------|--------------|
| HippoRAG2 | 66.3 | 62.0 | 63.1 | 68.1 |
| LightRAG | 63.3 | 61.3 | 63.1 | 67.9 |
| **Our EQ (smoke n=40, mistral-small)** | **75.2** | **71.5** | **83.6** | **75.5** |
| **Our LR (same pins)** | **65.4** | **77.6** | **86.6** | **72.0** |

**Honesty banner:** Absolute Acc levels are **not Table-2 comparable**. Different judge family, Acc cosine embed (`mistral-embed` vs `BAAI/bge-large-en-v1.5`), and n=40 smoke vs full medical. Use for **relative** lessons only.

Paper medical **retrieval** (illustrative LightRAG / HippoRAG2):

| Method | Fact Recall / Relevancy | Reason Recall / Relevancy |
|--------|-------------------------|---------------------------|
| HippoRAG2 | ~79 / **~88** | ~77 / **~81** |
| LightRAG | ~80 / **~41** | ~83 / **~43** |
| **Our EQ** | overall recall **0.93** / relevancy **0.38** | (aggregate) |
| **Our LR** | overall recall **0.99** / relevancy **0.54** | (aggregate) |

**Reading:** Our LR relevancy (0.54) already beats paper LightRAG’s medical relevancy band (~0.41–0.45) under a different stack — pin differences matter. Our EQ relevancy (0.38) sits in the **noisy GraphRAG** regime the paper warns about. HippoRAG2’s ~0.80+ relevancy remains the **aspirational L2 SOTA** on this task family.

### Where EQ sits vs July 2026 SOTA

| Dimension | EQ today (this run) | vs LightRAG (peer) | vs HippoRAG2-class SOTA |
|-----------|---------------------|--------------------|-------------------------|
| Acc (fair dual-SUT, our pins) | Tied / +1pp | **On par** | Unknown under same pins (not run) |
| Evidence recall | Strong (0.93) | Slightly behind | Below HippoRAG2 paper band |
| Context relevancy | Weak–mid (0.38) | **Clearly behind** | Far below HippoRAG2 |
| Latency | Slow (~3× LR) | Behind | Behind compact PPR systems |
| Operability (Postgres AGE, API) | Strong product surface | Different job | N/A |

**Bottom line vs SOTA:** Under a fair Mistral-small dual-SUT, EdgeQuake is a **credible LightRAG peer on Acc**, not yet a **retrieval-SOTA** system. The July 2026 frontier on GraphRAG-Bench is defined more by **high-relevancy, low-noise graph retrieval** (HippoRAG2-like) than by raw Acc under a forgiving gold-style generator.

---

## 6. Lessons learned (process + science)

1. **Capped corpus lies.** The earlier “full” Acc with 100k chars (Acc ~0.43, ctx_rel ~0.08) was not publication Acc. Full corpus + pin hygiene flipped the story.
2. **Vision/LLM env bleed corrupts lineage.** Forced publication pins (mistral-small + mistral-embed) are mandatory for honest claims.
3. **Acc can mask retrieval debt.** Fix L2 relevancy before celebrating Acc leads.
4. **Type mix matters.** EQ wins Fact/Creative; LR wins Reasoning/Summarize — product routing should respect that.
5. **Fairness is a product feature.** Matched chunk 1200, top-k 30, Mix arms, official judge, CI on Δ — without these, “we beat LightRAG” is marketing, not measurement.

---

## 7. Recommended next experiments (ordered)

Full multi-lens backlog: **[001-edgquake-improvements](./001-edgquake-improvements/000-index.md)** · Acc-win plan: **[017 Beat LightRAG](./001-edgquake-improvements/017-beat-lightrag.md)** · roadmap: [020](./001-edgquake-improvements/020-roadmap.md).

| # | Experiment | Success criterion | Status |
|---|------------|-------------------|--------|
| 1 | EQ context prune / CE / protect ablation | EQ ctx_rel ≥ 0.50; Acc drop ≤0.02; recall drop ≤0.03 | **Done** — `T151125Z` S1 green |
| 2 | Acc + CI under S1 package pins | Δ Acc CI excludes 0 **or** documented tie | **Done** — Acc **tie** (`T151125Z`+`T151836Z`); L2 unstable → no promote |
| 2b / E1 | Soft path+protect under S1 | ctx_rel ≥0.50; Acc drop ≤0.02 | **Done** `T153436Z` (ctx_rel 0.519) — [017](./001-edgquake-improvements/017-beat-lightrag.md) |
| E2 | Query-conditioned entity ranking | Complex ΔF1 vs LR ≤ 0.03 | **Missed** `T153959Z` (ΔF1 −0.094); code labeled |
| E3 | `RELATED_CHUNK_NUMBER` 5→8 | Summarize recall ≥ 0.95 | **Missed** `T154427Z` (0.863 flat) |
| E3b | `MIX_NAIVE_WEIGHT=2` | Summarize recall ≥ 0.95 | **Missed** `T155350Z` (0.882) — [017](./001-edgquake-improvements/017-beat-lightrag.md) |
| E4 | Acc CI / honesty close | Document persistent tie | **Done** — [018](./001-edgquake-improvements/018-e4-acc-tie-close.md); all Δ Acc CIs include 0; no promote |
| 3 | Optional HippoRAG2-inspired PPR / dual nodes (research) | L2 relevancy → HippoRAG2-like band under same pins | Open after Acc honesty |
| 4 | `P0_paper` rescore (`make bench001-smoke-paper`) | Table-2-comparable Acc under GPT-4o-mini+BGE — separate claim | Open |
| 5 | Core ladder (`make bench001-core`) | Same story on medical+novel — only after Acc-win honesty | Blocked on E4 |

---

## 8. Citation anchors

- Xiang et al., *When to use Graphs in RAG*, arXiv:2506.05690 (ICLR 2026).
- GraphRAG-Bench dataset / eval: https://huggingface.co/datasets/GraphRAG-Bench/GraphRAG-Bench · https://github.com/GraphRAG-Bench/GraphRAG-Benchmark
- This run: `specs/001-benchmark/e2e/artifacts/history/smoke-20260719T124903Z/`
- Launch: `make bench001-full`
