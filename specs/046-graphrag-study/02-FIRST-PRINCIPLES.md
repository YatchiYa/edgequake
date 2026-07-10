# 02 — First Principles of Hybrid RAG (July 2026)

**Lens:** Strip marketing. Keep only physics that survive GraphRAG-Bench + HippoRAG2 + production Hybrid Search practice.

---

## Axioms (irreducible)

### A1 — Retrieval is a matching problem under a budget

```text
maximize  P(answer correct | context)
subject to  tokens(context) ≤ B
            latency ≤ L
            $cost ≤ C
```

Every GraphRAG feature that increases tokens without increasing evidence recall is **negative EV**.

### A2 — Chunks are evidence; graphs are indexes

- **Chunks** (passages) are what the LLM should cite.
- **Entities / relations / communities** are **indexes** that help find the right chunks.
- Systems that dump huge entity/relation JSON into the prompt without chunk evidence pay the Graph Tax (GraphRAG-Bench: LightRAG ~1e5 tokens on Novel).

### A3 — Query difficulty is 2D, not "multi-hop" marketing

```text
                 REASONING DIFFICULTY →
              Low                    High
         ┌─────────────┬─────────────────────┐
  R   Low│ L1 Fact     │ L3 Summary          │
  E      │ Naive wins  │ Graph helps         │
  T      ├─────────────┼─────────────────────┤
  R  High│ L2 Multi-hop│ L4 Creative         │
  I      │ Graph wins  │ Graph + prune       │
  E      └─────────────┴─────────────────────┘
  V
```

**Corollary:** One default mode cannot be optimal for all cells. Router is mandatory.

### A4 — Graph value is mediated by graph quality

GraphRAG-Bench: higher **average degree** and **clustering** correlate with better recall (HippoRAG2 denser than LightRAG).  
A sparse, noisy extraction graph is worse than no graph.

### A5 — Hybrid = orthogonal signals fused, not "more modes"

Production Hybrid Search 2026 minimum viable stack:

```text
Dense (embedding)  ─┐
Sparse (BM25/FTS)  ─┼─► Fusion (RRF) ─► Rerank ─► Truncate ─► Generate
Graph (paths/PPR)  ─┘
```

EdgeQuake already has dense + sparse + graph arms and RRF (`fusion.rs`, `sparse_retrieval.rs`).  
**v0.16.0:** strong graph walk (**PPR default** + bipartite dual-node) and **difficulty-aware routing** (Factual→Naive + Mix/Hybrid arm gate) are **shipped**. Remaining physics gap: true cross-encoder rerank + full HF GraphRAG-Bench ACC.

### A6 — Incremental index > full rebuild

LightRAG paper contribution #2. Both codebases merge by entity name and accumulate `source_id`.  
Delete/rebuild correctness is part of the product (LightRAG `rebuild_knowledge_from_chunks`; EdgeQuake cascade delete + reanalyze).

### A7 — Code is law

Specs and papers are hypotheses. Behavior is defined by:

| Concern | EdgeQuake | LightRAG |
|---------|-----------|----------|
| Default query mode | `QueryMode::Mix` | `QueryParam.mode = mix` |
| Global semantics | Relation vectors + community_id expand | Relation vectors (no community reports) |
| Fusion | RRF default for Mix | Round-robin merge of C/E/R chunks |

### A8 — Silent degradation is negative EV (ops axiom, 2026-07-10)

Any path that **warns and continues** (Semantic→Recursive, HNSW create `.ok()`, embedding truncate, stub repair `200 OK`) increases `P(answer looks fine | evidence broken)`.  
July 2026 production RAG practice treats empty retrieval, truncation, and strategy downgrade as **first-class telemetry**, not log noise.

### A9 — Storage complexity contract

| Hot path | Max complexity |
|----------|----------------|
| Vector ANN | O(log N) with live HNSW |
| Graph expand | O(k · degree · hops) batched |
| Community on ingest | O(sample) capped — **never** full `get_all_nodes` |
| Mix arms | Router may zero arms; forced 3-arm is a cost choice |

---

## First-Principles Decomposition of a Query

```text
q ──► Classify difficulty (L1..L4)
   │
   ├─ L1 ──► Dense + Sparse ──► Rerank ──► Generate
   │         (skip graph; avoid tax)
   │
   ├─ L2 ──► Seed entities ──► PPR / hops ──► Passage nodes
   │         ──► Fuse with dense/sparse ──► Rerank ──► Generate
   │
   ├─ L3 ──► Theme/relation vectors + (optional) community summaries
   │         ──► Aggressive prune ──► Generate
   │
   └─ L4 ──► Same as L3 + higher creativity temperature
             + faithfulness check against evidence
```

**EdgeQuake v0.16.0:** Adaptive intent routing + Mix/Hybrid **arm gate** skip graph tax on L1 when configured; default API mode remains Mix. Graph walk defaults to **PPR** (`EDGEQUAKE_GRAPH_WALK=bfs` escape). That approximates "L2-capable by default" with an L1 off-ramp — closer to HippoRAG2 physics than always-3-arm Mix, still short of full dual-node AGE store + cross-encoder.

---

## First-Principles Decomposition of Ingest

```text
Document
  │
  ├─ Parse (layout fidelity)     ← quality ceiling for everything below
  ├─ Chunk (semantic coherence)  ← bad chunks ⇒ bad entities ⇒ bad graph
  ├─ Extract (precision/recall)  ← gleaning, schema, JSON vs tuple
  ├─ Merge (dedupe + summarize)  ← graph density & description quality
  ├─ Embed (chunk/entity/rel)    ← retrieval surface
  └─ Measure (degree, clustering, orphan rate)  ← **shipped** `graph_metrics` + Prometheus `record_graph_quality`
```

**Implication:** Improving extraction prompts without measuring graph quality is cargo cult — v0.16 closes the measure gap; density tuning YAML remains open.

---

## What "Best Hybrid RAG" Means (operational definition)

A system is best-in-class if, on a dual-density corpus (medical + narrative) with GraphRAG-Bench-style levels:

| Metric | Target |
|--------|--------|
| L1 Fact ACC | ≥ Basic RAG + rerank (no regression) |
| L2 Reasoning ACC | ≥ HippoRAG2 ballpark |
| L3 Summary ACC | ≥ LightRAG / HippoRAG2 |
| Avg tokens / query | ≪ LightRAG global tax (prefer ~1e3–1e4) |
| p95 latency | Competitive with Mix today after router |
| Incremental ingest | No full rebuild |
| Multi-tenant ops | Workspace isolation (EdgeQuake strength) |

EdgeQuake's path is not "copy HippoRAG in Python" — it is **import the physics into the Rust Mix spine**.
