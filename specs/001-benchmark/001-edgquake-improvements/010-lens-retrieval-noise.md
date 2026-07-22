# 010 — Lens: Retrieval Noise / Context Relevancy (P0)

**Priority:** P0 — binding constraint (**S1 cleared** on labeled package)  
**Cross-ref:** [001 First Principles](./001-first-principles.md) · [011 Acc Report §4](../011-publication-acc-report.md) · [020 Roadmap §1b](./020-roadmap.md)

---

## 1. Observation

### Baseline (`smoke-20260719T124903Z`)

| Metric | EQ | LR | Gap |
|--------|----|----|-----|
| context_relevancy | **0.375** | **0.544** | **−0.169** |
| evidence_recall | 0.928 | 0.991 | −0.063 |
| Acc | 0.765 | 0.754 | +0.011 (tie) |

EQ **found** evidence but **packed noise** (graph neighbors, redundant entities/relations, low-utility chunks). Acc stayed competitive because the gold-style generator compensated — L1 violation if we celebrate Acc alone.

### S1 package (`smoke-20260719T151125Z`)

| Metric | EQ | vs baseline | Budget |
|--------|----|-------------|--------|
| context_relevancy | **0.519** | +0.144 | ≥0.50 ✅ |
| evidence_recall | **0.928** | ~0 | ≤0.03 drop ✅ |
| Acc | **0.760** | −0.004 | ≤0.02 drop ✅ |

Soft CE+cosine still holds the **max raw ctx_rel** (0.544 on `T142841Z`) but fails Acc/recall companion budgets. The S1 package trades ~2.5pp ctx_rel for Acc recovery.

---

## 2. First-principles diagnosis

- **Violated law (baseline):** L2 (Recall ≠ Relevancy) and L3 (binding constraint is relevancy).
- **Mechanism:** Mix fuses local + global + naive via RRF, then unions entities/relations first-seen. Prompt formatting emits Entities → Relations → Chunks. Volume without query-conditioned admission control → mid relevancy.
- **What worked:** Cross-encoder reorder (`qwen3-rerank`) raises relevancy; **CE-order protect inclusion** (`PROTECT_FIRST=12`) keeps Mix top-12 in the set so Complex/Summarize Acc does not collapse when CE buries diverse evidence.
- **What failed:** Cosine keep=12 *then* aggressive CE top_k (double-cut) → Fact F1 collapse; front-loading Mix ranks ahead of CE order → Acc regression.

---

## 3. July 2026 practice

| Practice | Implication for EQ |
|----------|-------------------|
| Hybrid → RRF → **cross-encoder** → keep top-k for LLM | Shipped: DashScope intl `qwen3-rerank` via `bootstrap.rs` |
| PathRAG flow-based path keep | Shipped: query-conditioned `path_prune` + optional orphan entities (S1 package leaves path **off**) |
| Guaranteed first-stage slots under CE | Shipped: `rerank_protect.rs` — CE order preserved; Mix top-N forced into set |
| HippoRAG2 high relevancy + compact prompts | Still open: tighter `balance_context` after S1 |
| Rule of thumb: retrieve 20–50, rerank, keep | Acc uses retrieve 30 → CE → protect blend → top_k 30 |

References: PathRAG ([arXiv:2502.14902](https://arxiv.org/abs/2502.14902)); GraphRAG-Bench ([arXiv:2506.05690](https://arxiv.org/abs/2506.05690)); hybrid+CE ([Jacar 2026](https://jacar.es/en/hybrid-rag-in-2026-the-patterns-that-keep-winning/)).

---

## 4. EQ insertion points (shipped)

```text
pipeline_retrieve(Mix) → fuse_mix_contexts (RRF)
       │
       ├─ HOOK A: relevancy prune (rrf) / empty-arm graph prune
       ▼
pipeline_finalize → postprocess_retrieved_context
       │
       ├─ filter_context_by_document_ids
       ├─ HOOK B: cosine relevancy prune (env)     ← relevancy_prune.rs
       ├─ rerank_chunks + PROTECT_FIRST blend      ← reranking.rs + rerank_protect.rs
       ├─ path_prune (query-conditioned, env)      ← path_prune.rs
       ├─ HOOK C: Mix prune_empty_arm_graph
       └─ balance_context (token caps)
```

| Hook | File | Env / status |
|------|------|----------------|
| **A** Mix fuse prune | `modes/mix.rs` | `EDGEQUAKE_MIX_RELEVANCY_*` (rrf path); empty-arm graph |
| **B** Cosine prune | `query_pipeline.rs` / `relevancy_prune.rs` | `SCORE=cosine` — helps L2 alone; not in S1 package |
| **CE** Cross-encoder | `bootstrap.rs` + `reranking.rs` | `EDGEQUAKE_RERANKER=cross_encoder` + DashScope intl |
| **Protect** | `rerank_protect.rs` | `EDGEQUAKE_RERANK_PROTECT_FIRST=12` — **S1 key** |
| **Path** | `path_prune.rs` | `EDGEQUAKE_PATH_PRUNE_*` — off in S1 package |

---

## 5. Experiments (one confound each)

| # | Change | Success | Status |
|---|--------|---------|--------|
| E1 | Post-RRF embed-cosine keep top-m + floor | ctx_rel ≥ 0.50; Acc/recall budgets | Partial — ctx_rel 0.456 (`T140420Z`); Acc drop open |
| E2 | Stronger path_prune only | Same gates | Partial — helps L2 with CE; Acc package uses path **off** |
| E3 | Port `prune_empty_arm_graph` into Mix | ctx_rel↑ without recall collapse | Shipped |
| E4 | Cross-encoder on fused top-30 | ctx_rel ≥ 0.50; latency noted | Partial alone (`T145634Z` Acc 0.709) |
| E4b | CE + `PROTECT_FIRST=12` + path off + top_k=30 | All three S1 budgets | **Green** `T151125Z` |
| E5 | Phase 2: Acc + bootstrap CI under S1 pins | Δ Acc CI excludes 0 **or** documented tie | **Done** — Acc tie (`T151125Z`+`T151836Z`); L2 unstable → no promote ([020 §2b](./020-roadmap.md)) |
| E6 | Phase 2b: stabilize L2 under S1 pins | ≥2/3 Acc runs ctx_rel ≥0.50 or EQ ≥ LR | **Next** |

**Stop:** S1 cleared for labeled package. Phase 2 Acc honesty done (tie). Do not promote Acc headline until Phase 2b L2 stable ([016](./016-lens-eval-fairness.md)).

### Ablation ladder (selected)

| Archive | Config | Acc | ctx_rel | recall |
|---------|--------|-----|--------|--------|
| `T124903Z` | baseline BM25 / prune off | 0.765 | 0.375 | 0.928 |
| `T140420Z` | cosine keep=12 | 0.722 | 0.456 | 0.950 |
| `T142841Z` | cosine+CE+path0.4 top_k=16 | 0.696 | **0.544** | 0.911 |
| `T145324Z` | CE-only top_k=24 path0.4 | 0.710 | 0.506 | 0.909 |
| `T145634Z` | CE path-off top_k=30 | 0.709 | 0.525 | 0.936 |
| `T150417Z` | protect front-loaded (wrong) | 0.698 | 0.506 | 0.916 |
| **`T151125Z`** | **CE-order protect=12 path-off** | **0.760** | **0.519** | **0.928** |

---

## 6. Non-goals

- Do not change default `EDGEQUAKE_MIX_FUSION` away from `rrf` for headline Acc.
- Do not turn Mix arm gate back on for headline Acc.
- Do not change `related_chunk_number` in the same run as prune/CE.
- Do not treat BM25-only min-score filters that empty the context as a win (empty → `valid:false`).
- Do not claim HippoRAG2 parity without same-pin dual-SUT.
- Do not front-load Mix ranks ahead of CE order when using protect (kills Acc).
