# 011 — Complementary Benchmarks & Evaluation Methodology

**Cross-ref:** [001](./001-first-principles.md) · [007](./007-ml-scientist-lens.md) · SPEC-046 GraphRAG-Bench ACC

MMLongBench-Doc is the **right first stress test** for long multi-modal PDFs. It is **not** a complete RAG quality program. This doc defines what else to run and how to think about methodology.

---

## 1. Methodology (first principles)

A serious RAG evaluation program separates **layers**:

```text
  L1 Embedding / retrieval quality     (no generation)
  L2 End-to-end QA on text corpora     (classic RAG)
  L3 Multi-hop / graph necessity       (GraphRAG-specific)
  L4 Long multi-modal documents        (MMLongBench-Doc, UniDoc, LongDocURL)
  L5 Robustness / refusal              (RGB, CRAG unanswerable)
  L6 Domain / product golden set       (your real users)
```

**Rule:** Never optimize only L4. Gains on MMLongBench-Doc can hide regressions on L1–L3.

**Rule:** Always fix the system profile (models, chunking, mode) before comparing versions.

**Rule:** Report task name + corpus + metrics + pins — not a single magical “RAG score.”

---

## 2. Suggested benchmark portfolio for EdgeQuake

| Priority | Benchmark | Layer | Why for EdgeQuake | Primary metrics |
|----------|-----------|-------|-------------------|-----------------|
| **P0** | **MMLongBench-Doc** (this SPEC) | L4 | Real long PDFs, multi-modal, cross-page, unanswerable | Acc, gen. F1, slices |
| **P0** | **GraphRAG-Bench** (SPEC-046 ACC + **[SPEC-001](../001-benchmark/000-index.md)** dual-SUT) | L3 | When graphs help; EQ vs LightRAG head-to-head | ACC / Acc+ROUGE side-by-side |
| **P1** | **MultiHop-RAG** ([repo](https://github.com/yixuantt/MultiHop-RAG), COLM 2024) | L2/L3 | Cross-document hops; retrieval Hits@K + QA | Hits@K, MAP, QA Acc |
| **P1** | **UniDoc-Bench** ([Salesforce](https://github.com/SalesforceAIResearch/UniDOC-Bench), arXiv:2510.03663) | L4 | Explicit **MM-RAG** paradigms (text/image/fusion/joint) | RAGAS + retrieval + correctness |
| **P2** | **LongDocURL** ([repo](https://github.com/dengc2023/LongDocURL)) | L4 | Understanding + reasoning + locating on long PDFs | Task-category scores |
| **P2** | **CRAG** (Meta / KDD Cup) | L2/L5 | Dynamic / long-tail / unanswerable trustworthiness | Challenge score |
| **P2** | **RGB** | L5 | Noise, rejection, integration, counterfactual | Per-mode rates |
| **P3** | **BEIR / MTEB** | L1 | Embedding choice (`mistral-embed` vs alternatives) | nDCG, Recall |
| **P3** | **HotpotQA** | L2 | Legacy multi-hop reference | EM, F1, support F1 |
| **P3** | **Edinburgh MMLongBench** (NeurIPS 2025) | L4 | Broader LCVLM suite incl. Visual RAG | Suite metrics |
| **Always** | **Product golden set** | L6 | 50–200 real user questions with citations | Acc + citation precision |

References for landscape: [TypeGraph RAG benchmarks overview](https://typegraph.ai/blog/rag-benchmarks-retrieval-and-end-to-end-evaluation), UniDoc-Bench paper critique of single-page DocVQA-style evals.

---

## 3. How MMLongBench-Doc fits vs UniDoc-Bench

| Dimension | MMLongBench-Doc | UniDoc-Bench |
|-----------|-----------------|--------------|
| Original SUT | LVLM full-doc | MM-RAG pipelines |
| Corpus | 135 long PDFs | ~70k pages / 8 domains |
| EdgeQuake fit | Excellent stress for ingest+hybrid | Excellent for modality ablation |
| Cost | High per doc (long) | High at full scale |
| SPEC-047 role | **First** | **Second** multimodal RAG |

Do not start UniDoc until smoke MMLongBench is operational — harness patterns transfer.

---

## 4. Evaluation methodology playbook

### 4.1 Version comparison

1. Freeze dataset revision + fixture lists.  
2. Run smoke on old and new EdgeQuake SHAs.  
3. If smoke ΔF1 > noise band → run core.  
4. Only run full for release candidates.  
5. Attach scorecards to CHANGELOG.

### 4.2 Ablation discipline

Change **one** variable per profile (mode XOR parse XOR oracle). Record `profile_id`.

### 4.3 Retrieval vs generation split

Always keep an oracle or page-hit diagnostic so you know whether to fix retriever or prompt/LLM.

### 4.4 Human spot-check

For each stage, manually review 20 errors (stratified). Automatic metrics miss “right number, wrong interpretation.”

### 4.5 Cost-normalized quality

Report `F1 per $` and `F1 per hour` for smoke — forces honest provider choices.

---

## 5. Anti-patterns

- Cherry-picking questions after seeing scores  
- Tuning prompts on the full test set without a holdout  
- Comparing RAG F1 to LVLM leaderboard as identical  
- Declaring victory from smoke n≈10 without core  
- Mixing embedding models mid-corpus  

---

## 6. Proposed EdgeQuake “Quality Dashboard” (future)

```text
  SPEC-047 MMLongBench F1 (smoke/core/full)
  SPEC-046 GraphRAG-Bench ACC
  MultiHop-RAG Hits@5 + QA
  Product golden Acc
  Ops: ingest_fail_rate, p95_query
```

Single markdown or CI artifact index — not a heavyweight product UI in v1.

---

## 7. Recommendation (executive)

1. **Now:** Implement SPEC-047 smoke → core → full with Mistral Small + embed + hybrid.  
2. **Next:** Wire MultiHop-RAG for cross-doc hops (graph value).  
3. **Then:** UniDoc-Bench for modality-fair MM-RAG claims.  
4. **Always:** Maintain a private product golden set — public benches do not replace it.

Next: [012 Acceptance & Scorecard](./012-acceptance-criteria-and-scorecard.md).
