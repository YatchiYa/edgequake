# 011 — Lens: Evidence Coverage / Recall (P1)

**Priority:** P1 — after noise control (**S1 package keeps recall flat**)  
**Cross-ref:** [010 Retrieval noise](./010-lens-retrieval-noise.md) · [005 Pins](../005-mode-map-and-pins.md) · [020 §1b](./020-roadmap.md)

---

## 1. Observation

| Metric | EQ baseline | EQ S1 (`T151125Z`) | LR (baseline) |
|--------|-------------|--------------------|---------------|
| evidence_recall | **0.928** | **0.928** | 0.991 |
| context_relevancy | 0.375 | **0.519** | 0.544 |

Recall was already strong (≥0.93). Soft CE+cosine dipped to 0.911; the S1 CE+protect package **holds recall flat** vs baseline while clearing ctx_rel ≥ 0.50. Remaining P1 work is closing the **−6pp vs LR** gap without reopening Acc/relevancy budgets.

---

## 2. First-principles diagnosis

- **Law:** Completeness of C w.r.t. gold evidence spans — orthogonal to “how clean” C is.
- **Risk of P0:** Over-pruning can trade relevancy↑ for recall↓. Success criteria must couple both (010 E1: recall drop ≤ 0.03).
- **Likely EQ miss sources:** (1) KG→chunk ID take under `related_chunk_number`; (2) naive vector arm under-contribution on some questions; (3) keyword dual-level extraction quality vs LightRAG; (4) min_score filters discarding borderline gold chunks.

---

## 3. July 2026 practice

- Measure **Evidence Recall** as a first-class retrieval metric (GraphRAG-Bench F.2 / L2) — do not infer from Acc.
- Hybrid dense + BM25 improves exact-match spans (names, codes, dosages in medical text).
- Multi-query / HyDE help ambiguous questions — latency cost; use as labeled ablation.
- Keep candidate pool wide (top-30) then prune; do not shrink retrieve_topk for headline Acc (pin = 30).

---

## 4. EQ insertion points

| Area | File | Notes |
|------|------|-------|
| KG chunk ID pick | `edgequake-query/src/kg_chunk_pick.rs` | `collect_kg_chunk_ids_scoped`; Acc pin `related_chunk_number=5` |
| Chunk append / fuse | `edgequake-query/src/chunk_retrieval.rs` | `append_score_ranked_chunks` — vector/BM25/PPR modes |
| Local / Global arms | `.../modes/local.rs`, `global.rs` | Entity/rel pools → chunk append |
| Naive arm | Mix path in `modes/mix.rs` | Ensure naive always contributes under LR-like arms |
| Keyword extraction | Mix / keyword path (LLM-assisted) | Audit failures vs LR dual-level keywords |

---

## 5. Experiments (one confound each)

| # | Change | Success |
|---|--------|---------|
| R1 | Audit keyword extraction failure rate on smoke (no pin change) | Report miss taxonomy; fix bugs only |
| R2 | Ensure naive arm non-empty contribution when arms-on | Per-question recall↑ on Fact subset |
| R3 | Labeled ablation: `RELATED_CHUNK_NUMBER` 5 → 8 | recall↑ ≥ 0.02 **and** ctx_rel not ↓ ≥ 0.05 — else reject. **Rejected** Acc-win E3 `T154427Z`: Summarize recall **flat 0.863** — see [017](./017-beat-lightrag.md) |
| R4 | Soften vector `min_score` only if diagnostics show gold drops | recall↑; empty rate stays 0 |
| R5 | After P0 prune wins: re-check recall under prune | Must stay ≥ 0.90 on smoke |

**Order:** Prefer R1/R2 (bug/parity) before R3 (pin ablation). Never combine R3 with a new prune in the same run.

---

## 6. Non-goals

- Do not raise `related_chunk_number` in the Acc **headline** without labeling an ablation profile.
- Do not lower `retrieve_topk` below 30 for publish Acc.
- Do not optimize recall by dumping more entities into the prompt (hurts P0).
- Do not claim “closed the recall gap” from Acc F1 alone.
