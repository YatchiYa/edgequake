# 04 — Query Pipeline Compare (Code is Law)

**Lens:** Mode semantics, fusion, context assembly, routing honesty.  
**Critical finding:** EdgeQuake intent→mode mapping is cost-optimized but partially misaligned with GraphRAG-Bench evidence.

---

## End-to-End ASCII

```text
                              QUERY
 ═══════════════════════════════════════════════════════════════════

  q ──► Keyword LLM ──► {hl_keywords, ll_keywords, intent?}
     │
     ├── Embed(q)           → naive arm
     ├── Embed(ll_keywords) → local arm (entities)
     └── Embed(hl_keywords) → global arm (relations)

                    ┌──────── Mode Router ────────┐
                    │ Naive | Local | Global      │
                    │ Hybrid (RR interleave)      │
                    │ Mix (weighted / RRF) ★def   │
                    │ Bypass                      │
                    └─────────────┬───────────────┘
                                  ▼
              ┌───────────────────────────────────┐
              │ Retrieve entities / relations /   │
              │ chunks (+ BM25 fuse on EQ)        │
              └─────────────────┬─────────────────┘
                                ▼
              ┌───────────────────────────────────┐
              │ Enrich: filter → rerank → truncate│
              │ (30k token budget both)           │
              └─────────────────┬─────────────────┘
                                ▼
                         Prompt + LLM answer
```

---

## Mode Semantics Truth Table

| Mode | EdgeQuake behavior | LightRAG behavior | Parity |
|------|--------------------|-------------------|--------|
| **naive** | Chunk VDB + optional BM25 | Chunk VDB only (+ rerank) | **EQ +sparse** |
| **local** | Entity VDB(ll) → BFS hops → source chunks score-rank | Entity VDB(ll) → incident edges → chunks | Near; EQ has explicit BFS depth |
| **global** | Relation VDB(hl) → hydrate entities → **community_id expand** → chunks | Relation VDB(hl) → endpoint entities → chunks | **EQ +community labels** |
| **hybrid** | Local ∥ Global ∥ Naive → **round-robin** | Local ∥ Global → round-robin (**no naive**) | **Divergent** |
| **mix** ★ | Local ∥ Global ∥ Naive → **RRF / weighted** | Local ∥ Global ∥ Naive → round-robin C/E/R | Same arms; **EQ better fusion** |
| **bypass** | Direct LLM | Direct LLM | Parity |

★ Default both sides: **mix**.

**Important divergence:** LightRAG `hybrid` = local+global only. EdgeQuake `hybrid` also pulls naive (`query_hybrid_with_vector_storage`). Naming collision — document in API carefully.

---

## Dual-Level Retrieval (paper §3.2)

```text
Paper                          EdgeQuake                         LightRAG
─────                          ─────────                         ────────
local keywords k^(l)    →      QueryEmbeddings.low_level   →     ll_keywords
global keywords k^(g)   →      QueryEmbeddings.high_level  →     hl_keywords
match entities          →      VectorType::Entity          →     entities_vdb
match relations         →      VectorType::Relationship    →     relationships_vdb
```

Both implement the paper correctly at the **index key** level. Neither implements MS GraphRAG community **reports** as retrieval units.

---

## Chunk Selection After KG Hit

### LightRAG (explicit)

```text
KG_CHUNK_PICK_METHOD = VECTOR | WEIGHT   (default VECTOR)
related_chunk_number = 5

_find_related_text_unit_from_entities / _from_relations
  → pick_by_vector_similarity OR pick_by_weighted_polling
```

### EdgeQuake

```text
append_score_ranked_chunks (chunk_retrieval.rs)
  1. Collect source_chunk_ids from entities + relations
  2. vector_storage.query_filtered(embedding, ids=...)
  3. Optional BM25 fuse
```

**Assessment:** Same physics as LightRAG VECTOR pick. EdgeQuake lacks WEIGHT polling and `related_chunk_number` as a first-class knob — add for parity/tuning.

---

## Fusion & Rerank

| Feature | EdgeQuake | LightRAG |
|---------|-----------|----------|
| Mix fusion | RRF default (`fusion.rs`, `EDGEQUAKE_MIX_FUSION`) | Round-robin merge |
| Sparse | BM25/FTS default-on | Not in core path |
| Rerank | BM25 reranker default; cross-encoder opt | Cohere/Jina/Aliyun bindings |
| Token budget | 30k total; 10k×3 balance | 6k entity / 8k rel / dynamic chunk remainder |

**EQ advantage:** RRF + BM25 is 2026 production Hybrid Search best practice.  
**LR advantage:** Dynamic chunk remainder after KG truncation is cleaner accounting (`_build_context_str`).

---

## Intent Router — Honest Critique

EdgeQuake `QueryIntent::recommended_mode` (`keywords/intent.rs`):

| Intent | Recommended mode | GraphRAG-Bench alignment |
|--------|------------------|--------------------------|
| Factual | **Local** | **Misaligned** — L1 facts should prefer **Naive** (avoid graph tax) |
| Relational | Global | OK for L2 |
| Exploratory | **Naive** | **Risky** — "tell me about / overview" often needs Global/Mix (L3) |
| Comparative | Local | Partial — often needs Mix |
| Procedural | Mix | OK |

```text
CURRENT (cost-first)                 EVIDENCE-ALIGNED (proposed)
────────────────────                 ───────────────────────────
Factual     → Local                  Factual     → Naive (+ optional Local verify)
Exploratory → Naive                  Exploratory → Global or Mix
Relational  → Global                 Relational  → Local+Global (Hybrid) or PPR
Comparative → Local                  Comparative → Mix
Procedural  → Mix                    Procedural  → Mix / Naive+Local
```

**This is the highest-ROI one-file fix in the query stack** — see [07](./07-IMPROVEMENT-PLAN.md) P0.

---

## Graph Walk: BFS vs Personalized PageRank

```text
EdgeQuake today
───────────────
  seed entities (vector)
       │
       ▼
  edges_within_depth (BFS, graph_depth)
       │
       ▼
  collect neighbor entities + edges
       │
       ▼
  score-rank source chunks

HippoRAG2 physics (missing)
───────────────────────────
  seed entities (vector / NER)
       │
       ▼
  Personalized PageRank over
  phrase nodes ↔ passage nodes
       │
       ▼
  top passages by PPR mass
       │
       ▼
  LLM recognition filter (optional)
```

BFS is **local neighborhood**. PPR is **associative memory** — better for multi-hop without exploding hop depth. EdgeQuake's AGE backend can implement PPR in Cypher/SQL or in-process on a subgraph snapshot.

---

## Context Assembly Format

Both emit roughly:

```text
### Entities
### Relationships
### Document Chunks
```

EdgeQuake: `QueryContext::to_context_string` + `balance_context`.  
LightRAG: `kg_query_context` / `naive_query_context` prompts + dynamic budgets.

**PathRAG lesson (2026):** prune low-flow paths before prompt. Neither system has flow-based pruning — opportunity for token tax reduction.

---

## Query Scorecard (honest)

| Dimension | EdgeQuake | LightRAG latest | Notes |
|-----------|:---------:|:---------------:|-------|
| Mode coverage | 5/5 | 5/5 | Both 6 modes |
| Default Mix | 5/5 | 5/5 | Aligned |
| Fusion quality | 5/5 | 3/5 | **EQ RRF wins** |
| Sparse hybrid | 5/5 | 2/5 | **EQ** |
| Dual-level keywords | 4/5 | 5/5 | LR cache/roles mature |
| Graph walk power | 2/5 | 2/5 | Both vector+expand; no PPR |
| Community reports | 1/5 | 0/5 | EQ has labels only |
| Intent routing | 2/5 | 0/5 | EQ has it but miswired |
| Token accounting | 3/5 | 5/5 | LR dynamic remainder |
| Rerank ecosystem | 3/5 | 4/5 | LR more bindings |
| Eval harness | 2/5 | 2/5 | Both weak vs GraphRAG-Bench |
| **Overall query** | **3.7** | **3.5** | EQ slightly ahead on fusion; both behind HippoRAG2 physics |

---

## Code Citations (query)

| Claim | EdgeQuake | LightRAG |
|-------|-----------|----------|
| Modes | `modes.rs` | `base.py:QueryParam` |
| Pipeline | `query_pipeline.rs` | `lightrag.py:aquery_llm` |
| Local | `modes/local.rs` | `operate.py:_get_node_data` |
| Global | `modes/global.rs` | `operate.py:_get_edge_data` |
| Mix | `modes/mix.rs` | `kg_query` + `_get_vector_context` |
| Hybrid merge | `hybrid_merge.rs` | round-robin in `_perform_kg_search` |
| RRF | `fusion.rs` | — |
| BM25 | `sparse_retrieval.rs` | — |
| Keywords | `keywords/llm_extractor.rs` | `extract_keywords_only` |
| Intent | `keywords/intent.rs` | — |
| Chunk pick | `chunk_retrieval.rs` | `_find_related_text_unit_*` |
| Truncation | `truncation.rs` | `_apply_token_truncation` |
| Community expand | `community_global.rs` | — |
