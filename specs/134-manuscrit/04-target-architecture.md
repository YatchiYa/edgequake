# 04 — Target architecture

## Locked approach (v1)

**Manuscript page-class profile on existing Vision Pass-A.** No classical HTR binary.
Same ship posture as SPEC-128 L2 ONNX: optional later, not required to ship honesty.

```ascii
  PDF page bytes
       │
       ▼
  PageClassifier (heuristics ± optional tiny VLM)
       │
       │  print | manuscript | mixed
       ▼
  ┌─────────────────────────────────────────────────────┐
  │ ManuscriptProfile (when manuscript or mixed-hand)   │
  │  RenderProfile     PromptProfile      AssetPolicy   │
  │  DPI ≥ floor       MS system prompt   keep page PNG │
  │  max_px ≥ floor    source language    Pass-B budget │
  │  no EdgeParse FP   [?] unreadables    filter policy │
  └─────────────────────────────────────────────────────┘
       │
       ▼
  Pass-A VLM → per-page MD + page markers
       │
       ├─ persist page_modality + transcription_confidence
       ├─ assemble MD (print assemble path; prompt already selected)
       └─ Insert / KG (SPEC-133 delimiter-safe)
              │
              ▼
         UI: modality chip + confidence + side-by-side page PNG
```

## Components (SRP)

| Component | Responsibility | Non-responsibility |
|-----------|----------------|--------------------|
| `PageModality` | Enum + serde | Prompt text |
| `PageClassifier` | Bytes/page → modality + score | Render, VLM call |
| `ManuscriptProfile` | Resolve DPI, max_px, prompt id, filter flags | HTTP, DB |
| `pass_a_system_prompt_for` | Return `&'static str` SSOT | Classification |
| `pdf_processing` | Apply profile into `VisionConversionConfig` | Prompt authorship |
| Persist layer | Write modality/confidence on `document_pages` | Classification |
| UX | Read API fields; chip | Re-run vision |

## Classifier heuristics (v1, deterministic)

Order: force-env override → heuristics → optional VLM tie-break (off by default).

| Signal | Suggests manuscript |
|--------|---------------------|
| Image-primary page (large JPEG area frac) + low glyph text density | Yes |
| Low–mid ink fraction with non-uniform stroke (vs clean print) | Weak yes |
| Mixed orientation within doc | Soft prior |
| High CCITT tile count + sparse meaningful text | Scanner OCR residue → prefer Vision |
| Dense ruled/graph background + hand ink | Manuscript chart page |

Fail-open: classifier parse errors → `Print` (never block ingest). Mixed *pages*
in a mixed *document* convert with the manuscript group; print pages in that
file stay on the print group (Slice E — do not MS-prompt print Acc pages).

## Prompt contract (manuscript)

New constant `RAG_PAGE_MANUSCRIPT_VISION_SYSTEM_PROMPT` in `vision_prompts.rs`:

1. Transcribe **all** readable handwriting in **source language** (no translate).
2. Implicit tables → GFM with every readable cell; European decimals preserved.
3. Hand charts / multi-panel histograms / log plots → **one section per graphic**;
   series by **color/ink**; axis labels + scales; Key values / GFM of readable points;
   omit unreadables. Never describe a single bar or tick digit as the whole figure.
4. Diagrams → list labels + directed relations as bullets (names without raw `->` chains in entity slots — or rely on SPEC-133 parse).
5. Strikeouts: mark as struck / omit from index per LAW-134-4 (document choice in WP-3: omit from MD body, keep note).
6. Unreadable → `[?]` only; no invention.
7. Output Markdown only; no crop monologue; no geometric “lines and frames” essay for chart fragments.

Print prompt remains Acc SSOT for print modality.

## Render contract

| Setting             | Print (keep)    | Manuscript                                      |
| ---------------------| -----------------| -------------------------------------------------|
| DPI                 | adaptive 96–150 | `max(adaptive, MANUSCRIPT_DPI)` default 300     |
| max_rendered_pixels | 2000            | `max(2000, MANUSCRIPT_MAX_PIXELS)` default 3600 |
| Concurrency         | existing        | Cap ≤2 local when DPI≥250 (VRAM)                |
| EdgeParse fast-path | allowed (Auto)  | **forbidden** (classify first; honor skip flag) |
| Pass-A long-edge    | 2000 (pdf2md default) | **3600 forwarded** (pdf2md ignores dpi)   |
| ImageGuard floor    | 1024            | **2000** (JPEG q85 first; no 1024 crush)        |
| Fragment inject     | fig/chart hrefs | **off** (LAW-134-20); page PNG viewer-only      |
| Verify pass         | none            | Judge-and-Refine when confidence low (WP-9)     |
| Vision model        | fleet default   | Frontier VLM recommended (WP-10)                |
| Consensus           | none            | Optional two-VLM (WP-11)                        |

## Asset / Pass-B policy

```ascii
  modality == manuscript?
       │
       ├─ always keep page-NNNN.png for viewer
       ├─ Pass-A MD is RAG SSOT (includes whole-graphic digitization)
       ├─ figure filter: skip signature discard OR treat MS crops as keep-for-overlay-only
       └─ Pass-B / region specialize — SUPPRESS when any of:
              area_frac < T_noise
              ink_frac < T_ink
              aspect looks like tick-strip / single glyph (narrow digit crop)
              crop is a fragment of a larger chart bbox (IoU child of chart region)
              (LAW-134-16 graphic-as-unit)
              constants move only with gold Δ — LAW-134-10
```

**Chart region rule (v1):** If Pass-A or layout marks a page as hand-chart dense, prefer
**zero** Pass-B cards for that page unless a crop covers ≥ `T_chart_frac` of the
detected chart band (default start `0.35`). Overlay may still show L1 boxes; index does not.

## Data model (WP-5)

Prefer typed columns on `document_pages` (SPEC-091 / SPEC-128 grain):

| Column                     | Type          | Notes                     |                        |         |
| ----------------------------| ---------------| ---------------------------| ------------------------| ---------|
| `page_modality`            | text/enum     | `print` \                 | `manuscript` \         | `mixed` |
| `transcription_confidence` | real nullable | 0..1 or null if unknown   |                        |         |
| `vision_profile`           | text nullable | `print` \                 | `manuscript` for audit |         |
| `verified`                 | bool nullable | WP-9 Judge-and-Refine ran |                        |         |
| `consensus_score`          | real nullable | WP-11 two-VLM agreement   |                        |         |

API: extend `GET .../pages/{n}` or layout payload with these fields (lazy, LAW-128-12 style).

## Sequence (happy path) — Slice E per-page groups

pdf2md 0.9.11 has one `ConversionConfig` per `convert_from_bytes` and ignores
`dpi` at raster time (`max_rendered_pixels` is the long-edge). Slice E does
**not** wait for per-page DPI inside pdf2md.

```ascii
  1 classify_pages_from_bytes  BEFORE EdgeParse Auto
  2 PageConvertPlan: print_pages vs manuscript_pages
       unsampled pages inherit document majority
  3 groups:
       all print → one convert (Acc prompt, 2000px, figure filter ON)
       all MS    → one convert (MS prompt, 3600px forwarded, no fragment inject)
       mixed     → two converts with PageSelection::Set + stitch_page_markdown_in_order
  4 assemble: LAW-134-20 — MS or empty Pass-A never inject fig/chart hrefs
  5 escalate if placeholder still present (containment, not exact body)
  6 [WP-9] verify on manuscript-like document modality
  7 persist MD + page_modality + page_modalities
  8 enqueue Insert
```

`classify_document_majority` is **metadata / verify gating only**. Convert
policy is the grouped plan so print pages in a mixed file keep Acc English +
figure filter (no regression on text+figures pages).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- As-is: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- UX: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
