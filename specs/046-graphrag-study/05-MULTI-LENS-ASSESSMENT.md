# 05 — Multi-Lens Assessment

**Lens pack:** Product, retrieval science, systems, cost, risk, honesty.  
**Date:** 2026-07-09

---

## Lens A — Product / User Value

| Question | EdgeQuake | LightRAG | Verdict |
|----------|-----------|----------|---------|
| Can a team run multi-tenant GraphRAG in prod? | Yes (workspaces, RLS, API) | Partial | **EQ** |
| Can a researcher swap Neo4j/Milvus quickly? | No (Postgres-first) | Yes | **LR** |
| PDF / enterprise docs? | Strong (pdfium + page chunks) | Strong (MinerU/Docling) | Tie |
| Default answer quality out of box? | Mix+RRF+BM25 — strong | Mix+round-robin — strong | **EQ slight** |
| Time-to-first-success for OSS hackers | Higher (Rust/make/Postgres) | Lower (pip) | **LR** |

**Product thesis:** EdgeQuake should own **enterprise Hybrid RAG**. LightRAG owns **hackable GraphRAG lab**. Do not blur.

---

## Lens B — Retrieval Science (July 2026)

```text
Capability ladder (must climb in order)
───────────────────────────────────────
 1. Dense retrieval                 ✅ EQ ✅ LR
 2. Sparse + dense fusion (RRF)     ✅ EQ ❌ LR (core)
 3. Entity/relation dual-level      ✅ EQ ✅ LR
 4. Difficulty-aware routing        ⚠️ EQ (miswired) ❌ LR
 5. Associative graph walk (PPR)    ❌ EQ ❌ LR
 6. Dual-node phrase↔passage        ❌ EQ ❌ LR
 7. Path / flow pruning             ❌ EQ ❌ LR
 8. Community reports (optional)    ❌ EQ ❌ LR
 9. Full-link eval (graph→ret→gen)  ❌ EQ ❌ LR
```

**Science verdict:** EdgeQuake is at rung 3–4. Market leaders for hard multi-hop (HippoRAG2) sit at 5–6 with better token efficiency. Closing 4→6 is the science gap.

---

## Lens C — Systems / Reliability

| Concern | EdgeQuake | Notes |
|---------|-----------|-------|
| Typed pipelines | Excellent | Rust Result/async |
| Cross-store consistency | Good saga | Not 2PC |
| Resume after crash | Medium | Reanalyze/retry-chunks; weaker than LR process_options purge |
| Observability | Medium | tracing; missing graph quality metrics |
| Horizontal scale | Postgres-bound | Correct for AGE+pgvector choice |

---

## Lens D — Cost & Latency

```text
                    Index $          Query $           Latency
LightRAG paper      Low (no rebuild) Medium            Low vs GraphRAG
MS GraphRAG         Very High        High (reports)    High
HippoRAG2           Medium           Low tokens        Low-Med
EdgeQuake Mix       Med-High (3 arms)Med-High          Med (parallel arms)
EdgeQuake + router  Med              Low on L1         Better p50
```

**Cost lever #1:** Fix intent router → skip graph on L1.  
**Cost lever #2:** Path/PPR prune → cut tokens 30–50% on L2/L3.  
**Cost lever #3:** Role-specific small models for extract/keyword (LR pattern).

---

## Lens E — Competitive Honesty Scorecard

Scores 1–5. **5 = best-in-class market.**

| Capability | EQ | LR | HippoRAG2 | MS GraphRAG | Naive+Rerank |
|------------|:--:|:--:|:---------:|:-----------:|:------------:|
| L1 Fact | 3 | 3 | **5** | 2 | **5** |
| L2 Multi-hop | 3 | 3 | **5** | 4 | 2 |
| L3 Summary | 3 | 4 | 4 | **5** | 3 |
| Token efficiency | 3 | 2 | **5** | 1 | **5** |
| Incremental ingest | **5** | **5** | 4 | 2 | **5** |
| Hybrid fusion (dense+sparse) | **5** | 2 | 3 | 2 | 4 |
| Enterprise multi-tenant | **5** | 2 | 1 | 2 | 3 |
| Eval / benchmarks | 2 | 2 | 4 | 3 | 3 |
| Dev experience (OSS) | 3 | **5** | 3 | 2 | **5** |
| **Weighted* ** | **3.7** | **3.2** | **4.1** | **2.7** | **3.8** |

\*Weights: L2×2, token×1.5, enterprise×1.5, others×1. HippoRAG2 leads science; Naive+Rerank wins cheap L1; **EQ leads enterprise Hybrid**.

---

## Lens F — SWOT (EdgeQuake)

```text
STRENGTHS                         WEAKNESSES
─────────────────────────────     ─────────────────────────────
Rust + Postgres AGE/pgvector      No PPR / dual-node walk
Mix + RRF + BM25 spine            Intent router misaligned
Workspaces / RLS / API            No GraphRAG-Bench harness
PDF page-aware + VLM path         Semantic chunking missing
Community labels on nodes         Token accounting less dynamic
Saga ingest + recovery APIs       Hybrid naming ≠ LightRAG hybrid

OPPORTUNITIES                     THREATS
─────────────────────────────     ─────────────────────────────
Import HippoRAG2 physics          LightRAG keeps shipping knobs
Become "Postgres GraphRAG" ref    Neo4j GraphRAG module DX
Own enterprise eval brand         Teams default to Naive+Rerank
Path prune → kill token tax       GraphRAG-Bench shows graph tax
```

---

## Lens G — "Would I bet production on it today?"

| Workload | Bet? | Why |
|----------|------|-----|
| Enterprise multi-tenant KB, mixed queries | **Yes** | EQ Mix+BM25+tenancy |
| Pure multi-hop research QA, max ACC | **Not yet** | Need PPR arm + eval |
| Global corpus thematic reports | **No** | Use MS GraphRAG / build reports |
| Simple FAQ / policy lookup | **Overkill** | Naive+rerank; router should send here |

---

## Synthesis

EdgeQuake is the **best-positioned open Hybrid RAG substrate for Postgres-centric enterprises** in this comparison — but **not** the best retrieval brain. The improvement plan must upgrade the brain without abandoning the substrate.
