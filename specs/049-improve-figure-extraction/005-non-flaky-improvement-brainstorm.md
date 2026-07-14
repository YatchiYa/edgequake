# 005 — Non-flaky improvement brainstorm (first principles)

> Goal: raise figure/table/chart fidelity **without** introducing detectors that
> fail differently across locale, authoring tool, or “almost the same” papers.
> Companion to [001-first-principles](./001-first-principles.md) and measured
> quality on `specs/048-improve-ux/e2e/` (≈95.8% labeled-figure recall).

## 0. Define “flaky” rigorously

| Class | Definition | Allowed? |
|-------|------------|----------|
| **Authoritative** | Decision entailed by ISO 32000 (StructTree role, XObject subtype, CTM×BBox at `Do`) | Always prefer |
| **Pinned geometry** | Pure function of paint geometry with **frozen constants** + golden corpus lock | Yes, if change requires eval delta |
| **Calibrated model** | Frozen weights + version pin + CI mAP/IoU gate; never overrides L0/L1 | Yes, residual only |
| **Flaky heuristic** | English keywords, magic caption ceilings, ad-hoc per-PDF knobs, invent paths | **Ban as primary** |

**Axiom F1:** Same PDF bytes → same proposals (bit-identical bounding boxes at fixed render DPI).  
**Axiom F2:** A constant is not “not a heuristic” — it is a **product invariant**. Changing it without a corpus Δ is flaky process, even if the math is deterministic.  
**Axiom F3:** Visual PNG exists only when text cannot carry the meaning (modality split in 001).

## 1. Inventory — soft debt still in the stack

| Location | Signal today | Class | Risk |
|----------|--------------|-------|------|
| `caption_label.rs` distance &lt; 80 pt | Label attach | Pinned geometry (OK as labeler) | Flaky **if** used to invent crops |
| `geometry.rs` IoU 0.05 / gap 12 pt / area 2–55% | Cluster + reject | Pinned geometry | Over-merge / under-merge without golden lock |
| `object_cluster.rs` rule `h≤2.5 ∧ w≥80` | Table vs figure | Soft composition cue | False table on wide rules; false figure on grids |
| `region_assets` “any ImageXObject ⇒ skip all region figs” | Merge policy | **Logic bug / policy heuristic** | Drops co-located Forms (LightRAG-class miss) |
| `chart_crop::text_suggests_chart` | Residual proposal | **Flaky heuristic** | Pass-A English; P3 debt |
| `include_pdf_assets` page filter via `"figure "` / `"table "` | Which pages get assets | Soft gate | Misses visual-only pages; over-includes text-table pages |
| `UnavailableStructTreeProposer` | L0 | Missing authority | Untagged-only effective cascade |
| Extra unlabeled crops (Ideas 21 vs 10 captions) | Over-proposal | Precision debt | Noise for VLM / Drawing |

## 2. Non-flaky levers (ordered by authority × leverage)

### Lever A — Real StructTree L0 (authoritative)

**Principle:** When `/StructTreeRoot` exists, roles `Figure` / `Table` / `Caption` + MCID → bbox are ISO facts, not guesses.

**How (non-flaky):**
1. Prefer a pdfium-render build that exposes `page.struct_tree()` publicly (kreuzberg fork / upstream when `page_handle` visibility allows), **or** thin FFI: `FPDF_StructTree_GetForPage` → walk Figure/Table → MCID quads.
2. Telemetry: `% pages with non-empty tree`, `% proposals with `RegionSource::StructTree``.
3. Corpus: add **one tagged** fixture (PDF/UA export) — L0 must win over L1 on IoU≥0.5 for same element (G4).

**Does not help:** Our three arXiv GenPDF fixtures are `Tagged: no` — L0 is necessary for product breadth, not for current corpus recall.

### Lever B — CTM-traced XObject inventory (authoritative L1 upgrade)

**Principle:** A Form/Image is a figure candidate **iff** a content-stream `Do` paints it. Dictionary presence alone is not placement. `/BBox` is form-space; page-space = CTM_at_Do × Form.Matrix × BBox.

**How (non-flaky):**
1. Single-pass content-stream walker (lopdf / pdfium ops): track CTM; on `Do` resolve `/XObject`; emit `Placement { subtype, page_quad, nest_depth }`.
2. **Atomic proposals from placements** — each Image `Do` and each top-level Form `Do` is a proposal seed.
3. Cluster **only** when IoU ≥ pinned threshold **or** Form nests Image (parent Form wins).
4. Fix same-page skip: merge by IoU with existing ImageXObject figs; never “any embed ⇒ skip all”.

**Closes:** LightRAG Figure 2 class (missed Form placement); nested Form under-count; over-merge of distant panels.

**Reference practice:** content-stream CTM walk (PDFBox / PDF Oxide / edgeparse chunk_parser) — deterministic, no English.

### Lever C — Residual ink mask (deterministic residual; kills keyword chart)

**Principle:** After claiming L0+L1 quads, remaining **non-background ink** on a page raster is the only lawful residual signal.

**How (non-flaky):**
1. Render page at fixed DPI → binary mask (luma below pinned threshold **or** α).
2. Subtract dilated claimed regions.
3. Connected components with area ∈ [MIN, MAX]; emit `RegionSource::InkResidual`.
4. **Delete** `text_suggests_chart` as proposal source. Pass-A text may still **route** VLM specialize (“chart vs photo”) **after** a crop exists.

**Why not flaky:** Same bytes + same DPI + same threshold → same components. Threshold is a pinned invariant with golden PNGs, not an English list.

### Lever D — Precision: proposal cardinality control

**Principle:** Over-extraction (Ideas 21 crops / 10 captions) is a precision failure that looks like “success” in recall metrics.

**How (non-flaky):**
1. Prefer Form/Image placements over free path unions when both explain the same ink (IoU).
2. Suppress path-only clusters that are subsets of a Form placement (containment ≥ 0.8).
3. Multi-panel: keep N Form placements as N figs; do not glue by large gap merge unless StructTree says one Figure.
4. Unlabeled crops allowed only if `has_image || has_form`; path-only unlabeled → drop (or ink-residual only).

### Lever E — Text-channel structure (not PNG) for glyph tables

**Principle:** Already locked in 001. Improvement is **markdown lattice quality**, not `-table-` invent.

**How (non-flaky):**
1. Lattice from long path rules + glyph column alignment (geometry), or StructTree `Table`/`TR`/`TD` when tagged.
2. Eval: cell F1 on a table fixture — separate from visual crop suite.
3. Never gate visual include on the string `"table "`.

### Lever F — Calibrated L2 only as empty-page residual

**Principle:** Frozen DocLayout-class ONNX is acceptable **iff** L0+L1+ink empty, weights hashed in repo/CI, and model may not override higher layers.

**Ban:** Online prompt-tuned “find the figure” without schema + area gate (that is L3 and must stay last).

## 3. Decision framework — when to accept a constant

Before adding/changing any threshold (`CLUSTER_*`, area frac, ink luma, caption label distance):

1. **State the ISO or paint fact** it approximates (or admit “product safety”).
2. **Lock a golden corpus** (048 e2e + synthetic vector/table/embedded) — bit or IoU tolerance.
3. **Require Δ metrics:** labeled recall, unlabeled crop rate, G3 fail count, invent-path count.
4. **Forbid paper-specific knobs** (`if filename.contains("ideas")`).

If you cannot write (1)–(3), the change is flaky by process.

## 4. Eval harness (anti-flake CI)

| Gate | Metric | Fail if |
|------|--------|---------|
| G1 | Invented hrefs | &gt; 0 |
| G2 | Drawing uses `page-NNNN.png` | &gt; 0 |
| G3 | Crop area &gt; 55% | &gt; 0 |
| G6 | Labeled figure recall on 048 corpus | &lt; 95% (or regress &gt; 1 fig) |
| G7 | Unlabeled crop rate | &gt; 1.5× caption count without Form/Image |
| G8 | Chart keyword proposals | = 0 after Lever C |
| G9 | StructTree coverage telemetry | logged (informational until tagged fixtures) |

## 5. Recommended sequence (no drama)

| Phase | Lever | Outcome |
|-------|-------|---------|
| **P1a** | Fix same-page ImageXObject skip (IoU merge) | Cheap; recovers co-located Forms |
| **P1b** | CTM `Do` placement inventory | True L1; LightRAG Fig 2 class |
| **P1c** | Precision: Form-first; suppress path subsets | Ideas 21 → ~10–12 |
| **P2** | Ink-residual replaces `text_suggests_chart` | Non-flaky chart residual |
| **P3** | Real StructTree L0 + tagged fixture | Authority for PDF/UA |
| **P4** | Optional frozen L2 ONNX | Hard pages only |
| **P5** | Lattice text tables (separate track) | RAG quality ≠ PNG |

## 6. Explicit non-goals (keep out of “improve further”)

- Per-paper prompt engineering to “find Figure N”
- Raising `MAX_AREA_FRAC` to “catch” misses (re-opens full-page dumps)
- Caption regex expansion (`Abbildung`, `Fig.`, …) as **detectors**
- Second PDF engine “just for figures” without proving Pdfium cannot CTM-walk
- Treating unlabeled crop count as success

## 7. One-sentence north star

**Propose only from paint or tags; label from nearby text; residual from leftover ink; never from English — and freeze every threshold behind a corpus gate.**
