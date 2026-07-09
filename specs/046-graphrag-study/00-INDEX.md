# SPEC-046 — GraphRAG / Hybrid RAG Deep Assessment

**Status:** Assessment complete (2026-07-09)  
**Scope:** EdgeQuake ingestion + query pipelines vs LightRAG (paper + latest code) + July 2026 Hybrid RAG state of the art  
**Law:** Code is law — every claim below is grounded in file paths / function names

---

## Document Map

| # | Doc | Purpose |
|---|-----|---------|
| 00 | [INDEX](./00-INDEX.md) | This hub |
| 01 | [FIVE WHYs](./01-FIVE-WHYS.md) | Root-cause chain: why EdgeQuake exists, why gaps persist |
| 02 | [First Principles](./02-FIRST-PRINCIPLES.md) | Physics of Hybrid RAG reduced to irreducible axioms |
| 03 | [Ingestion Compare](./03-INGESTION-PIPELINE-COMPARE.md) | Stage-by-stage EdgeQuake ↔ LightRAG (code) |
| 04 | [Query Compare](./04-QUERY-PIPELINE-COMPARE.md) | Mode-by-mode retrieval + fusion + context |
| 05 | [Multi-Lens Assessment](./05-MULTI-LENS-ASSESSMENT.md) | Quality / cost / latency / ops / tenancy / honesty scorecard |
| 06 | [Competitive Landscape 2026](./06-COMPETITIVE-LANDSCAPE-2026.md) | HippoRAG2, MS GraphRAG, PathRAG, RAPTOR, LazyGraphRAG |
| 07 | [Improvement Plan](./07-IMPROVEMENT-PLAN.md) | Phased, code-grounded roadmap to best-in-class Hybrid RAG |
| 08 | [Code-is-Law Traceability](./08-CODE-IS-LAW-TRACEABILITY.md) | Claim → file → symbol index |
| — | [LightRAG paper (v3)](./lighrad-2410-05779v3.md) | Source paper markdown |

**External code roots (read-only references):**
- EdgeQuake: `/Users/raphaelmansuy/Github/03-working/edgequake`
- LightRAG latest: `/Users/raphaelmansuy/Github/03-working/LightRAG`

---

## Executive Verdict (one screen)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  VERDICT 2026-07-09                                                       │
├──────────────────────────────────────────────────────────────────────────┤
│  EdgeQuake is a strong LightRAG-class Hybrid RAG in Rust with real        │
│  production extras (Mix/RRF, BM25, workspaces, AGE+pgvector, PDF).        │
│                                                                          │
│  It is NOT yet the best Hybrid RAG on the market.                         │
│                                                                          │
│  Why not:                                                                 │
│   1. Graph retrieval is still "vector-over-entities/relations + BFS"      │
│      — missing HippoRAG2-class Personalized PageRank / dual-node memory │
│   2. Global mode ≠ community reports; density/quality of graph is         │
│      under-instrumented (GraphRAG-Bench lesson)                           │
│   3. Intent router is cost-optimized but partially inverted vs evidence   │
│      (Factual→Local can add graph tax on Level-1 facts)                   │
│   4. LightRAG latest pulled ahead on: semantic chunking (V), KG chunk     │
│      pick VECTOR, role-LLM defaults, parse→VLM→extract staging maturity   │
│   5. No GraphRAG-Bench / HippoRAG-style eval harness in-repo              │
│                                                                          │
│  Path to #1: keep Mix+RRF+BM25 spine; add PPR dual-node arm; fix router;  │
│  close LightRAG parity gaps; measure graph quality → retrieval → gen.     │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## How to Read This Pack

1. Start with **01 FIVE WHYs** if you want the "why bother" narrative.
2. Read **02 First Principles** before debating features — it kills cargo-cult GraphRAG.
3. Use **03 / 04** as engineering truth tables (code citations).
4. Use **05 / 06** for product / competitive decisions.
5. Execute **07** — the only document that should drive tickets.

---

## Sources (July 2026)

- LightRAG paper: arXiv:2410.05779v3 ([local](./lighrad-2410-05779v3.md))
- LightRAG code: `/Users/raphaelmansuy/Github/03-working/LightRAG` (default query mode = `mix`)
- GraphRAG-Bench (ICLR 2026): https://github.com/GraphRAG-Bench/GraphRAG-Benchmark — "when to use graphs"
- HippoRAG 2: arXiv:2502.14802 — PPR + dual-node (phrase + passage)
- Practitioner Hybrid Search 2026: dense + sparse + graph + RRF + cross-encoder rerank
- Azure HorizonDB Graph-Augmented RAG patterns (pgvector + AGE) — aligns with EdgeQuake storage choice
