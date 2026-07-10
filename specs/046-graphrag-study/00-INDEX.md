# SPEC-046 — GraphRAG / Hybrid RAG Deep Assessment

**Status:** **RELEASED with EdgeQuake v0.16.0** (2026-07-10) — OPS-P0–P3 + Science P4 (EQ-046-16…18)  
**Scope:** EdgeQuake ingestion + query pipelines vs LightRAG + July 2026 Hybrid RAG; **plus** Postgres/pgvector/AGE performance, defense-in-depth, auto-repair, tracing/metrics  
**Law:** Code is law — every claim is grounded in file paths / function names ([08](./08-CODE-IS-LAW-TRACEABILITY.md))

---

## Document Map

| # | Doc | Purpose |
|---|-----|---------|
| 00 | [INDEX](./00-INDEX.md) | This hub |
| 01 | [FIVE WHYs](./01-FIVE-WHYS.md) | Root-cause chain: why EdgeQuake exists, why gaps persist |
| 02 | [First Principles](./02-FIRST-PRINCIPLES.md) | Physics of Hybrid RAG reduced to irreducible axioms |
| 03 | [Ingestion Compare](./03-INGESTION-PIPELINE-COMPARE.md) | Stage-by-stage EdgeQuake ↔ LightRAG (code) |
| 04 | [Query Compare](./04-QUERY-PIPELINE-COMPARE.md) | Mode-by-mode retrieval + fusion + context |
| 05 | [Multi-Lens Assessment](./05-MULTI-LENS-ASSESSMENT.md) | Quality / cost / latency / ops / tenancy / honesty (v0.16 refresh) |
| 06 | [Competitive Landscape 2026](./06-COMPETITIVE-LANDSCAPE-2026.md) | HippoRAG2, MS GraphRAG, PathRAG, RAPTOR, LazyGraphRAG |
| 07 | [Improvement Plan (Science)](./07-IMPROVEMENT-PLAN.md) | Phased retrieval-physics roadmap (EQ-046-01…18 DONE) |
| 08 | [Code-is-Law Traceability](./08-CODE-IS-LAW-TRACEABILITY.md) | Claim → file → symbol index |
| 09 | [Ops Reliability Deep Study](./09-OPS-RELIABILITY-DEEPSTUDY.md) | Defense / migration / auto-repair / observability (post-ship) |
| 10 | [Postgres · pgvector · AGE Perf](./10-POSTGRES-PGVECTOR-AGE-PERFORMANCE.md) | PG16/17/18 pins, HNSW, AGE indexes, O(N) contract |
| 11 | [Code Smells & Lenses](./11-CODE-SMELLS-AND-LENSES.md) | Smell register + DB/AI/SRE scorecard (v0.16) |
| 12 | [Implementation Plan (Ops)](./12-IMPLEMENTATION-PLAN-OPS.md) | Tickets EQ-046-OPS-01…24 — **all DONE** |
| 13 | [Ops Runbooks](./13-OPS-RUNBOOKS.md) | Upgrade / corruption / drift / retry |
| — | [e2e/](./e2e/) | `run_spec046_acc.sh`, `run_ops17_perf_smoke.sh` |
| — | [LightRAG paper (v3)](./lighrad-2410-05779v3.md) | Source paper markdown |

**External code roots (read-only references):**
- EdgeQuake: repo root (this tree)
- LightRAG latest: sibling checkout when comparing (optional)

---

## Executive Verdict (one screen)

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  VERDICT — EdgeQuake v0.16.0 (SPEC-046 shipped)                          │
├──────────────────────────────────────────────────────────────────────────┤
│  SCIENCE: Hybrid RAG + PPR-default + bipartite dual-node + ACC CI JSON. │
│  SUBSTRATE: Strong Postgres GraphRAG + fail-closed ANN readiness.        │
│                                                                          │
│  OPS-P0–P3: fail-closed ops, arm spans, graph Prometheus, PPR default,  │
│             LLM-judge opt-in, Mistral live faithfulness.                 │
│  Science P4: make spec046-acc + workflow artifact; bipartite PPR;        │
│              mini corpus retrieval ACC (HF full corpus still optional).  │
│                                                                          │
│  Still open (post-0.16): full HF GraphRAG-Bench download ACC;            │
│  true cross-encoder rerank; density YAML; LLM community report depth;    │
│  large-scale postgres-perf wall-time artifacts.                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Pre-release gates (code is law)

```bash
make ops17-smoke          # PG pin SSOT
make spec046-acc          # ACC + AccReport JSON
make release-gates        # fmt/clippy/SPEC-006/018/WebUI/version
make test-e2e-lint        # Playwright flake anti-patterns
```

---

## How to Read This Pack

1. Start with **01 FIVE WHYs** if you want the "why bother" narrative.
2. Read **02 First Principles** before debating features — it kills cargo-cult GraphRAG.
3. Use **03 / 04** as engineering truth tables (code citations).
4. Use **05 / 06 / 11** for product / competitive / smell decisions (**v0.16 scores in 05+11**).
5. **07** science tickets and **12** ops tickets are **complete** for v0.16 — use deferred lists for next cut.
6. Use **09 / 10** when challenging Postgres, migration, or reliability claims (post-ship status).
7. Use **08** as the claim → symbol index — **if it disagrees with crates/, update 08**.

---

## Sources (July 2026)

- LightRAG paper: arXiv:2410.05779v3 ([local](./lighrad-2410-05779v3.md))
- LightRAG code: sibling `LightRAG` checkout when comparing (default query mode = `mix`)
- GraphRAG-Bench (ICLR 2026): https://github.com/GraphRAG-Bench/GraphRAG-Benchmark — "when to use graphs"
- HippoRAG 2: arXiv:2502.14802 — PPR + dual-node (phrase + passage)
- Practitioner Hybrid Search 2026: dense + sparse + graph + RRF + cross-encoder rerank
- pgvector 0.8.0: iterative index scans — https://www.postgresql.org/about/news/pgvector-080-released-2952/
- Azure AGE performance (2026-01): https://learn.microsoft.com/en-us/azure/postgresql/azure-ai/generative-ai-age-performance
- Apache AGE PG16/17/18 + 1.7.0 RLS / upgrade warnings — https://github.com/apache/age
- OpenTelemetry GenAI semantic conventions — retrieval + generation spans
- EdgeQuake pins SSOT: `edgequake/docker/extension-pins.sh` (pg16/pg17/pg18)
- Release: `CHANGELOG.md` **[0.16.0]** · `VERSION` = `0.16.0`
