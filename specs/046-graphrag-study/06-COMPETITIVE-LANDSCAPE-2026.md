# 06 — Competitive Landscape (July 2026)

**Lens:** Where EdgeQuake sits among GraphRAG / Hybrid RAG systems that matter in 2026.  
**Refresh:** EdgeQuake **v0.16.0** — PPR-default + bipartite + ACC CI + fail-closed ops (see [00](./00-INDEX.md)).

---

## Family Tree

```text
                        RAG
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
   Naive / Hybrid    Tree / Hierarchy   Knowledge Graph
   (dense+sparse)    (RAPTOR)           RAG family
         │               │               │
         │               │     ┌─────────┼──────────┬────────────┐
         │               │     ▼         ▼          ▼            ▼
         │               │  MS GraphRAG LightRAG  HippoRAG2   PathRAG
         │               │  (communities)(dual-lvl)(PPR+dual) (flow prune)
         │               │     │         │
         │               │     │         ├── EdgeQuake (Rust port+)
         │               │     │         └── LazyGraphRAG
         └───────────────┴─────┴──────────────────────────────────┘
                              Hybrid production stacks
                     (dense + sparse + graph + router + rerank)
```

---

## System Cards

### LightRAG (paper 2024 → code 2026)

- **Idea:** Entity/relation graph + dual-level keyword retrieval; incremental merge.
- **Strength:** Simple, fast vs MS GraphRAG; huge OSS ecosystem; mix mode; multimodal pipeline.
- **Weakness:** Token tax high on some benches; graph walk is still vector+expand; sparse hybrid weak.
- **EdgeQuake relation:** Direct ancestor. EQ ≈ LightRAG physics + Rust + Postgres + Mix/RRF/BM25.

### Microsoft GraphRAG

- **Idea:** Leiden communities + hierarchical **community reports**; local/global/DRIFT/basic search.
- **Strength:** Best-in-class **global thematic** synthesis (L3).
- **Weakness:** Index cost extreme; often worse on L1 facts; Microsoft docs admit Basic Search needed.
- **EdgeQuake relation:** Explicitly **out of scope** for Global mode today. Optional future arm for L3-only.

### HippoRAG / HippoRAG2 (NeurIPS'24 / 2025)

- **Idea:** OpenIE triples + **Personalized PageRank**; HippoRAG2 adds **phrase↔passage dual nodes** + online LLM recognition.
- **Strength:** Best multi-hop / associative memory; strong L1 retention; low query tokens (~1e3).
- **Weakness:** Less enterprise packaging; Python research UX.
- **EdgeQuake relation:** **Highest-value physics to import** into Mix as a new retrieval arm.

### PathRAG

- **Idea:** Flow-based pruning of relational paths → ~44% less context.
- **Strength:** Token efficiency without killing accuracy.
- **EdgeQuake relation:** Apply as post-retrieval prune before `balance_context`.

### RAPTOR

- **Idea:** Recursive tree summarization of chunks.
- **Strength:** Creative / summary faithfulness in some benches.
- **EdgeQuake relation:** Optional summary index; not core KG.

### LazyGraphRAG

- **Idea:** Defer expensive graph work; query-time structure.
- **Strength:** Cost control.
- **EdgeQuake relation:** Aligns with "router skips graph on L1".

### Naive RAG + Rerank

- **Idea:** Dense (+sparse) + cross-encoder.
- **Strength:** Wins L1 on GraphRAG-Bench; cheapest.
- **EdgeQuake relation:** Already inside Mix/Naive; must remain first-class, not second-class citizen.

---

## GraphRAG-Bench Lessons (ICLR 2026)

Source: https://github.com/GraphRAG-Bench/GraphRAG-Benchmark

1. **Do not use graphs for L1 facts** — they add noise.
2. **Graphs help L2/L3** when density is high.
3. **Density mediates quality** — HippoRAG2 denser ⇒ better recall.
4. **Measure three stages:** graph quality → retrieval quality → generation quality.
5. **Token tax is real** — LightRAG/MS can burn 1e4–1e5 tokens.

---

## Positioning Map

```text
                 Enterprise / Multi-tenant readiness →
            Low                         High
         ┌──────────────┬────────────────────────────┐
  High   │ HippoRAG2    │  ★ TARGET: EdgeQuake       │
  Multi- │ (science)    │    after PPR + router +    │
  hop    │              │    eval harness            │
  ACC    ├──────────────┼────────────────────────────┤
         │ LightRAG     │  EdgeQuake TODAY           │
  Med    │ MS GraphRAG  │  (Mix+RRF+BM25+AGE)        │
         ├──────────────┼────────────────────────────┤
  Low    │ Scripts      │  Naive+Rerank SaaS         │
         └──────────────┴────────────────────────────┘
```

**Strategic target:** Top-right cell — HippoRAG2-class retrieval on EdgeQuake's enterprise substrate.

**v0.16.0 position (code is law):** EdgeQuake moved **up** (PPR-default + bipartite + ACC CI) and **right** (fail-closed ops). Still short of HippoRAG2 on full GraphRAG-Bench download ACC and true cross-encoder; ahead of LightRAG core on sparse fusion + enterprise ops.

---

## What NOT to Copy Blindly

| Temptation | Why not |
|------------|---------|
| Full MS community reports as default | Index cost; L1 regression; paper/docs warn |
| Replace Mix with PPR-only | Loses L1 / sparse wins |
| Abandon Postgres for Neo4j "because GraphRAG" | EQ's AGE+pgvector matches Azure HorizonDB pattern; switching is product risk |
| Add every LightRAG storage backend | Dilutes ops focus |

---

## References

- LightRAG: arXiv:2410.05779 — local `lighrad-2410-05779v3.md`
- HippoRAG2: arXiv:2502.14802
- GraphRAG-Bench: ICLR 2026 OpenReview `i9q9xDMjG7`
- Hybrid Search 2026 practice: dense+sparse+graph+RRF+cross-encoder
- Azure HorizonDB Graph-Augmented RAG: pgvector + AGE patterns
