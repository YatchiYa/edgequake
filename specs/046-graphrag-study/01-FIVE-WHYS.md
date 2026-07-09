# 01 — Five WHYs: EdgeQuake Hybrid RAG

**Lens:** Root-cause analysis (Toyota 5-Why) applied to product + architecture.  
**Grounding:** Code paths in EdgeQuake + LightRAG; GraphRAG-Bench (ICLR 2026).

---

## WHY #1 — Why does EdgeQuake exist as a GraphRAG system?

**Because flat chunk RAG fails on multi-hop and relational questions.**

The LightRAG paper (arXiv:2410.05779) states the failure mode clearly: flat representations cannot synthesize inter-dependencies (e.g. EVs → air quality → transit planning). EdgeQuake inherits that thesis and implements graph-based indexing + dual-level retrieval in Rust.

```text
User question (multi-hop)
        │
        ▼
┌───────────────────┐     FAIL      ┌────────────────────┐
│ Naive chunk RAG   │ ───────────► │ Fragmented answer  │
│ (vector only)     │              │ Missing links      │
└───────────────────┘              └────────────────────┘
        │
        │ SUCCESS PATH (LightRAG / EdgeQuake thesis)
        ▼
┌───────────────────┐              ┌────────────────────┐
│ Entity + Relation │ ───────────► │ Coherent synthesis │
│ graph + vectors   │              │ across documents   │
└───────────────────┘              └────────────────────┘
```

**Code law:** EdgeQuake modes encode this thesis in `edgequake-query/src/modes.rs` (`Naive` | `Local` | `Global` | `Hybrid` | `Mix`).

---

## WHY #2 — Why is EdgeQuake shaped like LightRAG (not MS GraphRAG)?

**Because LightRAG optimizes for incremental updates and low retrieval cost vs community-report GraphRAG.**

Paper §3.4: indexing cost ≈ `total_size / chunk_size` LLM calls; retrieval is vector search over entities/relations — not Leiden community traversal. EdgeQuake mirrors that:

| LightRAG concept | EdgeQuake symbol |
|------------------|------------------|
| Dual-level keywords | `LLMKeywordExtractor` → `high_level` / `low_level` |
| Local = entity VDB | `query_local_with_vector_storage` |
| Global = relation VDB | `query_global_with_vector_storage` |
| Hybrid round-robin | `merge_hybrid_contexts` |
| Mix = hybrid + naive | `query_mix_with_vector_storage` (default) |

**Honest note:** EdgeQuake's `Global` docs explicitly disclaim MS GraphRAG community reports (`modes.rs` + `contract_global_mode_semantics.rs`). That is a deliberate LightRAG-class choice, not an unfinished GraphRAG port.

---

## WHY #3 — Why is EdgeQuake not yet "best Hybrid RAG" despite parity features?

**Because parity ≠ superiority. July 2026 winners win on (a) when to use the graph, (b) how to walk it, (c) how to measure it.**

GraphRAG-Bench (ICLR 2026) finding:

```text
Level-1 Fact Retrieval  →  Basic RAG ≥ GraphRAG  (graph adds noise / tax)
Level-2+ Reasoning      →  GraphRAG >> Basic RAG (HippoRAG2 leads)
Token tax               →  LightRAG / MS-GraphRAG can burn 10^4–10^5 tokens
```

EdgeQuake today:

1. **Always-on Mix default** — good for recall, expensive for Level-1 facts.
2. **Intent router exists** (`QueryIntent::recommended_mode`) but maps `Factual → Local` (graph tax) and `Exploratory → Naive` (may under-use graph when synthesis is needed).
3. **No Personalized PageRank** — retrieval is cosine + BFS hops (`graph_hops::edges_within_depth`), not associative activation (HippoRAG2).
4. **Graph quality not measured** — no avg-degree / clustering / evidence-recall dashboards tied to ingest.

**Root:** Feature checklist completed; **retrieval physics** and **eval loop** not yet first-class.

---

## WHY #4 — Why do gaps vs latest LightRAG still exist?

**Because LightRAG (Python) kept shipping product pipeline depth while EdgeQuake invested in Rust production systems (tenancy, AGE, API, PDF).**

| Area | LightRAG latest | EdgeQuake today | Gap owner |
|------|-----------------|-----------------|-----------|
| Chunk strategies | F / R / V / P | Recursive / Fixed / Markdown / Pdf | EdgeQuake missing **V semantic** |
| KG→chunk pick | `KG_CHUNK_PICK_METHOD=VECTOR` default | Score-rank by query embedding over source IDs | Near-parity; LightRAG more explicit |
| Role LLMs | extract / keyword / query / vlm first-class | Present (SPEC-026) but less defaulted in docs | Ops maturity |
| Parse→VLM→extract | Staged pipeline + MinerU/Docling | PDF2md + multimodal services | EdgeQuake strong on PDF; LightRAG broader parsers |
| Storage backends | Many (Neo4j, Milvus, Qdrant, …) | Postgres AGE + pgvector (production law) | Intentional — ops simplicity |
| Mix default | Yes (`base.py`) | Yes (`QueryMode::Mix`) | Aligned |

**Root:** Different investment axes. EdgeQuake wins on **enterprise runtime**; LightRAG wins on **research-to-product retrieval knobs**.

---

## WHY #5 — Why will closing these gaps make EdgeQuake best-in-class?

**Because EdgeQuake already owns the hard production substrate that research systems lack — and the missing pieces are now well-specified by 2026 evidence.**

```text
WHAT EDGEQUAKE ALREADY HAS (moat)
─────────────────────────────────
  • Rust performance + typed pipelines
  • PostgreSQL AGE + pgvector (Azure HorizonDB pattern)
  • Mix + RRF + BM25 + rerank spine
  • Workspace / tenant isolation + RLS
  • PDF page-aware chunking + VLM path
  • Saga-style ingest persistence + recovery APIs
  • Adaptive mode hooks (intent) — needs rewiring, not invention

WHAT TO ADD (evidence-backed, not fashion)
──────────────────────────────────────────
  • Query router aligned to GraphRAG-Bench levels
  • PPR / dual-node retrieval arm (HippoRAG2 physics)
  • Graph quality metrics at ingest time
  • Semantic chunking + VECTOR KG-chunk pick parity
  • Eval harness: Fact / Reason / Summary / Faithfulness
  • Context pruning (PathRAG-style) to kill token tax
```

**Fifth Why answer:** Best Hybrid RAG = **right retrieval for the query** + **dense enough graph** + **cheap enough context** + **measured end-to-end** — on a production substrate. EdgeQuake has the substrate; the plan in [07](./07-IMPROVEMENT-PLAN.md) adds the physics.

---

## 5-Why Chain (compressed)

```text
Why answers fragment?     → Flat chunks lose relations
Why add a graph?          → Encode entities + edges for multi-hop
Why still lose to SOTA?   → Wrong walk (BFS≠PPR), wrong router, no eval
Why behind LightRAG tip?  → Different investment (ops vs retrieval knobs)
Why can EQ become #1?     → Moat + known physics + executable plan
```
