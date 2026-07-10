# 05 — Multi-Lens Assessment

**Lens pack:** Product, retrieval science, systems, cost, risk, honesty.  
**Date:** 2026-07-09 (baseline) · **Refresh:** 2026-07-10 **v0.16.0** code-is-law (see [11](./11-CODE-SMELLS-AND-LENSES.md) for post-ship scores)

---

## Lens A — Product / User Value

| Question | EdgeQuake | LightRAG | Verdict |
|----------|-----------|----------|---------|
| Can a team run multi-tenant GraphRAG in prod? | Yes (workspaces, RLS, API) | Partial | **EQ** |
| Can a researcher swap Neo4j/Milvus quickly? | No (Postgres-first) | Yes | **LR** |
| PDF / enterprise docs? | Strong (pdfium + page chunks) | Strong (MinerU/Docling) | Tie |
| Default answer quality out of box? | Mix+RRF+BM25+PPR — strong | Mix+round-robin — strong | **EQ slight** |
| Time-to-first-success for OSS hackers | Higher (Rust/make/Postgres) | Lower (pip) | **LR** |

**Product thesis:** EdgeQuake should own **enterprise Hybrid RAG**. LightRAG owns **hackable GraphRAG lab**. Do not blur.

---

## Lens B — Retrieval Science (July 2026)

### Baseline (2026-07-09 assessment)

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

**Science verdict (then):** EdgeQuake at rung 3–4.

### Code truth — EdgeQuake **v0.16.0** (2026-07-10)

```text
 4. Difficulty-aware routing        ✅ Factual→Naive + arm gate
 5. Associative graph walk (PPR)    ✅ default PPR (`bfs` escape)
 6. Dual-node phrase↔passage        ✅ bipartite PPR pick (AGE store lite)
 7. Path / flow pruning             ✅
 8. Community reports (optional)    ⚠️ extractive opt-in (LLM depth open)
 9. Full-link eval                  ✅ ACC CI + mini corpus (+ LLM-judge opt-in)
10. Ops fail-loud / heal / OTel     ✅ OPS-P0–P3
```

**Science verdict (now):** Rungs 4–7 + 9–10 shipped. Market leaders may still lead on **full HF GraphRAG-Bench ACC** and **true cross-encoder**; EQ leads **enterprise Hybrid + fail-closed Postgres**.

---

## Lens C — Systems / Reliability

| Concern | EdgeQuake v0.16 | Notes |
|---------|-----------------|-------|
| Typed pipelines | Excellent | Rust Result/async |
| Cross-store consistency | Good saga | Not 2PC; KV compensate + drift metrics |
| Resume after crash | Good | failed_chunks retry→merge; fingerprint stale purge |
| Observability | High | arm timings, rag spans, graph Prometheus |
| Horizontal scale | Postgres-bound | Correct for AGE+pgvector choice |

---

## Lens D — Cost & Latency

```text
                    Index $          Query $           Latency
LightRAG paper      Low (no rebuild) Medium            Low vs GraphRAG
MS GraphRAG         Very High        High (reports)    High
HippoRAG2           Medium           Low tokens        Low-Med
EdgeQuake Mix       Med-High (≤3 arms)Med              Med (gated arms)
EdgeQuake + router  Med              Low on L1         Better p50
```

**Cost lever #1:** Intent router → skip graph on L1 — ✅ shipped.  
**Cost lever #2:** Path/PPR prune → cut tokens on L2/L3 — ✅ shipped.  
**Cost lever #3:** Role-specific small models for extract/keyword — ✅ matrix present.

---

## Lens E — Competitive Honesty Scorecard

Scores 1–5. **5 = best-in-class market.**  
**v0.16.0 EQ** scores reflect shipped code (baseline 2026-07-09 was lower on L2/eval/ops).

| Capability | EQ | LR | HippoRAG2 | MS GraphRAG | Naive+Rerank |
|------------|:--:|:--:|:---------:|:-----------:|:------------:|
| L1 Fact | **4** | 3 | **5** | 2 | **5** |
| L2 Multi-hop | **4** | 3 | **5** | 4 | 2 |
| L3 Summary | 3 | 4 | 4 | **5** | 3 |
| Token efficiency | **4** | 2 | **5** | 1 | **5** |
| Incremental ingest | **5** | **5** | 4 | 2 | **5** |
| Hybrid fusion (dense+sparse) | **5** | 2 | 3 | 2 | 4 |
| Enterprise multi-tenant | **5** | 2 | 1 | 2 | 3 |
| Eval / benchmarks | **4** | 2 | 4 | 3 | 3 |
| Dev experience (OSS) | 3 | **5** | 3 | 2 | **5** |
| Ops fail-closed | **5** | 2 | 1 | 2 | 2 |

\*Weights: L2×2, token×1.5, enterprise×1.5, others×1. HippoRAG2 still leads pure science ACC; **EQ leads enterprise Hybrid + ops**.

---

## Lens F — SWOT (EdgeQuake v0.16.0)

```text
STRENGTHS                         WEAKNESSES
─────────────────────────────     ─────────────────────────────
Rust + Postgres AGE/pgvector      Full HF GraphRAG-Bench ACC open
Mix + RRF + BM25 + PPR default    True cross-encoder rerank open
Workspaces / RLS / API            LLM community report depth open
PDF page-aware + VLM path         Perf bench artifacts optional
Fail-closed /ready + ACC CI       Hybrid naming ≠ LightRAG hybrid
Saga + failed_chunks retry        —

OPPORTUNITIES                     THREATS
─────────────────────────────     ─────────────────────────────
Publish Postgres GraphRAG ref     LightRAG keeps shipping knobs
Own enterprise eval brand         Neo4j GraphRAG module DX
HF corpus nightly ACC             Teams default to Naive+Rerank
Cross-encoder prod path           GraphRAG-Bench graph-tax narratives
```

---

## Lens G — "Would I bet production on it today?" (v0.16.0)

| Workload | Bet? | Why |
|----------|------|-----|
| Enterprise multi-tenant KB, mixed queries | **Yes** | Mix+BM25+PPR+tenancy+`/ready` |
| Pure multi-hop research QA, max ACC | **Conditional** | PPR+bipartite shipped; need HF ACC to claim SOTA |
| Global corpus thematic reports | **Partial** | Extractive reports; not MS GraphRAG depth |
| Simple FAQ / policy lookup | **Yes via router** | Factual→Naive; avoid Mix tax |

---

## Synthesis

EdgeQuake is the **best-positioned open Hybrid RAG substrate for Postgres-centric enterprises** — and as of **v0.16.0** the retrieval brain has climbed to **PPR-default + bipartite + ACC CI**. Honest label: **production Hybrid RAG (LightRAG-class+) with fail-closed ops**, not yet "beat HippoRAG2 on GraphRAG-Bench download."

Post-ship scorecard detail: [11-CODE-SMELLS-AND-LENSES.md](./11-CODE-SMELLS-AND-LENSES.md).
