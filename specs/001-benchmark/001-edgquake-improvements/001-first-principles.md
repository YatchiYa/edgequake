# 001 — First Principles: What to Improve in EdgeQuake

**Cross-ref:** [000 Index](./000-index.md) · [Parent eval axioms](../001-first-principles.md) · [011 Acc Report](../011-publication-acc-report.md) · [017 Beat LightRAG](./017-beat-lightrag.md)

---

## 1. System decomposition (irreducible)

```text
  Question q
     │
     ▼
  Retriever R  ──►  Context C   ──►  Generator G  ──►  Answer Â
     │                 │                              │
     │                 │                              ▼
     │                 │                     Acc = 0.75·F1(Â, gold) + 0.25·cos
     │                 │
     ├─ Evidence Recall: does C cover gold evidence spans?
     └─ Context Relevancy: is C on-topic for q (low noise)?
```

**Law L1 — Acc is a composite.** High Acc with low relevancy means the generator compensated for noisy C. That is not retrieval excellence.

**Law L2 — Recall ≠ Relevancy.** Finding gold spans while packing graph neighbors is the classic GraphRAG failure mode (ICLR 2026 GraphRAG-Bench Obs.4–6).

**Law L3 — Optimize the binding constraint.** On baseline `T124903Z`, Acc was tied and **relevancy (−17pp)** was binding. After Phase 1–2 (CE+protect), L2 improved but Acc stays a **tie** and ctx_rel is **unstable** at the gate. Binding now: **Complex Reasoning F1 (−8 to −12pp with recall=1.0)** plus **stable ctx_rel ≥ LR** — packing/selection, not Acc fishing or more CE alone. See [017](./017-beat-lightrag.md).

**Law L4 — One confound.** Changing prune + fusion + related_chunk_number together destroys causal inference.

**Law L5 — Fairness pins define the task.** Exceeding LightRAG means beating LR under the same pins — not inventing a different product profile and calling it Acc.

---

## 2. Authoritative evidence snapshot

### 2a. Publication baseline (pre-Phase-1)

| Layer | Metric | EQ | LR | Gap |
|-------|--------|----|----|-----|
| L0 | Acc | 0.765 | 0.754 | +0.011 (CI includes 0) |
| L2 | evidence_recall | 0.928 | 0.991 | −0.063 |
| L2 | context_relevancy | **0.375** | **0.544** | **−0.169** |
| L3 | query p50 | ~9.6 s | ~3.0 s | ~3× |
| Type | Fact Acc | 0.752 | 0.654 | EQ leads |
| Type | Reasoning Acc | 0.715 | 0.776 | LR leads |
| Type | Summarize Acc | 0.836 | 0.866 | LR leads |
| Type | Creative Acc | 0.755 | 0.720 | EQ leads |

Archive: [`../e2e/artifacts/history/smoke-20260719T124903Z/`](../e2e/artifacts/history/smoke-20260719T124903Z/)

### 2b. Phase 1 S1 package (labeled CE+protect)

| Layer | Metric | EQ (S1) | vs baseline EQ | Budget |
|-------|--------|---------|----------------|--------|
| L0 | Acc | **0.760** | −0.004 | ≤0.02 drop ✅ |
| L2 | context_relevancy | **0.519** | +0.144 | ≥0.50 ✅ |
| L2 | evidence_recall | **0.928** | ~0 | ≤0.03 drop ✅ |

Archive: [`../e2e/artifacts/history/smoke-20260719T151125Z/`](../e2e/artifacts/history/smoke-20260719T151125Z/) · pins in [000](./000-index.md) / [020 §1b](./020-roadmap.md).

**Law update:** Acc-win E0–E4 is **closed** ([018](./018-e4-acc-tie-close.md)). Acc remains a **persistent statistical tie** under baseline and all soft Mix labeled ablations (every Δ Acc CI includes 0). S1 CE+protect is the best **labeled** L2 package (ctx_rel ~0.50), not the Acc headline. Soft knobs exhausted; harder path = truncation/budget or latency ([017](./017-beat-lightrag.md)).

---

## 3. Priority ranking (first principles → backlog)

| Priority | Gap | Why first | Primary lens | Status |
|----------|-----|-----------|--------------|--------|
| **B0** | Complex F1 −8 to −12pp (recall=1.0) | Binding Acc swing vs LR | [017](./017-beat-lightrag.md) · [012](./012-lens-multihop-graph.md) | **Next** (E2) |
| **B1** | ctx_rel unstable at ≥0.50 | Blocks promotion / L2 claim | [017](./017-beat-lightrag.md) · [010](./010-lens-retrieval-noise.md) | **Next** (E0–E1) |
| **P0** | Context relevancy −17pp → **cleared on S1 package** | Was binding; Acc hid it | [010](./010-lens-retrieval-noise.md) | **S1 green** `T151125Z` |
| **P1** / **B2** | Evidence / Summarize recall vs LR | Remaining coverage | [011](./011-lens-evidence-coverage.md) | Open (E3) |
| **P2** | Reasoning Acc −6pp (baseline) | Multi-hop path selection | [012](./012-lens-multihop-graph.md) | Open; protect helped; E2 targets packing |
| **P3** | Latency ~3× | Product parity | [013](./013-lens-latency-ops.md) | Open after Acc honesty |
| Product | Type split Fact/Creative vs Reason/Summarize | Router, not Acc-pin change | [014](./014-lens-generation-routing.md) | Open |
| Ingest | Chunk situating / adaptive | Opt-in; Acc pin stays 1200/100 | [015](./015-lens-ingest-chunking.md) | Open |
| Eval | Ablation + publish gates | Prevents false wins | [016](./016-lens-eval-fairness.md) · [017](./017-beat-lightrag.md) | Open |

---

## 4. What is *not* the bottleneck (this run)

| Hypothesis | Verdict |
|------------|---------|
| Wrong LLM / embed pin | Rejected — mistral-small + mistral-embed both sides |
| Empty context / failed ingest | Rejected — rates 0; full corpus |
| Acc formula / gold style | Rejected — F1≈0.70, cos≈0.96 |
| Chunk size mismatch | Rejected — fair 1200/100 |
| Need larger judge to “win” | Wrong goal — inflates both SUTs; use for calibration only (`P0_paper`) |

---

## 5. July 2026 practice (compressed)

Consensus for production RAG and GraphRAG-Bench-class systems:

1. **Hybrid retrieve → RRF → rerank/prune → small prompt** (top 5–10 chunks), not “stuff top-30 graph neighbors.”
2. **PathRAG-style flow prune** — redundancy is the GraphRAG failure, not insufficiency.
3. **HippoRAG2-class lesson** — high Evidence Recall *and* Context Relevancy with compact prompts (~1k tokens), not prompt-heavy dumps.
4. **Contextual embeddings** (chunk + 50–100 token situating text) remain high-ROI at ingest.
5. **Evals first** — Context Relevancy / Evidence Recall (RAG Triad / GraphRAG-Bench L2) before celebrating Acc.

Primary paper: Xiang et al., *When to use Graphs in RAG*, [arXiv:2506.05690](https://arxiv.org/abs/2506.05690) (ICLR 2026).

---

## 6. Exceed definition (honest)

EdgeQuake **exceeds** LightRAG on this task when **all** hold under publication pins:

1. EQ `context_relevancy` ≥ LR (or absolute ≥ 0.50 on smoke as intermediate gate), and  
2. Δ Acc 95% bootstrap CI excludes 0 in EQ’s favor, and  
3. Empty answer / empty context rates remain ≤ publish fairness thresholds, and  
4. Scorecard `valid=true` with L0+L1+L2 present.

Partial wins (e.g. Acc point estimate only) must be labeled **tie / directional**, never “beats LightRAG.”

Full Acc-win ladder, EQ↔LR architecture diff, and experiment pins: **[017 Beat LightRAG](./017-beat-lightrag.md)**.
