# 017 — Beat LightRAG: First Principles + Architecture Diff

**Priority:** Acc-win ladder (post Phase 1–2)  
**Status:** Acc-win E0–E4 **closed** — persistent Acc **tie**; soft Mix knobs exhausted; no promote  
**Date:** 2026-07-19  
**Cross-ref:** [000 Index](./000-index.md) · [018 E4 Acc-tie close](./018-e4-acc-tie-close.md) · [001 First Principles](./001-first-principles.md) · [011 Acc Report](../011-publication-acc-report.md) · [020 Roadmap](./020-roadmap.md)

**Evidence:** [`T124903Z`](../e2e/artifacts/history/smoke-20260719T124903Z/) · S1 [`T151125Z`](../e2e/artifacts/history/smoke-20260719T151125Z/) · E1–E3b archives · **E4 close:** [018](./018-e4-acc-tie-close.md)

**LightRAG code (peer SUT):** `/Users/raphaelmansuy/Github/03-working/LightRAG` (`lightrag/operate.py`, `utils.py`, `prompt.py`)

---

## 1. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  E4 CLOSE: EQ vs LR Acc = persistent STATISTICAL TIE (all Δ Acc CIs          │
│  include 0). Soft Mix Acc-win ladder exhausted. No headline promotion.       │
│                                                                              │
│  Best labeled package: S1 CE+protect (T151125Z) — ctx_rel 0.519, Acc OK.   │
│  Headline Acc defaults remain BM25 / PRUNE=0 / PROTECT_FIRST=0.              │
│                                                                              │
│  Deferred hard path: truncation/chunk budget (Summarize) · Phase 3 latency.  │
│  Full close: 018-e4-acc-tie-close.md                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Definition of “beat”

Under frozen Acc fairness pins (`MIX_ARM_GATE=false`, `MIX_FUSION=rrf`, chunk 1200/100, related_chunk=5, mistral-small + mistral-embed, top-k=30):

| Requirement | Gate |
|-------------|------|
| Δ Acc 95% bootstrap CI | Excludes 0 in EQ’s favor |
| L2 (preferred) | EQ `context_relevancy` ≥ LR (or absolute ≥ 0.50 stable across ≥2/3 Acc runs) |
| Validity | Empty answer / empty context ≤ fairness thresholds; `valid=true` with L0+L1+L2 |

Partial wins (Acc point estimate only, or L2-only without CI) must be labeled **tie / directional**, never “beats LightRAG.”

---

## 3. EQ vs LightRAG — architecture difference

```text
  LightRAG Mix                         EdgeQuake Mix (Acc)
  ─────────────                        ───────────────────
  LLM hl/ll keywords                   LLM hl/ll keywords
       │                                    │
       ├─ LOCAL  entity VDB cosine         ├─ LOCAL  entity ANN + graph
       ├─ GLOBAL relation VDB cosine       ├─ GLOBAL relation ANN
       └─ NAIVE  chunk VDB cosine          └─ NAIVE  chunk ANN + BM25/FTS
       │                                    │
       ▼                                    ▼
  Round-robin merge + dedupe           Weighted RRF fuse
       │                                    │
       ▼                                    ▼
  KG chunks: VECTOR cosine pick        (kg_chunk_pick vector/weight)
       │                                    │
       ▼                                    ▼
  Entity/rel token truncate            BM25 or CE rerank (+ protect)
       │                                    │
       ▼                                    ▼
  Entities + Relations + Chunks        Degree-sort entities
  (JSON kg_query_context)                   │
                                       Entities → Relations → Chunks
                                       (markdown context_format)
```

| Dimension | LightRAG | EdgeQuake | Effect on Acc / L2 |
|-----------|----------|-----------|---------------------|
| Mix fusion | **Round-robin** (`_merge_all_chunks` in `operate.py`) | **RRF** (`EDGEQUAKE_MIX_FUSION=rrf`; `fusion.rs`) | Different chunk order / consensus bias |
| Lexical arm | **None** (dense-only) | **BM25/FTS** inside arms (`sparse_retrieval.rs`) | EQ keeps term-overlap noise; also helps Fact |
| KG→chunk pick | `pick_by_vector_similarity` (default `VECTOR`) | `kg_chunk_pick.rs` vector/weight; Mix RRF + rerank dominate after | LR scores KG chunks by **query cosine** earlier |
| Graph depth | **1-hop** edges from local entities; endpoints of global edges | Local neighborhood + global; optional `path_prune` | Both ~1-hop; Acc S1 has path **off** |
| Entity order in prompt | VDB / cosine order | **`sort_entities_by_degree`** (`reranking.rs` / `query_pipeline.rs`) | Degree hubs ≠ query relevance → Complex noise |
| Acc rerank | Harness `enable_rerank=false` (`fair_pins.py`) | Headline **BM25**; S1 **CE + PROTECT_FIRST=12** | EQ has an extra admission stage LR Acc lacks |
| Context shape | JSON entities / relations / chunks (`kg_query_context`) | Markdown Entities → Relations → Chunks (`context_format.rs`) | Same three blocks; EQ degree-sort hurts |
| Fairness budgets | top_k / chunk_top_k **30**, related_chunk **5**, chunk 1200/100 | Same | Matched |

### Code anchors

| Concern | LightRAG | EdgeQuake |
|---------|----------|-----------|
| Mix orchestration | `lightrag/operate.py` — `kg_query`, `_perform_kg_search`, `_merge_all_chunks` | `modes/mix.rs` — `query_mix_with_vector_storage`, `fuse_mix_contexts` |
| Chunk pick | `lightrag/utils.py` — `pick_by_vector_similarity` | `kg_chunk_pick.rs`, `modes/chunk_retrieval.rs` |
| Postprocess | `utils.py` — `process_chunks_unified` | `query_pipeline.rs` — `postprocess_retrieved_context` |
| Entity sort | VDB order preserved | `reranking.rs` — `sort_entities_by_degree` |
| Path / prune | soft token budgets on E/R | `path_prune.rs`, `relevancy_prune.rs`, `rerank_protect.rs` |
| Acc pins | `tools/bench001/bench001/fair_pins.py` | same + `acc_env.py` |

---

## 4. First-principles reading of the gap

| Law | Meaning for “beat LR” |
|-----|------------------------|
| **L1** Acc composite | Tied Acc + worse L2 = generator compensation, not retrieval win |
| **L2** Recall ≠ Relevancy | EQ finds evidence (Complex recall=1.0) but packs distractors |
| **L3** Binding constraint **(now)** | After Phase 1–2: **Complex F1** + **stable ctx_rel ≥ LR** — not more Acc fishing |
| **L4** One confound | E0–E4 change one pin/code path each |
| **L5** Fairness | Win under same pins; CE/protect stays labeled until L2 stable + Acc CI win |

**What is *not* the bottleneck:** LLM/embed pin, empty context, chunk size mismatch, upsizing the judge.

### Evidence snapshot (why Complex is #1)

| Archive | EQ Acc | LR Acc | Δ Acc CI | EQ ctx_rel | Complex note |
|---------|--------|--------|----------|------------|--------------|
| `T124903Z` baseline | 0.765 | 0.754 | includes 0 | 0.375 | Complex Acc −6pp |
| `T151125Z` S1 | 0.760 | 0.780 | includes 0 | **0.519** | Complex Acc −8pp; recall=1.0 |
| `T151836Z` Phase 2 | 0.751 | 0.771 | includes 0 | 0.481 | Complex Acc −9pp; F1 −12pp |

CE+protect closed most of the L2 gap but left **Complex F1** and **L2 variance**. LightRAG’s Complex edge is cleaner **relational packing** (cosine-ordered entities/relations, VECTOR KG chunk pick, round-robin without degree hubs) — not a stronger Acc reranker (LR Acc has none).

---

## 5. Binding-constraint ranking (post Phase 2)

| Rank | Gap | Evidence | Fix direction |
|------|-----|----------|---------------|
| **B0** | Complex Reasoning F1 −8 to −12pp | recall=1.0 both sides under S1/P2 | Query-conditioned entity/path packing (not volume) |
| **B1** | ctx_rel unstable at ≥0.50 | 0.519 → 0.481 | Soft path + protect; replicate before promote |
| **B2** | Summarize recall shortfall | EQ ~0.85 vs LR ~0.97 | Naive/global contribution / related-chunk audit |
| **B3** | Overall Acc CI includes 0 | both S1-pin Acc runs | Decision run only after B0–B2 move |
| **B4** | Latency ~3× | p50 ~10s vs ~2–3s | Phase 3 after Acc honesty |

---

## 6. Experiment ladder (E0–E4)

All runs: warm EQ workspace, `BENCH001_QUERY_ONLY=1`, fairness pins frozen, **one confound**, `ABLATION_NOTE.md` in the archive.

```text
  Stabilize L2 (2b)  →  Complex packing  →  Summarize recall  →  Acc CI gate
         │                    │                   │                  │
         ▼                    ▼                   ▼                  ▼
   soft path+protect    query-score ents     related/global      CI excludes 0
   under S1 CE pins     (not degree-first)   coverage only       then promote?
```

| Step | Change (one confound) | Code hooks | Success | Status |
|------|----------------------|------------|---------|--------|
| **E0** | Replicate S1 pins (ladder baseline) | existing CE+protect ([000](./000-index.md)) | ctx_rel ≥0.50 on ≥2/3 runs **or** EQ ≥ LR ctx_rel | Skipped (T151125Z + T151836Z document variance); E1 re-cleared 0.50 |
| **E1** | Soft `PATH_PRUNE=0.4` **with** `PROTECT_FIRST=12` | `path_prune.rs` | ctx_rel ≥0.50; Acc drop ≤0.02; Complex F1 not worse | **Done** [`T153436Z`](../e2e/artifacts/history/smoke-20260719T153436Z/) — ctx_rel 0.519 ✅ · Acc −0.018 ✅ |
| **E2** | **Query-conditioned entity ranking** (`EDGEQUAKE_ENTITY_RANK=query_score`) | `entity_rank.rs` · `query_pipeline.rs` | Complex ΔF1 vs LR ≤ **0.03**; ctx_rel not↓ | **Missed** [`T153959Z`](../e2e/artifacts/history/smoke-20260719T153959Z/) — ΔF1 −0.094; code kept labeled |
| **E3** | Summarize coverage: `RELATED_CHUNK_NUMBER` 5→8 | `kg_chunk_pick` / Acc pin | Summarize recall ≥ **0.95**; Fact Acc not↓ | **Missed** [`T154427Z`](../e2e/artifacts/history/smoke-20260719T154427Z/) — Summarize recall **0.863** flat |
| **E3b** | Mix naive-weight boost `EDGEQUAKE_MIX_NAIVE_WEIGHT=2` | `mix_arm_weight_from_env` · Acc pins | Summarize recall ≥ **0.95**; Fact Acc not↓; ctx_rel not↓ ≥0.05 | **Missed** [`T155350Z`](../e2e/artifacts/history/smoke-20260719T155350Z/) — Summarize **0.882** (+1.8pp); Acc tax |
| **E4** | Acc CI decision / honest close under best labeled pins | docs ([018](./018-e4-acc-tie-close.md)) | Δ Acc CI excludes 0 → promote; else **document persistent tie** | **Done** — persistent **tie**; no promote |

### Acc-win measurement table

| Archive | Confound | EQ Acc | EQ ctx_rel | Key gap metric |
|---------|----------|--------|------------|----------------|
| `T151125Z` S1 | CE+protect path-off | 0.760 | 0.519 | Complex ΔF1 −0.110 |
| `T153436Z` E1 | + path 0.4 | 0.742 | **0.519** | Complex ΔF1 −0.106 |
| `T153959Z` E2 | + `entity_rank=query_score` | 0.734 | **0.519** | Complex ΔF1 −0.094 |
| `T154427Z` E3 | + `related_chunk=8` | 0.752 | 0.506 | Summarize recall 0.863 |
| `T155350Z` E3b | + `MIX_NAIVE_WEIGHT=2` | 0.734 | 0.500 | Summarize recall **0.882** (need 0.95) |

**Reading:** Soft path stabilizes L2. Entity order / related_chunk / naive×2 **cannot** reach Summarize recall 0.95. Soft Mix knobs exhausted → **E4 honesty close** ([018](./018-e4-acc-tie-close.md)). Harder path (truncation/budget) is a **new** labeled ladder, not Acc-win soft knobs.

### S1 package pins (starting point for E0/E1)

```text
EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
EDGEQUAKE_RERANKER=cross_encoder
EDGEQUAKE_RERANKER_PROVIDER=aliyun
EDGEQUAKE_RERANKER_MODEL=qwen3-rerank
EDGEQUAKE_PATH_PRUNE=0          # E1 flips to soft 0.4
EDGEQUAKE_RERANK_PROTECT_FIRST=12
BENCH001_EQ_RERANK_TOP_K=30
```

### Rejected for this ladder

| Change | Why rejected |
|--------|----------------|
| Cosine keep=12 + short CE top_k | Fact Acc cliff (ablation ladder) |
| Protect front-loaded Mix ranks | Wrong order — CE order required (`T150417Z`) |
| Silent `MIX_ARM_GATE=true` on Acc | Breaks fairness (L5) |
| Stack prune + path + CE together | Violates one-confound (L4) |

### Later (after Acc-win E4 close)

- **Truncation / chunk token budget** (Summarize coverage) — `truncation.rs` `balance_context`; one confound; labeled ([013](./013-lens-latency-ops.md), [018 §5](./018-e4-acc-tie-close.md))
- Phase 3 latency — parallel Mix arms, keyword cache ([013](./013-lens-latency-ops.md))
- Phase 4 product type routing — Acc headline stays arms-off ([014](./014-lens-generation-routing.md))
- Phase 5 HippoRAG2-style PPR research profile ([012](./012-lens-multihop-graph.md))

---

## 7. Why this sequence (not more CE)

Ablation ladder already showed: soft CE maximizes ctx_rel but taxes Acc; CE+protect recovers Acc but leaves **Complex F1** and **L2 variance**. Next leverage is **query-conditioned graph packing**, then coverage, then CI — matching GraphRAG-Bench / PathRAG / HippoRAG2 lessons: high relevancy via **selection**, not volume.

---

## 8. Non-goals

- UltraDomain / MMLongBench / paper Table-2 claims under different pins (`P0_paper` is separate)
- Changing LightRAG upstream
- Silent Acc headline promotion without E4 honesty archive ([018](./018-e4-acc-tie-close.md))
- Treating n=40 point-estimate Acc wander as a win

---

## 9. Launch

```bash
# Warm query-only Acc (after Acc backend restart with labeled pins)
export BENCH001_EQ_WORKSPACE_ID=<warm-ws-from-baseline-ingest>
export BENCH001_QUERY_ONLY=1
make bench001-full
make bench001-watch STAGE=smoke
```

Artifacts: `specs/001-benchmark/e2e/artifacts/` → archive `history/smoke-*/` with `ABLATION_NOTE.md`.
