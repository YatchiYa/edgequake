# 018 — Quality & Speed Improvement Plan (Query + Ingest)

**Status:** actionable · 2026-07-11  
**Goal:** Make EdgeQuake the best **quality** and **speed** GraphRAG for SPEC-047 (and production), using LightRAG as a code-law peer, not a marketing target.  
**Assessment:** [017](./017-lightrag-vs-edgequake-query-pipeline-assessment.md)  
**Depends on:** [013](./013-first-principles-improvement-roadmap.md) · [014](./014-ingest-query-pipeline-first-principles.md) · [015](./015-modality-aware-vision-improvement-plan.md) · [016](./016-ingest-speed-reliability-battle-plan.md)  
**Law:** one causal change per experiment · fail closed · honesty > Acc inflation

---

## 0. Strategy in one paragraph

Keep EdgeQuake’s hybrid advantages (BM25 + Mix RRF + scope + modality). Close the **representation → grounding** chain so `page_hit@5` converts to Acc (especially Chart). Parallelize irreversible ingest work without redoing vision. Align bench mode with the best fusion (`mix`+RRF or naive-first hybrid) after measuring arm hit-rates. Do **not** chase LightRAG naming or ban “Not answerable.”

```text
QUALITY path:  vision/MM fidelity → atomic chunks → page headers in prompt → fusion that keeps page hits
SPEED path:    soft-resume + unique embed + merge gates (done) → vision admission → extract concurrency → skip dead work
```

---

## 1. Scoreboard targets (gates)

Baseline: smoke `P0_mm_ite` · Acc≈0.38 · `page_hit@5`≈0.76 · Chart Acc≈0.14 · Unanswerable Acc≈0.69

| Gate ID | Metric | Target (smoke) | Do not regress |
|---------|--------|----------------|----------------|
| G-Q1 | Chart Acc | ≥ 0.30 | Unanswerable Acc ≥ 0.65 |
| G-Q2 | `page_hit@5` (scoped) | ≥ 0.85 | context_empty_rate = 0 |
| G-Q3 | Overall Acc | ≥ 0.50 | valid=true |
| G-Q4 | answer_in_evidence (Chart) | ↑ vs baseline | — |
| G-S1 | Soft-resume extract wall-clock | ≤ baseline − 30% | quality gates |
| G-S2 | Fresh PDF ingest (10 docs) | ≤ baseline − 20% | ingest coverage 1.0 |
| G-S3 | Query p95 (hybrid/mix, scoped) | ≤ baseline + 10% after quality wins | — |

Every ticket below must name **symbols** and a **gate**.

---

## 2. Phase A — Convert retrieval into answers (query quality, no re-ingest)

**Why first:** `page_hit@5≈0.76` already finds pages; Acc lags → prompt/fusion, not more arms.

| # | Ticket | Change | Code anchors | Gate | Effort |
|---|--------|--------|--------------|------|--------|
| A1 | **Inline page headers** | Chunk prompt lines include `page_start` / modality | `context.rs::to_context_string` | Acc ↑ when page_hit; unanswerable held | S |
| A2 | **Bench mode ablation** | Profile `P1_mix_rrf` vs locked `hybrid` round-robin | `profiles.py`, `mix.rs`, `hybrid_merge.rs` | `page_hit@5` + Acc delta | S |
| A3 | **Naive-first hybrid** | For chart/factual slices: `EDGEQUAKE_HYBRID_FUSION=rrf` or naive-weighted | `hybrid_merge.rs`, env | Chart Acc / page_hit | S |
| A4 | **Chunk budget floor** | Cap graph tokens so chunk_budget ≥ N (e.g. 40% of total) | `truncation.rs::balance_context` | mean chunk tokens ↑ | M |
| A5 | **Arm-gate telemetry review** | Publish misclassified intents on smoke misses | `mix_weights.rs`, `diagnostics.py` | false naive-only rate | S |

**Reject:** ban “Not answerable”; feed gold `evidence_pages` into retrieve.

**Exit:** A1 + A2 land · scorecard shows Acc lift **or** proves fusion not the bottleneck (then Phase B).

---

## 3. Phase B — Representation (ingest quality → query)

**Why:** Chart Acc≈0.14 is representation physics (014/015).

| # | Ticket | Change | Code anchors | Gate | Effort |
|---|--------|--------|--------------|------|--------|
| B1 | **Always-on MM for bench** | `process_options=ite` default in bench047 profiles | `client.py`, `profiles.py` | Chart answer_in_evidence ↑ | S |
| B2 | **Typed chart/table prompts** | Execute [015](./015-modality-aware-vision-improvement-plan.md) P0/P1 | `multimodal/prompts.rs`, `image_specialize.rs` | Chart Acc ≥ 0.30 | L |
| B3 | **Keep empty vision pages** | Placeholder markdown for figure-only pages | `edgequake-pdf/.../vision.rs` | page coverage ↑ | M |
| B4 | **Atomic + caption glue** | Never split chart block from numeric table; optional caption prepend | `atomic_blocks.rs`, `page_aware.rs` | Chart fidelity ↑ | M |
| B5 | **Page number in embed text** | Optional `Page N:` prefix before embed (not only metadata) | `chunk_storage.rs` / page_aware | page_hit@1 ↑ | M |
| B6 | **Modality stamp audit** | Contract: every MM sidecar → `modality` on vector metadata | `retrieval_modality.rs`, storage contracts | filter hit-rate | S |

**Exit:** B2 + B1 on smoke · Chart Acc gate G-Q1.

---

## 4. Phase C — Query parity & LightRAG fairness

| # | Ticket | Change | Code anchors | Gate | Effort |
|---|--------|--------|--------------|------|--------|
| C1 | **Mode map docs in API** | OpenAPI / query_types note hybrid≠LR hybrid | `handlers/query_types.rs`, docs | reviewer checklist | S |
| C2 | **`lightrag_hybrid` profile** | Eval profile: local∥global only (no naive) for LR parity runs | `profiles.py`, optional arm override | comparable Acc run | M |
| C3 | **Stronger rerank binding** | Wire Cohere/Jina-class when key present (LR pattern) | API rerank provider | nDCG / Acc on slice | M |
| C4 | **Keyword cache + fingerprint** | Match LR cache key discipline (mode+tokens+rerank) | keyword extractor cache | p95 latency ↓ | S |
| C5 | **Content headings optional** | LR `enable_content_headings` parity for MD | chunk metadata / prompt | Acc on layout slice | M |

---

## 5. Phase D — Ingest speed (keep quality)

Build on [016](./016-ingest-speed-reliability-battle-plan.md) (P0–P7f mostly done).

| # | Ticket | Change | Code anchors | Gate | Effort |
|---|--------|--------|--------------|------|--------|
| D1 | **Markdown soft-resume SSOT** | Never re-vision when L1 markdown+assets present | `pdf_processing.rs`, resume helpers | G-S1 | M |
| D2 | **Admission dashboard** | Publish `VISION_JOBS × CONCURRENCY × MM` product in ops | Makefile / health | no pool thrash | S |
| D3 | **Chunk-only / retrieve-only bench path** | Profile skips extract+merge for retrieve ablations | `PipelineConfig` ingest profiles | G-S2 for retrieve-only | S |
| D4 | **Community gate on ingest** | Never refresh communities above N without ResourceGuard | community refresh path | ingest p95 | M |
| D5 | **Extract concurrency profile** | Tie `max_concurrent_extractions` to provider RPS | `pipeline/config.rs` | wall-clock vs error rate | S |
| D6 | **Drawing-tag resume check** | Soft-resume that lacks `<drawing>` must not claim MM complete | resume + MM specialize | no silent MM skip | M |

**Exit:** G-S1/G-S2 on smoke corpus without Acc drop.

---

## 6. Phase E — Optional science (only if G-Q3 stalls)

| # | Ticket | When | Anchor |
|---|--------|------|--------|
| E1 | PPR / bipartite chunk pick default-on for Exploratory | multi-hop Acc flat | `graph_walk`, `chunk_retrieval.rs` |
| E2 | True community reports (Leiden-lite) | Exploratory Acc flat | community pipeline |
| E3 | Late-interaction / page embeddings | caption-and-index plateaus | research spike; out of v1 |

Do not start E while A/B gates unmet.

---

## 7. Experiment order (copy-paste)

```text
1. A1  page headers in to_context_string     # query-only, re-score smoke
2. A2  P1_mix_rrf vs hybrid ablation         # query-only
3. B1  confirm process_options=ite           # already in P0_mm_ite; verify artifacts
4. B2  typed chart prompts (015)             # re-ingest sample charts
5. D1  soft-resume wall-clock measure        # speed
6. A3/A4 fusion + truncation if still needed
7. C2  optional LightRAG-fair hybrid profile # publish side-by-side Acc
```

Each step: locked `profiles.py` · one scorecard · `page_hit@5` + Chart Acc + Unanswerable Acc.

---

## 8. Ownership map (crates)

| Area | Crate / path |
|------|----------------|
| Query fusion / arms | `edgequake-query` — `mix.rs`, `hybrid_merge.rs`, `mix_weights.rs`, `fusion.rs` |
| Prompt context | `edgequake-query` — `context.rs`, `truncation.rs`, `prompt.rs` |
| Sparse / modality | `edgequake-query` — `sparse_retrieval.rs`, `modality_retrieve.rs` |
| API request shape | `edgequake-api` — `query_types.rs`, `query_execute.rs` |
| Vision / MM | `edgequake-pdf`, `edgequake-api/services/multimodal/` |
| Chunk / atomic | `edgequake-pipeline/chunker/` |
| Persist / merge | `edgequake-pipeline` merger + `edgequake-storage` |
| Harness | `tools/bench047/` |

---

## 9. Anti-patterns (flaky heuristics)

| Do not | Why |
|--------|-----|
| Claim EQ hybrid ≡ LightRAG hybrid | Different arms (017 §3) |
| Turn off BM25 to “match LightRAG” | Throws away EQ’s 2026 advantage |
| Re-vision all PDFs to fix merge | Violates speed P2 (016) |
| Prompt-only “always answer” | Inflates Acc; kills unanswerable |
| Mid-run provider swap | Invalidates scorecard |
| Start PPR before page headers + Chart MM | Wrong bottleneck |

---

## 10. Definition of done (SPEC-047 v1 quality/speed)

- [ ] A1 merged; smoke Acc ↑ or documented null result  
- [ ] A2 published: Mix RRF vs hybrid round-robin table in artifacts  
- [ ] B2 Chart Acc ≥ 0.30 on smoke (or written waiver with fidelity proof)  
- [ ] D1 soft-resume ≥ 30% faster extract path without coverage loss  
- [ ] 017 naming trap documented in API OpenAPI  
- [ ] Unanswerable Acc ≥ 0.65 throughout  

When all checked: update [000-index](./000-index.md) one-screen verdict and bump smoke SUMMARY.

---

## 11. Relationship to prior docs

| Doc | Role vs 018 |
|-----|-------------|
| 013 | Broad workstreams W0–Wn; 018 is the **ordered execution** after LightRAG deep compare |
| 014 | Causal physics; 018 turns ranked lifts into tickets A/B |
| 015 | Modality vision detail for Phase B |
| 016 | Ingest speed reliability; Phase D continues it |
| 017 | Evidence and scorecard; this plan is the response |
