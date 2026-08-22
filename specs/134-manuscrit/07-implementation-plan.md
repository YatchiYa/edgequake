# 07 — Implementation Plan

Principles: DRY / SOLID / LAW-134-*. Docs in this folder are the contract; code follows.
**This WP list is the implementation roadmap — code lands in follow-up PRs, not in the doc pack PR.**

## Sequence

```ascii
  WP-0  Docs + synthetic fixtures rubric          (this pack)
  WP-1  PageModality + heuristic classifier
  WP-2  ManuscriptProfile render floors (bypass DPI clamp)
  WP-3  Manuscript Pass-A prompt SSOT (fidelity, no EN pin)
  WP-4  Asset / figure-filter / Pass-B graphic-as-unit policy
  WP-5  Persist modality + confidence (DB + API)
  WP-6  Env / parse overrides + docs
  WP-7  UX chips
  WP-8  Contracts + e2e + Playwright + edge matrix
  ─────────────────────────────────────────────
  WP-9  MS Verify pass (Judge-and-Refine lite)     v1.5
  WP-10 Frontier VLM routing for MS                v1.5
  WP-11 Consensus confidence (two-VLM)             v1.5
```

**Ship slice A (backend honesty):** WP-1…WP-4 + contracts (no UI).  
**Ship slice B (persist + UX):** WP-5…WP-7 + Playwright.  
**Ship slice C (quality gates):** WP-8 gold CER harness.  
**Ship slice D (SOTA):** WP-9…WP-11 (verify, routing, consensus).  
**Ship slice E (page-as-unit, 2026-08-22):** WP-E2…E8 — Pass-A actually renders
the MS long-edge, per-page convert groups, assemble does not inject tiles,
EdgeParse veto, escalate containment, ImageGuard 2000 floor, print regression.

WP-1…D are **wired but not sufficient** without Slice E (live placeholder + crop gallery).

## WP-0 — Doc pack + fixtures rubric (Done with this folder)

- Spec pack complete; `fixtures/README.md` defines synthetic gold rules.
- No trigger document content in repo.

## WP-1 — PageModality + classifier

**Modules (new):** `edgequake-pdf/src/page_modality.rs` (or `manuscript/mod.rs`)

```rust
pub enum PageModality { Print, Manuscript, Mixed }
pub struct PageClassResult { pub modality: PageModality, pub score: f32 }
pub fn classify_page_heuristic(...) -> PageClassResult;
pub fn classify_document_majority(...) -> PageModality;
```

**Signals:** image area frac, glyph text density, ink frac proxy (optional light render), force env.

**Acceptance:** unit tests with synthetic PDF fixtures (print vs image-primary).

**Estimate:** 1–2 days

## WP-2 — ManuscriptProfile render

**Modules:** `manuscript_profile.rs`; wire in `backend/vision.rs` + `pdf_processing.rs`

| Field | Default |
|-------|---------|
| `dpi_floor` | 300 (`EDGEQUAKE_PDF_MANUSCRIPT_DPI`) |
| `max_rendered_pixels` | 3600 |
| `skip_edgeparse_fastpath` | true |

`resolve(modality, adaptive_dpi, env) -> VisionRender knobs`

**Blocker fix (verified):** `pdf_processing.rs:953` `.clamp(96, safe_dpi.max(96))` caps
DPI at the adaptive profile. ManuscriptProfile must **bypass or reorder** this clamp
when modality is manuscript. Also `backend/vision.rs:149` hardcodes
`max_rendered_pixels: 2000` — must become profile-driven.

**Acceptance:** contract asserts MS modality ⇒ dpi≥floor and max_pixels≥floor.

**Estimate:** 1 day

## WP-3 — Manuscript prompt SSOT

**Status: DONE (Slice B, 2026-08-20)** — routed on the production path in `pdf_processing.rs` (`route_pass_a_system_prompt`); precedence explicit upload > modality > print SSOT. Contract: `contract_spec134_strategy.rs`.

**Module:** `vision_prompts.rs`

- Add `RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT`
- Add `pass_a_system_prompt_for(modality) -> &'static str`
- Print path unchanged

**Acceptance:** snapshot/contract: MS prompt contains fidelity rules; print prompt still English Acc.

**Estimate:** 0.5–1 day

## WP-4 — Asset / filter / Pass-B policy

**Status: DONE (Slice B, 2026-08-20)** — SPEC-128 figure filter attach gated to Print modality (`should_attach_figure_filter`); manuscript pages treat embedded XObjects as scan-tiling artifacts (LAW-134-16), crop theater killed at source. Page PNGs/crops still written for the viewer; api-side suppression remains as defense-in-depth.

**Modules:** `figure_filter.rs`, multimodal service, assemble hooks, optional chart-band helper

Policy SSOT:

1. On MS modality, skip Pass-1 discard of `signature` when it would drop hand marks.
2. Full-page `page-NNNN.png` always retained for viewer.
3. **LAW-134-16:** Before Pass-B specialize, drop crops that are:
   - below `min_crop_area_frac` / `min_ink_frac`, **or**
   - classified as tick-strip / single-glyph aspect, **or**
   - children of a larger chart band (IoU) covering < `T_chart_frac` of that band.
4. Pass-A MS prompt owns whole-graphic KV/tables — Pass-B must not re-narrate fragments.

**Acceptance:**

- Mock tick-digit crops + single-bar crop → **zero** Pass-B analyze invocations.
- MS chart fixture MD contains series/Key values section (mock VLM returns gold shape).

**Estimate:** 1–2 days

## WP-5 — Persist modality + confidence

**Status: DONE (lite, Slice B, 2026-08-20)** — `page_modality`, `grounding_score`, `grounding_verified`, `grounding_low_pages` merged into document metadata via `patch_document_metadata` after the verify pass. Per-page DB columns remain future work.

**Migration:** add columns on `document_pages` (see DB lens).  
**API:** expose on page/layout GET.  
**Confidence heuristic (v1):** function of `[?]` rate, MD length vs ink, empty→0.0.

**Acceptance:** after convert, page row has `page_modality='manuscript'` for forced override fixture.

**Estimate:** 1–2 days

## WP-6 — Env / API overrides

| Env | Default |
|-----|---------|
| `EDGEQUAKE_PDF_MANUSCRIPT_DPI` | 300 |
| `EDGEQUAKE_PDF_MANUSCRIPT_MAX_PIXELS` | 3600 |
| `EDGEQUAKE_PDF_PAGE_MODALITY` | unset |
| `EDGEQUAKE_PDF_MANUSCRIPT_SKIP_EDGEPARSE` | true |

Update `.env.example`, AGENTS.md env table.

**Estimate:** 0.5 day

## WP-7 — UX chips

FE: `PageModalityChip`, `TranscriptionConfidence` per [06-ux-ui-spec.md](06-ux-ui-spec.md).  
Pass-B accordion collapsed for MS.

**Estimate:** 1–2 days

## WP-8 — Tests

See [08-test-protocol.md](08-test-protocol.md). Names:

- `contract_spec134_modality_profile`
- `contract_spec134_manuscript_prompt`
- `e2e_spec134_manuscript_convert` (mock VLM)
- Playwright `E2E-134-UI-chip`
- Gold metrics script (optional make target `spec134-proof`)

**Estimate:** 2–3 days

## WP-9 — MS Verify pass (Judge-and-Refine lite) — v1.5

**Status: DONE (pulled into Slice B, 2026-08-20)** — `edgequake-api/src/services/manuscript_verify.rs`: per-page judge against the page render (SSOT prompts in `vision_prompts.rs`), one refine pass on low verdict, still-low pages honestly marked `<!-- grounding:low score=... -->`. Fail-open on judge error/timeout/unparseable/missing PNG. Gates: `EDGEQUAKE_PDF_MANUSCRIPT_VERIFY` (default on), `EDGEQUAKE_PDF_MANUSCRIPT_VERIFY_MIN` (default 0.6). E2E: `e2e_spec134_grounding_verify.rs`.

**SOTA:** MinerU2.5-Pro Judge-and-Refine; OCR-Agent capability reflection.

**Modules:** new `manuscript_verify.rs` in `edgequake-pdf` or `edgequake-api` services.

**Behavior:**

1. After Pass-A MS, compute heuristic confidence (WP-5).
2. If confidence < threshold (default 0.6) OR `EDGEQUAKE_PDF_MANUSCRIPT_VERIFY=1`:
   - Second VLM call: page image + Pass-A MD → judge fidelity (missing tables? invented text? wrong language?).
   - If judge finds errors → refine MD (single pass, not loop).
3. Persist `verified: bool` + updated confidence.

**Gate:** `EDGEQUAKE_PDF_MANUSCRIPT_VERIFY` (default true).

**Acceptance:** mock low-confidence MS page → verify invoked; refined MD replaces original.

**Estimate:** 2 days

## WP-10 — Frontier VLM routing for MS — v1.5

**Status: DONE (env-only, Slice B, 2026-08-20)** — `EDGEQUAKE_VISION_PROVIDER_MANUSCRIPT` / `EDGEQUAKE_VISION_MODEL_MANUSCRIPT` applied in `pdf_processing.rs` (`resolve_vision_*_for_modality`); falls back to upload field > workspace > provider default. No vendor hardcoded; documented in `.env.example` + AGENTS.md.

**SOTA:** IAM leaderboard Aug 2026 — GPT-5 ~1.22% CER, Opus 4.7 ~1.31%, Gemini 3 ~1.44%.

**Modules:** `vision_env.rs`, docs.

**Behavior:**

- Document recommended MS models in `.env.example` and AGENTS.md.
- Optional: `EDGEQUAKE_VISION_MODEL_MANUSCRIPT` override (falls back to `EDGEQUAKE_VISION_MODEL`).
- Do **not** hardcode vendor; recommend frontier class.

**Acceptance:** docs list model matrix; env override works.

**Estimate:** 0.5 day

## P0 — Belief-store admission gate (Slice C, 2026-08-20)

**Status: DONE** — closes the three KG-pollution leaks measured on the live
assessment document after Slice B (32 `ASSETS/*.PNG` entities, a
`TRACTION_TEST_RESULTS` entity from a `grounding:low score=0.00` page, and
gear-sketch entities from Pass-B narration of a 270×324 tiling fragment).

**First principle:** verification must be an *admission gate*, not an
annotation. A marker the extraction stage cannot see is documentation, not
protection.

### P0-a — Scan-tiling fragment suppression

- `edgequake-pdf/src/embedded_images.rs`: `is_scan_tiling_page` — purely
  geometric predicate: ≥ 12 embedded images on the page AND median displayed
  area ≤ 2,000 pt² (bbox-first, pixel fallback). Calibrated on the live doc:
  21–47 tiles/page, medians 133–407 pt²; real figures are ≥ ~5,000 pt².
- `PageDrawingAssetsConfig.page_modality` (new field) carries the resolved
  modality into conversion; `backend/vision.rs` clears tiling pages from
  `figure_map` — the SSOT consumed by markdown link injection, `<drawing/>`
  analyze tags, and chart-residual logic, so all fragment channels close at
  once (including the Pass-B inline-analyze crop theater).

### P0-b — Quarantine lane (Display ≠ Index, LAW-134-4)

- `manuscript_verify.rs`: `strip_low_grounding_sections` — sections carrying
  `<!-- grounding:low ... -->` keep their page marker (provenance) but
  contribute no content to the index-bound text.
- Wired in `text_insert/prepare.rs` at the `processed_text` (index)
  construction; `text_content` (display SSOT: `documents.content` + KV mirror)
  keeps the full marked markdown for honest human review.

### P0-c — Tests

- Unit: tiling predicate (5 cases incl. gallery false-positive guard),
  quarantine filter (3 cases).
- Contract: `contract_spec134_belief_gate.rs` pins all three wirings + marker
  SSOT ownership.
- E2E: verify→quarantine chain in `e2e_spec134_grounding_verify.rs`.
- Regressions: 159 pdf + 1329 api lib tests green; clippy/fmt clean.

## Slice D — PDF→Markdown quality levers (2026-08-20)

Measured failures on doc `01a01ea2`: page 4 empty (11.9MB PNG), Pass-B English on a French page, KG mixed-language entities, verify fail-open with no reason.

### D-WP-1 — Image size guard (acquisition)

- `edgequake-pdf/src/image_guard.rs`: `ImageGuardProvider` wraps `dyn LLMProvider`.
- Budget `EDGEQUAKE_VISION_MAX_IMAGE_BYTES` (default 3.5MB). Over: PNG→JPEG q85, then downscale to the 1024px long-side floor, then quality 70/55/40.
- Wired once on Pass-A (`backend/vision.rs`), Pass-B (`vlm_provider_resolver.rs`), figure-filter / escalate / judge (`pdf_processing.rs`).

### D-WP-2 — Empty-page escalation (calibration)

- `escalate_empty_pages` in `manuscript_verify.rs`: pages whose body is exactly `EMPTY_VISION_PAGE_PLACEHOLDER` while a page PNG exists are re-OCR'd once.
- Runs before verify. Persists `pages_escalated` / `pages_failed`.

### D-WP-3 — Language fidelity (transduction)

- `detect_document_language` (stopword + script heuristic) on the first non-empty Pass-A page.
- `tokio::task_local` `DOCUMENT_LANGUAGE` in `edgequake-pipeline`: Pass-B `prompt_language()` and extractors' `effective_extraction_language()` read it.
- Judge/refine prompts restated: never translate.

### D-WP-4 — Verify observability

- `VerifyOutcome.fail_reason`: `judge_call_failed` / `page_png_missing` / `image_too_large` / `refine_call_failed`.
- One retry with 400ms backoff on judge failure.
- Persisted as `grounding_fail_reason`.

### D-WP-5 — Contracts

- `contract_spec134_quality.rs` pins all four wirings + French fixture detection.

## WP-11 — Consensus confidence (two-VLM) — v1.5

**SOTA:** Consensus Entropy (CVPR 2026) — multi-VLM agreement → accept / route.

**Modules:** `manuscript_consensus.rs` (optional feature).

**Behavior:**

1. If `EDGEQUAKE_PDF_MANUSCRIPT_CONSENSUS=1`:
   - Run Pass-A MS with two VLMs (primary + secondary).
   - Compute similarity (normalized edit distance or embedding cosine).
   - Low entropy (high agreement) → accept, confidence high.
   - High entropy → route to stronger model OR mark low confidence.
2. Persist consensus score.

**Gate:** `EDGEQUAKE_PDF_MANUSCRIPT_CONSENSUS` (default false — cost).

**Acceptance:** mock two-VLM disagreement → low confidence persisted.

**Estimate:** 2 days

## Slice E — Page-as-unit convert (2026-08-22)

Closes the live failure WP-1…D did not: Pass-A 2000px + crop inject on empty
pages. Python study: [`study/page_as_unit.py`](study/page_as_unit.py).

| WP | Change | Anchor |
|----|--------|--------|
| E0 | Spec truth (this folder) | ✅ `03-code-as-is.md`, LAW-134-20 |
| E1 | Python ablation (private PDF, gitignored `study/out/`) | ✅ `study/` — live go=true (mistral-small-latest) |
| E2 | Forward `max_rendered_pixels` into pdf2md Pass-A | ✅ `backend/vision.rs` |
| E3 | Per-page classify + `PageConvertPlan` + stitch | ✅ `page_convert_plan.rs`, `pdf_processing.rs` |
| E4 | Assemble: no fig/chart inject on MS or empty Pass-A; no caption/chart re-inject | ✅ `vision_markdown.rs`, `backend/vision.rs` |
| E5 | Classify / skip_edgeparse **before** SPEC-038 Auto | ✅ `pdf_processing.rs` |
| E6 | Escalate on placeholder containment; MS ImageGuard floor 2000 | ✅ `manuscript_verify.rs`, `image_guard.rs` |
| E7 | Print Acc byte-identical; print pages keep figure filter | ✅ e2e print guard + slice_e contract |
| E8 | Behavioral contracts (not `include_str!` only) | ✅ `contract_spec134_slice_e.rs` + pdf unit tests |

Defer: WP-7 UX chip, WP-11 consensus, classical HTR.

## PR split

| PR | Contents |
|----|----------|
| PR-docs | This folder only |
| PR-A | WP-1…4 + contracts |
| PR-B | WP-5…7 + Playwright |
| PR-C | Gold CER harness + fixtures PDFs |
| PR-D | WP-9…11 (SOTA verify / routing / consensus) |

## DRY / SOLID checklist (gate each PR)

- [x] Prompt text only in `vision_prompts.rs` (Pass-A MS + grounding judge + refine)
- [x] DPI/pixel math only in `ManuscriptProfile::resolve`
- [x] Classifier does not call VLM convert
- [x] FE does not re-detect modality
- [x] Print Acc path untouched (regression contract: `print_document_byte_identical_regression_guard`)

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
