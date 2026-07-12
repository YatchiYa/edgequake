# 015 — Modality-Aware Vision Improvement Plan

**Topic:** Analyze PDF images/figures/charts/tables with **typed prompts** and structured extract  
**Cross-ref:** [013](./013-first-principles-improvement-roadmap.md) · [014](./014-ingest-query-pipeline-first-principles.md) · [022](./022-reassessment-2026-07-11.md) · smoke [`FIDELITY`](./e2e/artifacts/smoke/FIDELITY.md) · [`SUMMARY`](./e2e/artifacts/smoke/SUMMARY.md)  
**Status:** Phase A–C + **MV-18/19 ✅ measured** · Chart a_in_e **0.409** (was ~0.32) · Chart Acc **0.182** · Acc 0.423 · **G-A still open** (≥0.50)  
**Owner lens:** First principles · code is law · no Acc heuristics

---

## 0. Problem statement (why this plan)

Smoke physics (**update 2026-07-11 — locked chart fixture**):

| Metric | Early W0 | Locked HEAD (lineage) | Meaning |
|--------|----------|----------------------|---------|
| Chart Acc | ~0.05 | **~0.14** | Still floor — Rep not solved |
| Overall Acc | ~0.41 | **~0.43** | Query lane moved Acc; Chart did not |
| page_hit@5 | ~0.59 | **~0.76** | Retrieve OK under document-scope |
| Unanswerable Acc | ~0.78 | **~0.83** | Keep; do not ban NA |
| Chart answer-in-evidence | ~0.36 | still weak | Gold numbers often absent from markdown |

**022 verdict:** Further Acc without Chart representation work is unlikely. Query fusion/refusal is the wrong knob.

**Root cause (code):**

1. **Page vision** uses one generic page→markdown pass (`VisionPdfConverter` / pdf2md).  
2. **Multimodal second pass** has typed prompts for image / table / equation (`multimodal/prompts.rs`) but:
   - P0 upload sends **no** `process_options` → analyze is a **no-op** (`analyzer.rs` `!opts.any_enabled()`).
   - **Chart is not a first-class modality** — only a `type` label inside the generic image prompt.
3. No classify→specialize loop for Chart vs Figure vs Infographic.

**One-line goal:** Make chart/table/figure numbers **land in indexable text** with modality-correct prompts, then prove it with fidelity before chasing Acc.

---

## 1. First principles (non-negotiable)

| ID | Principle | Implication for this plan |
|----|-----------|---------------------------|
| FP1 | Information only flows forward | Typed extract at ingest; prompts at query cannot invent chart values |
| FP2 | Measure the bottleneck | Gate on `answer_in_evidence` (esp. Chart) + `page_hit@5`, not Acc alone |
| FP3 | One causal change per experiment | Enable `ite` ≠ add chart prompt ≠ change fusion — separate tickets |
| FP4 | Honesty > Acc inflation | Never ban “Not answerable”; protect unanswerable Acc ≥ ~0.85 |
| FP7 | Code is law | Extend existing `prompts.rs` / `MultimodalProcessOptions` / bench047 — don’t fork a parallel pipeline |

### Reject

- Generic “describe the image better” without schema  
- Gold `evidence_pages` in retrieve  
- Prompt-only Acc patches  
- Enabling multimodal without a fidelity gate  

---

## 2. Current law (as-built)

```text
PDF page image
  → VisionPdfConverter (generic page markdown)     # ALWAYS on P0
  → multimodal analyze (i/t/e)                     # OFF on P0
       image  → IMAGE_ANALYSIS_SYSTEM_PROMPT       # classifies Chart|… in JSON
       table  → TABLE_ANALYSIS_SYSTEM_PROMPT
       equation → EQUATION_ANALYSIS_SYSTEM_PROMPT
  → PageAwareChunking → embed → graph
```

**Anchors:**

| Piece | Path |
|-------|------|
| Process flags | `vision_content.rs::MultimodalProcessOptions` (`i`/`t`/`e`) |
| Analyze gate | `multimodal/analyzer.rs::analyze_multimodal_images` |
| Prompts | `multimodal/prompts.rs` |
| Chunk render | `multimodal/chunks.rs` (`[Image Name]…`, `[Table Name]…`) |
| Upload field | `pdf_upload` form `process_options` |
| Bench gap | `bench047/client.py::upload_pdf` — no `process_options` today |
| Fidelity gate | `bench047 fidelity` / `fidelity.py` |

---

## 3. Target architecture

```text
                    ┌─────────────────────────────┐
  PDF page ────────►│ Pass A: page→markdown (keep) │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │ Pass B: detect candidates     │
                    │  inline images, HTML tables,  │
                    │  equation blocks, chart-like  │
                    └─────────────┬───────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
     ┌────────────────┐  ┌────────────────┐  ┌────────────────┐
     │ Classify (cheap)│  │ Table prompt   │  │ Equation prompt│
     │ Photo|Chart|…  │  │ (existing)     │  │ (existing)     │
     └───────┬────────┘  └────────────────┘  └────────────────┘
             │
     ┌───────┴────────┬──────────────┐
     ▼                ▼              ▼
 ┌─────────┐   ┌───────────┐   ┌──────────┐
 │ Chart   │   │ Figure/   │   │ Generic  │
 │ extract │   │ Diagram   │   │ image    │
 │ schema  │   │ extract   │   │ (today)  │
 └────┬────┘   └─────┬─────┘   └────┬─────┘
      │              │              │
      └──────────────┴──────────────┘
                     │
                     ▼
        Structured markdown + mm sidecar chunks
        (page_start preserved) → embed / graph
```

**Design choice (aligned with 2025–26 multimodal RAG practice):**

- **Ingest:** dense, searchable structured text (numbers, axes, labels).  
- **Optional later:** retrieval-time VLM re-read of stored page/crop images (Phase D) — do **not** block Phase A–C on this.

---

## 4. Phased delivery

### Phase A — Turn on the existing second pass (1–3 days)

**Goal:** Prove multimodal analyze moves Chart/Table fidelity without new prompts.

| Ticket | Work | Done when |
|--------|------|-----------|
| **EQ-047-MV-01** ✅ | `client.upload_pdf(..., process_options=...)` + profile field | Upload form includes `process_options` |
| **EQ-047-MV-02** ✅ | Profile `P0_mm_ite` — label cost | Scorecard pin shows `process_options=ite` |
| **EQ-047-MV-03** ✅ | Doctor checks `VLM_PROCESS_ENABLE` when profile requires it | Doctor FAIL without env |
| **EQ-047-MV-04** ✅ measured | Re-ingest chart subset (`smoke_chart_doc_ids_v1`, 8 docs) | Chart fidelity **0.32** (n=22) — **G-A FAIL** (gate ≥0.50) |
| **EQ-047-MV-05** ✅ measured | Query re-score same fixture | Acc 0.41 · F1 0.26 · Chart Acc 0.18 · unanswerable 0.64 · `page_hit@5` 0.80 |

**Gate G-A result (2026-07-10):** **FAIL.** Chart answer_in_evidence **0.318** vs P0 baseline **0.364** (n=22 each). Overall evidence rate 0.48 vs 0.51. Ingested markdown on chart docs has **zero** `<drawing>` / `data:image` / `[Chart Name]` / `multimodal-chunks` — `process_options=ite` ran on tasks but analyze had **no inline candidates** (vision page pass only). **Next law:** Phase C MV-21 (emit scannable image refs) before re-testing G-A/G-B.

**Cost note:** `ite` multiplies VLM calls. Prefer doc subset + resume; never silent full-corpus re-ingest.

---

### Phase B — Modality-specialized prompts (3–7 days)

**Goal:** Chart / figure / table get **different** extract schemas (first principles: different visual physics).

| Ticket | Work | Code anchors |
|--------|------|--------------|
| **EQ-047-MV-10** ✅ | Add `CHART_ANALYSIS_SYSTEM_PROMPT` + `chart_analysis_messages` | `multimodal/prompts.rs` |
| **EQ-047-MV-11** ✅ | Chart JSON schema: name, chart_kind, title, axes, series, key_values, description — never invent | `image_specialize.rs` |
| **EQ-047-MV-12** ✅ | `FIGURE_ANALYSIS_SYSTEM_PROMPT` — components, labels, relationships | `prompts.rs` |
| **EQ-047-MV-13** ✅ | Two-step: classify → Chart/Figure specialize (fail-open) | `analyzer.rs` + `image_specialize.rs` |
| **EQ-047-MV-14** ✅ | Strengthen table prompt: markdown table of all visible cells + units | `TABLE_ANALYSIS_SYSTEM_PROMPT` |
| **EQ-047-MV-15** ✅ | Render `[Chart Name]` / `[Figure Name]` + parse_mm_display_name | `chunks.rs` · `injection.rs` |
| **EQ-047-MV-16** ✅ | Unit/contract tests: route helpers, specialize merge, chunk labels | prompts / specialize / analyzer / contracts |
| **EQ-047-MV-17** | Extend `bench047 fidelity` optional: number-token recall on Chart rows | `fidelity.py` |
| **EQ-047-MV-18** ✅ | Pass A RAG page prompt: chart/figure **number dump** via pdf2md `system_prompt` | `edgequake-pdf/vision_prompts.rs` · wired in `backend/vision.rs` |
| **EQ-047-MV-19** ✅ | Context/caption chart route + `data_table_md` / `visible_text` denser index text | `should_specialize_as_chart` · `image_specialize.rs` · `e2e_spec047_chart_number_landing.rs` |

**Chart prompt contract (normative):**

```text
Extract ONLY what is visually readable.
Required JSON keys:
  name, chart_kind, title, x_axis, y_axis,
  series: [{ name, values: [{x, y_raw}] }],
  key_values: [{ label, value_raw }],
  data_table_md (optional GFM table of same points),
  description (≤300 words, no new numbers)
If a value is unreadable, omit it — do not estimate.
```

**Pass A contract (normative, MV-18):** page vision must emit a markdown table of every
readable chart data point; never invent. Specialize (Pass B) is a second chance, not a
substitute for Pass A on chart-heavy pages.

**Acc / G-B gate (2026-07-11 measured):** Full 8-doc re-ingest `smoke-post-mv18-full-chart`:
Chart answer_in_evidence **0.409** (n=22) vs prior ~0.32 · Chart Acc **0.182** · overall Acc **0.423**.
**G-A (≥0.50) FAIL** but direction clear. G-B (≥0.60) open. Next: denser extract, not fusion.

**Gate G-B:** Chart answer_in_evidence ≥ **0.60** on same frozen chart subset; Chart Acc moves in the same direction; unanswerable Acc ≥ ~0.85 on full smoke if re-scored.

---

### Phase C — Detection & chunk integrity (3–5 days)

**Goal:** Candidates exist and stay together for retrieval.

| Ticket | Work | Why |
|--------|------|-----|
| **EQ-047-MV-20** ✅ | Stop dropping empty vision pages — emit placeholder + keep page marker | `vision_markdown.rs` + `vision.rs` |
| **EQ-047-MV-21** ✅ | Ensure vision markdown emits inline image refs scannable by `scan_inline_image_refs` | `drawing_tags.rs` · `page_assets.rs` · `document_assets.rs` · `e2e_spec047_vision_drawing_pipeline.rs` |
| **EQ-047-MV-22** ✅ | Page-aware: do not split fenced tables / `[Chart Name]` / VLM `# title` + `**Type:**` blocks mid-chunk | `atomic_blocks.rs` · `recursive.rs` · `contract_spec047_atomic_chunking.rs` · `e2e_spec047_atomic_chunking.rs` |
| **EQ-047-MV-23** ✅ | Vector metadata: `modality=chart|figure|table|equation` for filtered retrieve later | `retrieval_modality.rs` · `chunk_storage.rs` · `extraction.rs` · `contract_spec047_modality_metadata.rs` · `e2e_spec047_modality_persist.rs` |
| **EQ-047-MV-24** ✅ | Page-crop re-render for chart regions (Pass A–gated hi-res ink crop → drawing override + specialize fallback) | `chart_crop.rs` · `vision.rs` · `vision_markdown.rs` · `image_specialize.rs` · `e2e_spec047_chart_crop.rs` |
| **EQ-047-MV-25** | (Related) Chart-crop path uses budgeted 200dpi / 3600px; full-page still default | `chart_crop.rs` `CHART_CROP_RENDER` |
| **EQ-047-MV-26** ✅ | Routing hygiene: Pass A body snippet in drawing caption → specialize `PromptContext` | `caption_with_page_context` · `format_drawing_block` · `prompts::context_suggests_chart` |
| **EQ-047-MV-27** ✅ | Specialize soft-fail: keep Pass A numeric dump (tables / key values) instead of weak classify | `pass_a_numeric_dump_from_context` · `soft_fail_chart_result` |
| **EQ-047-MV-28** ✅ | Page-local dump + viewer images: `![alt](assets/…)` + `<drawing/>`; mm-assets GET; analyze keeps image on-page | `drawing_tags.rs` · `vision_markdown.rs` · `mm_assets.rs` · `AuthenticatedMarkdownImage` |

**Gate G-C:** `n_pages_in_markdown` covers gold evidence pages for chart Qs; mm chunk count > 0 on chart docs.

---

### Phase D — Retrieve & generate assist (after A–C) (2–5 days)

Only after representation lifts. Do **not** start here.

| Ticket | Work | Gate |
|--------|------|------|
| **EQ-047-MV-30** | Ablation `--document-scope` on mm-enriched workspace | `page_hit@5` ↑ |
| **EQ-047-MV-31** | Inline `page_start` in `to_context_string` (W0c) | Grounded Acc ↑ when hit |
| **EQ-047-MV-32** ✅ | Prefer chunks with `modality=chart` when query intent is numeric/chart | `modality_retrieve.rs` · `MetadataFilter.modalities` · memory+postgres storage · `support/metadata_filter_modality_contract.rs` · `contract_spec047_modality_storage.rs` |
| **EQ-047-MV-33** | (Optional) Retrieval-time VLM: if top chunk is chart crop, re-ask chart prompt with user question | Measure separately; cost-gated |

---

### Phase E — Eval discipline (ongoing)

| Ticket | Work |
|--------|------|
| **EQ-047-MV-40** | Scorecard pins: `process_options`, `mm_prompt_sha`, chart fidelity rate |
| **EQ-047-MV-41** | Progression table in [012](./012-acceptance-criteria-and-scorecard.md) after each valid run |
| **EQ-047-MV-42** | Cost envelope: max VLM calls / doc; fail closed if exceeded |
| **EQ-047-MV-43** | Profiles: `P0_primary` (baseline), `P0_mm_ite`, `P0_mm_chart` (typed prompts) |

---

## 5. Experiment matrix (locked)

| Run ID | Change | Docs | Measure |
|--------|--------|------|---------|
| R0 | Current P0 (no `ite`) | smoke | Baseline fidelity + Acc |
| R1 | `process_options=ite`, existing prompts | chart subset → smoke | G-A |
| R2 | R1 + chart/figure typed prompts | same | G-B |
| R3 | R2 + `--document-scope` | query-only | page_hit@5 |
| R4 | R2 + page headers in prompt | query-only | Acc / false refusal given hit |

One change per run. Same fixture. Same models (`mistral-small-latest` + `mistral-embed`).

---

## 6. Success scorecard

| Gate | Metric | Pass |
|------|--------|------|
| G0 | Harness `valid=true` | Always |
| G-A | Chart answer_in_evidence | ≥ 0.50 or clear +Δ vs 0.36 |
| G-B | Chart answer_in_evidence | ≥ 0.60 |
| G-B2 | Chart Acc | ↑ vs 0.05 with n reported |
| G-C | Unanswerable Acc | ≥ ~0.85 on full smoke |
| G-D | page_hit@5 (answerable) | ↑ vs 0.59 after scope/fusion |
| G-E | Overall F1 | ↑ only with G-A/B/C held |

---

## 7. Implementation sketch (SRP / DRY)

```text
edgequake-pdf/vision_prompts.rs     # SSOT Pass A RAG page prompt (MV-18)
  → backend/vision.rs system_prompt override

multimodal/prompts.rs               # SSOT Pass B strings + route predicates
  chart_analysis_messages() / figure_analysis_messages()
  should_specialize_as_chart(type, ctx)   # type OR caption/leading hints
  keep image_analysis_messages() as classify + fallback

multimodal/image_specialize.rs      # parse → searchable markdown (SRP)
  chart_analysis_to_description (key_values + series + data_table_md)
  figure_analysis_to_description (components + visible_text)

multimodal/analyzer.rs              # orchestration only
multimodal/chunks.rs                # [Chart Name] | [Figure Name] | …

tools/bench047                      # fidelity Chart slice = Acc gate SSOT
```

**SOLID:** prompts own strings + predicates; specialize owns merge; analyzer owns orchestration;
chunks own render; pdf vision_prompts own Pass A; bench owns measurement.

---

## 8. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| VLM cost explosion | Subset re-ingest; cache (`analysis_cache_key`); concurrency caps |
| Two-step latency | Classify with small/fast call; extract only Chart/Table |
| Hallucinated chart numbers | Prompt: omit unreadables; fidelity checks exact gold containment |
| Inline images missing from vision markdown | MV-21/24; page-level crop fallback |
| Acc noise on re-query | Prefer fidelity + page_hit as primary gates |

---

## 9. Suggested calendar

| Week | Focus |
|------|-------|
| W1 | Phase A (enable `ite` + fidelity on chart subset) |
| W2 | Phase B (chart/figure prompts + routing) |
| W3 | Phase C (page retention + chunk integrity) |
| W4 | Phase D ablations (scope, page-in-prompt) + update 012 progression |

---

## 10. Immediate next action (post MV-18/19)

Code + e2e for number-landing are green. **Live Acc gate still open:**

```bash
# Rebuild API with Pass A override, then re-ingest chart fixture (not query-only):
export EDGEQUAKE_API_URL=http://localhost:8090
export EDGEQUAKE_BENCH_FIXTURE=smoke_chart_doc_ids_v1.txt
# fresh workspace OR soft-resume re-ingest with process_options=ite (P0_mm_ite)
bench047 smoke --profile P0_mm_ite --document-scope --workers 2
bench047 fidelity <artifact-dir>
# Gate: Chart answer_in_evidence ↑ vs ~0.32–0.36; Chart Acc ↑ vs ~0.14
```

Interactive board: canvas `spec047-modality-vision-plan.canvas.tsx`.
