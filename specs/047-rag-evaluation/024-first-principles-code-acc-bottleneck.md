# 024 — First-Principles Code Analysis: What Must Change to Score Higher

**Evidence run:** chart-8 `P0_mm_ite` 2026-07-15 (`smoke/` · Acc **0.4154** · Chart **0.227** · page_hit@5 **0.77** · Chart a_in_e **0.40**)  
**Law:** FP1 information only flows forward · FP2 measure bottleneck · FP7 code is law  
**Cross-ref:** [013](./013-first-principles-improvement-roadmap.md) · [015](./015-modality-aware-vision-improvement-plan.md) · [025](./025-stronger-vision-first-battle-plan.md) · **[026 next plan](./026-first-principles-score-improvement-brainstorm.md)** · canvas `spec047-fp-code-acc.canvas.tsx`

---

## 0. One-line verdict

> **Acc is capped by chart/table numbers missing from indexable markdown (W1), not by hybrid retrieval or the harness.**

| Stage | Metric | Value | Causal reading |
|-------|--------|------:|----------------|
| Representation | Chart `answer_in_evidence` | 0.40 | Gold digits never entered text |
| Retrieval | `page_hit@5` | 0.77 | Usually find the gold page |
| Generation | false refusal \| page_hit@5 | 0.10 | Secondary given dense context |
| Ops | empty / valid | 0 / true | Harness not the Acc gap |

---

## 1. End-to-end call graph (P0_mm_ite)

```text
PDF bytes
  → VisionPdfConverter::convert
       + RAG_PAGE_VISION_SYSTEM_PROMPT          [Pass A — page markdown]
       + fig/table/page PNG / chart residual crop
       + <!-- edgequake-page:N --> + <drawing/>
  → analyze_multimodal_images_with_substep (opts=ite)
       classify → specialize_image_analysis     [Pass B — chart/figure]
  → enrich_processed_text_with_mm_chunks
  → PageAwareChunking → embed + modality stamp
  → query hybrid + document_filter.document_ids
  → grounding_instructions → LLM answer
  → bench047 extract → Acc / page_hit / false_refusal
```

---

## 2. Code map by stage

### Pass A — vision page → markdown

| Piece | Path · symbol |
|-------|----------------|
| Task entry | `edgequake-api/.../processor/pdf_processing.rs` · `process_pdf_processing` |
| Converter | `edgequake-pdf/.../backend/vision.rs` · `VisionPdfConverter::convert` |
| Chart number-dump prompt | `edgequake-pdf/.../vision_prompts.rs` · `RAG_PAGE_VISION_SYSTEM_PROMPT` |
| Chart residual crops | `edgequake-pdf/.../chart_crop.rs` · `chart_residual_candidate_pages` → `write_chart_crop_assets` |
| Page assembly | `vision_markdown.rs` · page markers + optional `<drawing/>` |

**Lawful skip risk:** residual chart crops only run on pages **without** fig/table assets. Mis-bounded ImageXObjects ⇒ Pass B never sees a tight chart crop.

### Pass B — `process_options=ite`

| Piece | Path · symbol |
|-------|----------------|
| Flag parse | `vision_content.rs` · `MultimodalProcessOptions::from_option_str` |
| Orchestration | `multimodal/analyzer.rs` · `analyze_multimodal_images_with_substep` |
| Chart specialize | `multimodal/image_specialize.rs` · `specialize_image_analysis` |
| Chart prompts | `multimodal/prompts.rs` · `CHART_ANALYSIS_SYSTEM_PROMPT` / `should_specialize_as_chart` |
| Kill switch | `multimodal/gates.rs` · `vlm_process_enabled` |

**Skip / soft paths:** no `i` ⇒ no-op; `VLM_PROCESS_ENABLE=false` ⇒ degrade/strip drawings; specialize soft-fail keeps weak classify prose; no `<drawing/>` ⇒ Pass B blind on that chart.

### Chunks → embeddings

| Piece | Path · symbol |
|-------|----------------|
| Mm sidecar append | `multimodal/chunks.rs` · `append_mm_chunks_to_text` (`<!-- multimodal-chunks -->` at **doc end**) |
| Page split | `page_aware.rs` · `PageAwareChunking` |
| Metadata | `chunk_storage.rs` · `page_start` + modality |

**Bug-shaped Acc leak:** mm chunks after the last page marker inherit the **last page’s** `page_start` — hurts W0 page_hit and Gen `page=` grounding even when numbers exist.

### Retrieve + generate

| Piece | Path · symbol |
|-------|----------------|
| Document scope | `bench047/client.py` · `document_filter.document_ids` · `document_filter_resolver.rs` |
| Hybrid | `hybrid.rs` + `hybrid_merge.rs` |
| `page_start` on sources | `helpers.rs` · `build_chunk_from_result` · `source_reference_builder.rs` |
| Refusal policy | `edgequake-query/.../grounding.rs` · `grounding_instructions` |
| Bench extract | `bench047/extract.py` · `EXTRACT_PROMPT` |

---

## 3. Ranked Acc levers (code)

| Rank | Change | File · symbol | Why (this smoke) |
|:----:|--------|---------------|------------------|
| **1** | Stronger Pass A chart digit dump (+ optional digit gate on chart-like pages) | `vision_prompts.rs` · `RAG_PAGE_VISION_SYSTEM_PROMPT` · `VisionPdfConverter::convert` | Chart a_in_e 0.40 — primary forward channel |
| **2** | Always emit chart-region crop for Pass B when page is quantitative (don’t skip residual crops just because a fig asset exists) | `chart_crop.rs` · `chart_residual_candidate_pages` · `image_specialize.rs` | Specialize never sees the plot → cannot repair Pass A |
| **3** | Fail closed / retry when Chart specialize has **no digits** | `image_specialize.rs` · `description_lacks_numeric_dump` · chart prompts | Small VLM returns prose; soft-fail keeps empty index text |
| **4** | Stamp correct `page_start` on mm sidecar chunks (from drawing page / asset) | `chunks.rs` · `append_mm_chunks_to_text` · `PageAwareChunking` | Free Acc/W0 when numbers exist |
| **5** | Calibrate grounding to quote sparse chart numerics when page present | `grounding.rs` · `grounding_instructions` | FR\|hit@5 ≈ 0.10 — **after** W1 |

**Explicit non-levers now:** hybrid fusion knobs; banning “Not answerable”; chasing TeleMM Acc as same protocol; embed-only upgrades.

---

## 4. Lever honesty matrix

| Lever                                | Role in higher Acc       | First-principles read                  |
| --------------------------------------| --------------------------| ----------------------------------------|
| Stronger **vision** model (Pass A/B) | High                     | Matches W1 evidence                    |
| Stronger **query** LLM               | Medium-low               | After digits exist in context          |
| Better harness                       | Ops only                 | Already `valid` / empty=0 / ite on     |
| Better refusal **prompt** alone      | Low / harmful if bans NA | Violates FP1 / Unans Acc               |
| Wrong test vs LVLM board             | Framing                  | Right product test; wrong SOTA ranking |

---

## 5. Success gates before claiming Acc lift

1. Chart `answer_in_evidence` ≥ **0.50** on held-out fidelity (`bench047 fidelity`)  
2. Re-score **same** retrieve profile (`P0_mm_ite` + `--document-scope`) — Chart Acc ↑  
3. Keep `page_hit@5` ≥ ~0.75 and Unans Acc ≥ ~0.70 (honesty)  
4. Label any vision-model change in scorecard pins  

---

## 6. Immediate engineering tickets (suggested order)

0. **EQ-047-W1-vision (first):** `P0_mm_ite_vision_medium` → `mistral-medium-3-5` Pass A/B only — see [025](./025-stronger-vision-first-battle-plan.md)  
1. **EQ-047-W1-crop:** expand residual/chart crop eligibility beyond “no fig/table”  
2. **EQ-047-W1-dense:** numeric fail-closed retry on Chart specialize  
3. **EQ-047-W2-mm-page:** correct `page_start` for multimodal sidecar chunks  
4. **EQ-047-W3-ground:** grounded quote when sparse numeric context + page_hit  

Do **not** open a “beat TeleMM Acc” ticket.

---

## 7. Assessed against last chart-8 `ite` run (2026-07-15)

**Sources:** `e2e/artifacts/smoke/{scorecard,predictions,fidelity}.json` · workspace `59cf569f-…`

### 7.1 Headline facts from the run

| Fact | Value | Implication for the plan |
|------|------:|--------------------------|
| Acc / F1 / valid | 0.4154 / 0.2464 / true | Ops OK — Acc lift ≠ harness rewrite |
| Chart Acc · Table Acc | 0.227 · **0.193** | Tables as bad as charts — plan under-weights W1c |
| Chart a_in_e (fidelity n=15) | **0.40** | Rank-1 W1 still primary |
| page_hit@5 (answerable) | **0.773** | Rank hybrid-fusion earlier? No |
| Chart rows with hit@5 | **19/22** | Chart Acc ≠ retrieval failure |
| mean score \| Chart+hit | **0.211** | Find page, still wrong/NA |
| answerable zeros | 56 | Splits below |
| zero + hit + **wrong answer** | **35** | Dominant post-retrieve failure |
| zero + hit + NA (false refusal) | **6** | Rank-5 FR-only is **overstated** |
| zero + page miss | 15 | Real but smaller than wrong-with-hit |
| fidelity rep_miss | **13/25** | Confirms W1 |
| fidelity retrieval_miss \| rep_ok | **2/25** | Confirms W2 secondary |
| page_hit@1 vs @5 | 0.37 vs 0.77 | Ranking dilution exists; Acc impact secondary |

### 7.2 Ticket-by-ticket verdict

| Plan rank | Ticket | Verdict vs data | Notes |
|:---------:|--------|-----------------|-------|
| 1 | Pass A denser chart dumps / stronger vision | **Confirmed — keep #1** | Chart a_in_e 0.40; 13/25 rep miss; Chart hit@5 86% yet Acc 0.23 |
| 2 | Always crop charts for Pass B (don’t skip residual when fig exists) | **Plausible, not proven by this scorecard** | No crop-coverage telemetry in artifacts. Code hole is real; add metric before claiming Acc causation |
| 3 | Fail closed on Chart specialize with **no digits** | **Partially confirmed** | Helps empty/prose extracts + some NA. **Does not** fix wrong digits (7→16, 12%→17% with hit) — need denser *correct* dumps, not only “has a digit” |
| 4 | Fix mm sidecar `page_start` | **Keep, demote for Acc** | Helps W0 honesty + Gen `page=`. Only 2/25 are retrieve-miss-given-rep-ok — not the Acc bulk |
| 5 | Grounding / cut FR\|hit | **Overweight — revise** | Only **6/56** answerable zeros are NA-with-hit. **35/56** are wrong answers with hit. Broader Gen issue: pick the right numeric from clutter once W1 improves |

### 7.3 Gaps the plan under-states (from this run)

1. **Table Acc &lt; Chart Acc** (0.19 vs 0.23) with Table a_in_e 0.45 — elevate wide-table / Pass B `t` quality beside chart crops.  
2. **Wrong-with-hit ≫ false-refusal-with-hit** — after digits exist, Acc needs calibrated extraction from dense context, not “never say NA.”  
3. **Cross-page Acc 0.14** — little coverage in ranked tickets; still a slice gap.  
4. **Pure-text Acc 0.25** with a_in_e 0.53 — not purely a multimodal problem.

### 7.4 Revised engineering order (data-adjusted)

1. **W1 Pass A/B digit fidelity** (Chart **and Table**) + stronger vision — gate Chart/Table a_in_e ≥ 0.50  
2. **Instrument crop coverage** then ship residual-crop expansion (old rank 2) only if coverage is low on Chart-miss pages  
3. **Specialize numeric quality** (correct key_values, not merely non-empty digits)  
4. **Wrong-with-hit Gen** (quote matching numeric; still allow honest NA) — replaces FR-only W3 as Acc priority  
5. **mm `page_start` fix** — correctness / diagnostics, Acc secondary  
6. Cross-page / hybrid dilution — only after W1 moves page_hit@1 and Acc

### 7.5 Bottom line

The plan’s **main causal story is right** (W1 representation, not LVLM Acc chase, not ban-NA prompts).  
Adjustments from real data: **promote tables**, **prove crop hypothesis with telemetry**, **demote FR-only W3**, treat **wrong-with-hit** as the large post-retrieve failure mode.
