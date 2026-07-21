# 012 — Lens: Multi-Hop / Graph Selection (P2)

**Priority:** P2 — reasoning quality via selection, not volume  
**Cross-ref:** [010 Retrieval noise](./010-lens-retrieval-noise.md) · SPEC-046 GraphRAG study

---

## 1. Observation

| Question type | EQ Acc | LR Acc | Leader |
|---------------|--------|--------|--------|
| Fact Retrieval | **0.752** | 0.654 | EQ |
| Complex Reasoning | 0.715 | **0.776** | **LR (−6pp)** |
| Contextual Summarize | 0.836 | **0.866** | LR |
| Creative Generation | **0.755** | 0.720 | EQ |

EQ is competitive on local fact lookup and creative faithfulness; LR wins when multi-hop synthesis needs **cleaner relational paths**.

---

## 2. First-principles diagnosis

- Multi-hop fails when C contains the right nodes **plus** distractor edges that steer the generator.
- Fix is **query-conditioned path selection**, not larger entity/rel caps.
- GraphRAG-Bench: graphs help Level 2–3 (reasoning / summarize); fact lookup often prefers lean vector RAG.

---

## 3. July 2026 practice

| System | Lesson for EQ |
|--------|----------------|
| **PathRAG** | Flow-based prune; keep relational paths; order paths for lost-in-the-middle |
| **HippoRAG2** | PPR over passage+phrase nodes; high relevancy with compact context |
| **LightRAG** | Soft prune of irrelevant entities; dual-level keywords |
| Product consensus | Route: vector-lean for facts; path/graph for multi-hop — not always-on full GraphRAG |

Research track (labeled, not Acc headline): HippoRAG2-inspired PPR / dual-node indexing under same LLM/embed pins.

---

## 4. EQ insertion points

| Hook | File | Action |
|------|------|--------|
| Relationship prune | `edgequake-query/src/path_prune.rs` | Tune `PathPruneConfig` (drop_fraction, enable) for Mix; measure Reasoning Acc + ctx_rel |
| Postprocess order | `query_pipeline.rs` `postprocess_retrieved_context` | After path_prune, before `balance_context`: drop orphan entities not on kept paths |
| Mix graph union | `modes/mix.rs` | Prefer query-scored entities over first-seen union; port `prune_empty_arm_graph` |
| Local expand | `modes/local.rs` | Cap hop expansion; prefer on-query seeds |
| Global expand | `modes/global.rs` | Same — high-degree global hubs are noise magnets |
| Prompt structure | `context_format.rs` | Keep Entities → Relations → Chunks; optionally path-serialized blocks (PathRAG-style) as ablation |

---

## 5. Experiments (one confound each)

| # | Change | Success |
|---|--------|---------|
| M1 | Path-prune only (after P0 chunk prune stable) | Reasoning Acc ≥ LR − 0.02; ctx_rel ≥ 0.50 |
| M2 | Orphan entity drop after path prune | Prompt entity count↓; Reasoning Acc not↓ |
| M3 | Path-ordered prompt serialization | Reasoning/Summarize Acc↑; Fact Acc not↓ ≥ 0.02 |
| M4 | Research: PPR dual-node retrieval profile | Labeled `P_research_ppr`; L2 toward HippoRAG2 band under same pins |

**Dependency:** Run M1+ after [010](./010-lens-retrieval-noise.md) E1 clears, or as a pure path_prune ablation with fixed chunk list.

---

## 6. Non-goals

- Do not increase `max_entities` / `max_relationships` “to help reasoning” without relevancy gates.
- Do not replace Mix RRF with a research PPR path in the Acc **headline**.
- Do not conflate product intent-routing (014) with Acc fairness pins (arm gate must stay off for publish Acc).
