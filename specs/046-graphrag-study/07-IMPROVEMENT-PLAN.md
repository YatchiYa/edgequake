# 07 — Improvement Plan: Best Hybrid RAG on EdgeQuake

**Goal:** Move EdgeQuake from "strong LightRAG-class enterprise Hybrid RAG" to **best-in-class Hybrid RAG (July 2026 definition)** without abandoning the Postgres AGE + Mix spine.

**Law:** Every work item names files/symbols to touch. No vapor features.

---

## North Star Metrics

| Metric | Baseline (today) | Target (90 days) |
|--------|------------------|------------------|
| L1 Fact ACC | Untested systematically | ≥ Naive+rerank (no regression) |
| L2 Reasoning ACC | Untested | +10–15 pts vs Mix-only baseline on GraphRAG-Bench-style set |
| Avg tokens / query (L2) | Mix full context | −30% via prune + router |
| p50 latency L1 | Mix 3-arm | ≤ Naive path when routed |
| Graph avg degree (ops) | Unknown | Instrumented; alert if < 2.0 after ingest |
| Eval in CI | Partial unit/contract | Nightly GraphRAG-Bench subset |

---

## Phase Map

```text
P0 (1–2 weeks)     Fix router + measure + LightRAG parity knobs
P1 (3–6 weeks)     PPR dual-node arm + path prune + dynamic budgets
P2 (6–12 weeks)    Eval harness + optional community reports + semantic chunk V
P3 (ongoing)       Role-LLM defaults, delete-rebuild parity, density tuning
```

---

## P0 — Highest ROI (do first)

### P0.1 Rewire intent → mode (evidence-aligned)

**File:** `edgequake-query/src/keywords/intent.rs`

| Intent | Today | Change to |
|--------|-------|-----------|
| Factual | Local | **Naive** (optional Local verify if confidence low) |
| Exploratory | Naive | **Global** or **Mix** |
| Relational | Global | **Hybrid** or **Mix** |
| Comparative | Local | **Mix** |
| Procedural | Mix | Mix (keep) |

**Also:** Gate behind `EDGEQUAKE_ADAPTIVE_MODE=true` default-on after A/B; keep Mix as explicit API default for non-adaptive clients.

**Tests:** Update `intent.rs` unit tests; add contract that "What is X?" → Naive.

**Why:** GraphRAG-Bench — graphs hurt L1; help L2/L3.

### P0.2 Instrument graph quality at ingest

**Files:**
- `edgequake-storage/src/community.rs` / new `graph_metrics.rs`
- Emit after `KnowledgeGraphMerger` in `ingestion_persister.rs`
- Expose on `GET /health` or workspace stats API

**Metrics:** `|V|`, `|E|`, avg degree, orphan rate, % entities with empty description, embedding coverage.

### P0.3 LightRAG chunk-pick knobs

**Files:** `chunk_retrieval.rs`, `QueryEngineConfig`

Add:
- `related_chunk_number` (cap per entity/relation)
- `kg_chunk_pick_method: Vector | Weight`

Default Vector (current behavior).

### P0.4 Dynamic token remainder (port LR accounting)

**File:** `truncation.rs`

Replace fixed 10k/10k/10k with:

```text
chunk_budget = max_total - sys - query - entity_tokens - relation_tokens - buffer
```

Mirror `operate.py:_build_context_str`.

### P0.5 Document Hybrid naming divergence

**Files:** API OpenAPI descriptions, `modes.rs` docs

State clearly: EQ `hybrid` includes naive; LR `hybrid` does not.

---

## P1 — Retrieval Physics Upgrade

### P1.1 Personalized PageRank retrieval arm

**New module:** `edgequake-query/src/engine_impl/modes/ppr.rs` (or `graph_ppr.rs`)

**Algorithm (HippoRAG2-inspired, pragmatic):**

```text
1. Seed: top-k entities from ll embedding (reuse local arm seeds)
2. Build subgraph: seeds + 2-hop neighborhood from AGE
   OR run PPR in-process on adjacency loaded for workspace slice
3. Score nodes by PPR mass
4. Map entity nodes → passage/chunk nodes via source_chunk_ids
   (dual-node lite: treat chunks as passage nodes linked to entities)
5. Return top passages; fuse into Mix via RRF as 4th arm OR replace BFS in Local
```

**Storage:** Prefer in-process PPR on fetched subgraph first (ship fast). AGE native later if needed.

**Config:** `EDGEQUAKE_GRAPH_WALK=bfs|ppr` default `bfs` until eval passes.

### P1.2 Dual-node lite (phrase ↔ passage)

**Ingest change:** When upserting entities, ensure undirected edges `ENTITY --mentions--> CHUNK` are queryable cheaply (may already exist via `source_chunk_ids` — promote to first-class graph edges if missing).

**Query:** PPR over bipartite entity–chunk graph (HippoRAG2 dual-node essence without full OpenIE rewrite).

### P1.3 Path / context prune

**File:** new `edgequake-query/src/path_prune.rs`

After retrieving relations, score path flow (degree-normalized edge weight × embedding sim); drop bottom 40% before prompt (PathRAG-inspired).

### P1.4 Cross-encoder rerank default path

**File:** `bootstrap.rs`

Keep BM25 rerank; document `EDGEQUAKE_RERANKER=cross_encoder` as recommended prod; add local model hook (bge-reranker class).

---

## P2 — Parity + Eval + Optional Global Power

### P2.1 Semantic chunking (LightRAG V)

**Files:** `chunker/registry.rs`, new `chunker/semantic.rs`

Port strategy V: embedding breakpoint chunking. Feature-flag `ChunkStrategy::Semantic`.

### P2.2 GraphRAG-Bench-style eval harness

**New:** `edgequake/tests/graphrag_bench/` or `specs/046-graphrag-study/eval/`

Levels:
1. Fact retrieval
2. Complex reasoning
3. Context summary
4. Faithfulness

Wire to `cargo test` nightly + JSON report. Start with synthetic fixtures from existing `test_fixtures.rs`, then import GraphRAG-Bench subset.

### P2.3 Optional community reports (L3-only)

**Not default.** Background job: summarize communities → vector index `type=community_report`.  
Router sends L3 Exploratory/Summary intents to Global+reports.

**Files:** extend `community_persist.rs`; new query path in `global.rs`.

### P2.4 Delete → rebuild parity

Port LightRAG `rebuild_knowledge_from_chunks` semantics into `orchestrator/deletion.rs` for entities whose only sources were deleted.

---

## P3 — Extraction Quality & Ops

### P3.1 Role-LLM defaults (SPEC-026 completion)

Document and default:
- Extract: stronger/slower model
- Keyword: small/fast
- Query: long-context strong
- VLM: vision model

Mirror LightRAG `RoleSpecificLLMConfiguration.md`.

### P3.2 Extraction density tuning

- Increase gleaning selectively on low-degree docs
- Domain entity-type profiles (YAML) like LR `ENTITY_EXTRACTION_PROMPT_PROFILE`
- Measure: target avg degree band 3–8 for knowledge-dense corpora

### P3.3 Process-options fingerprint + stale purge

On reanalyze, if chunker/extract options changed, purge stale KG artifacts (LR `_purge_stale_extraction_if_resuming`).

### P3.4 Multimodal → KG injection guarantee

Ensure VLM table/drawing/equation chunks create entities + edges (LR `operate.py` multimodal inject). Audit SPEC-026 end-to-end.

---

## What We Explicitly Will NOT Do

| Anti-goal | Reason |
|-----------|--------|
| Replace Mix with MS GraphRAG default | L1 regression + index cost |
| Multi-backend zoo (Milvus+Neo4j+…) | Dilutes Postgres moat |
| Rewrite in Python | Loses systems moat |
| Cargo-cult every ICLR paper | Only import measured physics |

---

## Ticket-Sized Backlog (copy into tracker)

```text
[ ] EQ-046-01  Rewire QueryIntent::recommended_mode + tests
[ ] EQ-046-02  Graph metrics module + ingest emit + API field
[ ] EQ-046-03  related_chunk_number + kg_chunk_pick_method
[ ] EQ-046-04  Dynamic token remainder in truncation.rs
[ ] EQ-046-05  Docs: hybrid mode naming vs LightRAG
[ ] EQ-046-06  PPR walk behind EDGEQUAKE_GRAPH_WALK
[ ] EQ-046-07  Bipartite entity–chunk edges for dual-node lite
[ ] EQ-046-08  path_prune.rs before balance_context
[ ] EQ-046-09  ChunkStrategy::Semantic
[ ] EQ-046-10  GraphRAG-Bench subset harness
[ ] EQ-046-11  Optional community_report vectors (L3)
[ ] EQ-046-12  rebuild_knowledge_from_chunks on delete
[ ] EQ-046-13  Role-LLM default matrix in docs + config
[ ] EQ-046-14  Stale purge on process_options change
[ ] EQ-046-15  Multimodal entity injection audit
```

---

## Success Definition ("Best Hybrid RAG")

EdgeQuake is best-in-class when:

1. **Router** sends L1 → Naive+BM25+rerank with no quality loss.
2. **L2** uses PPR dual-node (or proven equal) inside Mix/RRF.
3. **Tokens** on L2/L3 drop ≥30% via prune without ACC loss.
4. **Eval** runs in CI and beats prior Mix baseline.
5. **Enterprise** properties (tenancy, AGE, PDF, saga) remain intact.

Until then: honest label is **"production Hybrid RAG (LightRAG-class) with a clear path to SOTA retrieval physics."**
