# SPEC-134 — Manuscript / handwritten page vision

> **Mission:** Make Vision Pass-A treat handwritten / MFD-scanned technical pages as
> **whole visual units** — fidelity-first transcription, class-routed DPI, manuscript
> prompts, honest confidence — so RAG indexes text, implicit tables, and hand charts
> instead of hallucinating on scribble crops.
>
> **Trigger:** [`zz-raw.md`](zz-raw.md) (structural intake only; no source filename or
> content quotes).

## Short verdict

| Layer | Finding |
|-------|---------|
| Gap A | Pass-A prompt is **print + English Acc**; manuscript pages are paraphrased, translated, or under-transcribed |
| Gap B | Adaptive DPI **96–150** + `max_rendered_pixels=2000` blurs thin ink, ticks, color series |
| Gap C | Figure filter + Pass-B specialize **fragments** (scribbles, axis ticks, single bars) while page/graphic body is ignored |
| Gap D | Scanner OCR / EdgeParse text layer is sparse or lying; Auto may skip real vision |
| Gap E | Hand charts never digitized **as a unit** (series × color × axis) in Pass-A MD |
| Fix | `PageModality` → per-page `PageConvertPlan` + `ManuscriptProfile` (3600px **forwarded** to pdf2md, MS prompt, **no crop inject**, EdgeParse veto, persist modality) |
| Non-goals | Classical HTR binary (TrOCR/Kraken); replace pdfium; claim archival CER |

```ascii
  WHY RAG fails on manuscript class
  ─────────────────────────────────
  Pixels hold meaning that glyphs do not
       │
       ├─ Print prompt + EN pin     → paraphrase / translate / invent cleanup
       ├─ DPI 96–150 / 2k cap       → thin strokes, ticks, color blocks blur
       ├─ Pass-B fragment theater   → scribble / tick / bar crops narrated
       ├─ EdgeParse / Auto text     → empty or misleading OCR layer
       ├─ Chart rules assume print  → hand plots never get whole-graphic KV
       └─ Diagram arrows in names   → KG fragility (SPEC-133 sibling)
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-134-*)
  → 02-cross-ref-matrix
  → 03-code-as-is
  → 04-target-architecture
  → 05-lenses/ (PO, fullstack, DB, UX, front, AI, OCR, PDF)
  → 06-ux-ui-spec
  → 07-implementation-plan
  → 08-test-protocol
  → 09-acceptance
  → 10-edge-cases
  → 11-honest-assessment
  → 12-sota-assessment (Aug 2026)
  → zz-raw.md (intake, not the contract)
  → fixtures/ (synthetic only)
```

## Status board

| ID  | Item                                   | Status       |
| -----| ----------------------------------------| --------------|
| D0  | Intake `zz-raw.md` (structural)        | Done         |
| D1  | Doc pack (this folder)                 | Done         |
| I0  | WP-1 PageModality + classifier         | Done (heuristic wired into `pdf_processing.rs`; env override wins) |
| I1  | WP-2 ManuscriptProfile render          | Done         |
| I2  | WP-3 Manuscript Pass-A prompt SSOT     | Done (routed in `pdf_processing.rs`; precedence explicit > modality > print — contract-pinned) |
| I3  | WP-4 Figure-filter / drawing policy    | Done (real crop geometry gates + filter attach gated to Print modality — crop theater killed at source) |
| I4  | WP-5 Persist modality + confidence     | Done (WP-5 lite: `page_modality`, `grounding_score`, `grounding_verified`, `grounding_low_pages` in doc metadata) |
| I5  | WP-6 Env / API override                | Done (env)   |
| I6  | WP-7 UX modality chip                  | Planned      |
| I7  | WP-9 MS Verify pass (Judge-and-Refine) | Done (`manuscript_verify.rs`: judge → refine-once → honest `grounding:low` marker; fail-open; env-gated) |
| I8  | WP-10 Frontier VLM routing for MS      | Done (env-only: `EDGEQUAKE_VISION_PROVIDER_MANUSCRIPT` / `EDGEQUAKE_VISION_MODEL_MANUSCRIPT`; no vendor hardcoded) |
| I9  | WP-11 Consensus confidence             | Planned v1.5 |
| I10 | P0-a Scan-tiling fragment suppression  | ✅ Done (geometric rule `is_scan_tiling_page` — count ≥ 12 + median tile ≤ 2,000 pt², calibrated on live doc; modality-gated at `figure_map` source so markdown links, `<drawing/>` tags, and chart residuals all close at once) |
| I11 | P0-b Quarantine lane (Display ≠ Index) | ✅ Done (`strip_low_grounding_sections`: `grounding:low` sections stay in stored/display markdown but are removed from the chunking/extraction input in `text_insert/prepare.rs`) |
| I12 | P0-c Belief-gate contracts + e2e       | ✅ Done (`contract_spec134_belief_gate.rs` pins all three wirings; e2e verify→quarantine chain) |
| T1  | WP-8 contracts + e2e + Playwright      | Partial (contracts + e2e mock done; Playwright pending) |
| I13 | Slice D WP-1 Image size guard          | Done (`ImageGuardProvider`: PNG→JPEG q85 then downscale; wraps Pass-A / Pass-B / judge / escalate; `EDGEQUAKE_VISION_MAX_IMAGE_BYTES`) |
| I14 | Slice D WP-2 Empty-page escalation     | Done (`escalate_empty_pages` before verify; persists `pages_escalated` / `pages_failed`) |
| I15 | Slice D WP-3 Language fidelity         | Done (detect from first non-empty Pass-A page; task-local → Pass-B `prompt_language` + extraction; never-translate on judge/refine) |
| I16 | Slice D WP-4 Verify observability      | Done (`fail_reason` + one judge retry; persisted as `grounding_fail_reason`) |
| I17 | Slice D WP-5 Quality contracts         | Done (`contract_spec134_quality.rs`) |
| I18 | Slice E page-as-unit convert           | ✅ Wired in tree (contracts green). **Not** live-reconverted on the trigger PDF; see [11-honest-assessment.md](11-honest-assessment.md) — channel fix ≠ grounded HTR |

> **Slice E trigger (2026-08-22):** operator side-by-side still showed
> `EMPTY_VISION_PAGE_PLACEHOLDER` plus scan-fragment crops on a 4-page
> manuscript PDF. WP-1…D were wired; Pass-A never received the 3600px raster
> (pdf2md default 2000, DPI ignored); assemble prepended figs onto the
> placeholder and defeated empty-page retry. A photo-to-VLM path recovers the
> page — EdgeQuake must do the same: **full-page raster + class-routed prompt**.

> **P0 evidence (2026-08-20 live run, post-Slice B):** the belief-store leaks were
> measured, not hypothesized — 32 `ASSETS/*.PNG` entities from tiling-fragment
> links, `TRACTION_TEST_RESULTS` from a `grounding:low score=0.00` page, and
> gear-sketch entities from Pass-B narration of a 270×324 tiling fragment.
> P0 closes all three channels; the fragment rule fires on all 4 pages of the
> assessment document (21–47 tiles/page, median 133–407 pt²).

## Related

- [SPEC-015](../015-vision-parser/) — Vision Pass A/B
- [SPEC-047](../047-rag-evaluation/) — Acc / vision-first battle plans
- [SPEC-049](../049-improve-figure-extraction/) — figure cascade; display ≠ index
- [SPEC-096](../096-multi-language-extraction/) — extraction language
- [SPEC-128](../128-improve-pdf-parsing/) — PDF vision + layout overlay SSOT
- [SPEC-133](../133-kv-error/) — `->` delimiter collision after diagram extract
- [SPEC-038](../038-ingestion-large-pdf/) — Adaptive DPI / Auto EdgeParse
- Sibling crate: `edgequake-pdf2md` (pdfium render + VLM page convert)

## Non-goals (v1)

- Shipping Kraken / TrOCR / AGPL HTR weights in the product binary
- Replacing pdfium or Pass-A architecture
- Claiming human-proof archival HTR accuracy
- Naming or quoting the trigger document / publishing its content
- Rewriting SPEC-133 delimiter logic (cross-ref only)

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
