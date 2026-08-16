
### Title
```text
SPEC-049: High-precision figure extraction — prune filter, geometry gates, and page layout extraction (L2)
```

Local version to modify before publishing to crates.io and link to edgequake

/Users/raphaelmansuy/Github/03-working/edgequake-llm
/Users/raphaelmansuy/Github/03-working/edgequake-pdf2md


### Suggested labels
`enhancement`, `pdf`, `vision` (adjust to repo labels)

### Body

```markdown
## Summary

Figure extraction remains **recall-heavy**: Image/Form XObjects and path clusters are proposed aggressively, and the two-pass VLM `FigureFilter` classifies crops but does **not** always remove discarded assets from the authoritative `figure_map` used at markdown assembly. Logos, stamps, scan artefacts, and decorative bitmaps become figure assets, inflate VLM cost, and pollute downstream GraphRAG.

This issue tracks a full SPEC-049 rewrite and implementation plan that:

1. **Closes the filter control loop** (classify → prune)
2. **Tightens deterministic geometry** for images (area + aspect)
3. Adds **page layout extraction (L2)** as an optional ONNX gate (DocLayout-YOLO class models)
4. Keeps **L0 StructTree / L1 paint** as primary proposers
5. Pins constants and model hashes behind corpus gates

**In-tree reference (existing work):**
- `specs/049-improve-figure-extraction/` (first principles, architecture, implementation, acceptance)
- `crates/edgequake-pdf/src/figure_filter.rs`
- `crates/edgequake-pdf/src/backend/vision.rs`
- `edgequake-pdf2md` visual pipeline (`object_cluster`, `struct_tree`, `geometry`, `extract_images`)

**Repo:** https://github.com/raphaelmansuy/edgequake

---

## Problem statement

A PDF has **no** native `Figure` paint type ([ISO 32000](https://pdfa.org/resource/iso-32000-2/) paint + structure model). Visuals are Image XObjects, Form XObjects, paths, and optional StructTree roles.

Current pipeline is intentionally recall-oriented:

1. Extract almost all Image XObjects (≥24 px).
2. Propose Form/path clusters with weak area gates for **images** (image branch can skip `MIN_AREA_FRAC`).
3. Optionally run two-pass VLM filter that **classifies** but historically may leave discarded crops in `figure_map` used at assemble.

**North star:** Propose from paint or tags; extract **page layout** to know what is a figure on the page; gate and filter semantically; **prune** the asset set; index only survivors; freeze thresholds and model hashes behind corpus gates.

---

## Architecture (cascade of truth)

```text
PDF bytes
  │
  ├─ [Optional] Page routing (text-only → skip figure path)
  │
  ├─ L0 StructTree Figure/Table          (pdfium-render)
  │     https://github.com/ajrcarey/pdfium-render
  │     https://crates.io/crates/pdfium-render
  │
  ├─ L1 Paint proposals                 (Image/Form seeds + path residual)
  │     + page-area / aspect / entropy gates for images
  │
  ├─ L2 Page layout extraction          (ONNX DocLayout-class)
  │     ort: https://docs.rs/ort
  │     guide: https://ort.pyke.io/
  │     ONNX Runtime: https://onnxruntime.ai/
  │     model family: https://github.com/opendatalab/DocLayout-YOLO
  │     paper: https://arxiv.org/abs/2410.12628
  │     weights (DocStructBench): https://huggingface.co/juliozhao/DocLayout-YOLO-DocStructBench
  │     community ONNX: https://huggingface.co/wybxc/DocLayout-YOLO-DocStructBench-onnx
  │
  ├─ Write PNG only for survivors
  │
  ├─ L3 FigureFilter (VLM Pass-1/2)      (edgequake_llm)
  │     + concurrent Pass-1
  │     + prune figure_map + optional delete discarded PNGs
  │
  └─ Assemble markdown / Drawing tags from kept set only
        + OpenTelemetry GenAI spans
          https://opentelemetry.io/docs/specs/semconv/gen-ai/
```

**Authority order:** L0 > L1 > L2 > L3. Lower layers must not override higher for the same visual.

### Page layout extraction (L2) — definition

Given a **full-page raster**, run a document layout detector that returns labeled boxes in **page image coordinates**, map to PDF space, and **gate** L1 proposals (default mode: layout-as-gate, not layout-as-sole-proposer).

DocStructBench-style classes (example model card: https://huggingface.co/anyformat/doclayout-yolo-docstructbench):

| Class | Role |
|-------|------|
| `figure` | Primary keep target |
| `figure_caption` | Labeling aid |
| `table` / captions | Table path |
| `title` / `plain_text` / `abandon` | Reject zones for noise |
| `isolate_formula` / `formula_caption` | Usually not figure assets |

Broader toolkits for comparison / offline A/B: [PDF-Extract-Kit](https://github.com/opendatalab/PDF-Extract-Kit), [MinerU](https://github.com/opendatalab/MinerU). Rust ONNX CV reference with DocLayout demos: [usls](https://github.com/jamjamjon/usls).

### Banned as primary detectors

- Searching `"Figure "` / `"Fig. "` / `"Table "` to **invent** crops
- Magic vertical ceilings above captions as sole geometry
- Emitting Drawing targets for full-page `page-NNNN.png`
- Inventing on-disk asset paths that were never written
- Manifest-only classification **without** pruning `figure_map`

### Caption role

Captions **label** an already-proposed region. They do not create the region.

### Modality split (tables)

| What the PDF has | Channel | Visual `-table-` crop? |
|------------------|---------|-------------------------|
| Selectable cell glyphs / rules → MD/HTML | Text ingest | **No** |
| Image of a table / unreliable text | Visual | **Yes** |
| Heavy vector grid that loses structure in text | Visual residual if area OK | **Yes** |

**Axiom:** Visual extraction recovers meaning **text cannot carry**.

---

## Detailed implementation plan

### WP-0 — Close the filter control loop (P0)

**Modules**
- `crates/edgequake-pdf/src/backend/vision.rs`
- `crates/edgequake-pdf/src/figure_filter.rs`

**Required behavior**
1. After successful `FigureFilter::run`:
   - Build `kept` set where `is_figure == true`
   - Rebuild `figure_map` to only kept paths; drop empty pages
2. Optionally delete discarded PNGs under `assets_root`
3. Log kept / discarded counts by `FigureKind`
4. Fail-open on filter error remains default unless config sets fail-closed for tiny crops

**Acceptance:** unit/contract — mock 3 keep / 2 discard → assemble references only 3. Extend `edgequake-api/tests/contract_spec049_figure_filter.rs` if present.

**Estimate:** 0.5–1 day

---

### WP-1 — Default-enable semantic filter when vision LLM exists

**Modules:** workspace / API config that builds `PageDrawingAssetsConfig` (`figure_filter_provider`)

**Behavior**
- If page vision conversion has a resolved LLM provider and `extract_figures` is true, set `figure_filter_provider` unless `EDGEQUAKE_FIGURE_FILTER=0`
- Document env/config in README + SPEC-049 changelog

**Estimate:** 0.5 day

---

### WP-2 — Deterministic geometry gates for images (P1)

**Modules**
- `edgequake-pdf2md/.../object_cluster.rs` — `region_area_ok`
- `edgequake-pdf2md/.../struct_tree.rs` — `l0_area_ok`
- `edgequake-pdf2md/.../extract_images.rs`
- `edgequake-pdf2md/.../geometry.rs`

| Gate | Proposed invariant |
|------|--------------------|
| Min page-area for **images** | `MIN_IMAGE_AREA_FRAC` (start `0.008`–`0.02`) |
| Max page-area | keep `MAX_AREA_FRAC = 0.55` |
| Max aspect ratio | `MAX_ASPECT ≈ 8.0` |
| Min edge | keep 24 px / 24 pt |

**Process rule:** any constant change requires corpus Δ on fixtures under `specs/048-improve-ux/e2e/` and pdf2md `test_cases/`.

**Estimate:** 1–2 days

---

### WP-3 — Expand discard taxonomy + prompts

**Modules**
- `figure_filter.rs` — `FigureKind`
- `vision_prompts.rs` — `FIGURE_FILTER_PASS1_SYSTEM`

Add discard kinds (`is_figure == false`): `Stamp`, `Signature`, `ScanArtefact`, optional `Watermark`.

**Estimate:** 0.5 day

---

### WP-4 — Concurrent Pass-1 / budgeted VLM

**Modules:** `figure_filter.rs`

- Concurrent Pass-1 (e.g. `futures::stream` + `buffer_unordered`)
- Config: `figure_filter_concurrency` (default 4–8)
- Soft cap: `max_figure_vlm_calls` (drop lowest-area candidates first after geometry)

**Estimate:** 1 day

---

### WP-5 — Page routing

Skip figure asset writes when page has zero Image/Form and empty L1 residual clusters. Optional later: pure-Rust doc router (evaluate [pdf-inspector](https://crates.io/crates/pdf-inspector) without hard dep).

**Estimate:** 1 day

---

### WP-6 — Page layout extraction L2 (ONNX) — feature-gated

**Modules**
- New: `edgequake-pdf2md/.../layout_page.rs` (or `edgequake-pdf` feature module)
- Wire after L1 refine, before crop write **or** as IoU filter on proposed bboxes using **shared page PNG**

**Stack**
- Inference: [ort](https://docs.rs/ort) / [ort.pyke.io](https://ort.pyke.io/) over [ONNX Runtime](https://onnxruntime.ai/)
- Model family: [DocLayout-YOLO](https://github.com/opendatalab/DocLayout-YOLO) — export offline; pin path + SHA-256
- Optional reference: [usls](https://github.com/jamjamjon/usls) DocLayout demos

**Trait sketch**

```rust
pub trait PageLayoutExtractor: Send + Sync {
    fn extract(&self, page_png: &image::DynamicImage) -> Result<PageLayout, LayoutError>;
}

// Keep L1 proposal if:
//   IoU(proposal, any figure_layout_box) >= LAYOUT_IOU
//   OR source == StructTree
//   OR (has_form && FORM_LAYOUT_EXEMPT)
//   OR layout disabled
```

**Ops**
- Feature flag: `layout-onnx` (default **off** until gates pass)
- CPU EP default; optional CUDA via ort execution providers: https://ort.pyke.io/perf/execution-providers
- **License review** before bundling weights (upstream DocLayout-YOLO repo is AGPL-3.0; verify model-card license per artifact)

**Dev spike (not production download):**
1. https://huggingface.co/juliozhao/DocLayout-YOLO-DocStructBench
2. Export ONNX (`imgsz=1024`) per upstream README
3. Or evaluate https://huggingface.co/wybxc/DocLayout-YOLO-DocStructBench-onnx
4. Vendor under `models/` + SHA-256 in CI

**Estimate:** 3–5 days

---

### WP-7 — Observability

Align with [OpenTelemetry GenAI semantic conventions](https://opentelemetry.io/docs/specs/semconv/gen-ai/):

| Signal | Examples |
|--------|----------|
| Counters | `xobjects_seen`, `geometry_kept`, `layout_kept`, `vlm_kept`, `vlm_discarded_by_kind` |
| Histograms | `layout_ms`, `pass1_ms`, `pass2_ms` |
| Spans | `figure.propose`, `figure.layout`, `figure.filter.pass1`, `figure.filter.pass2` |

**Estimate:** 1–2 days

---

### WP-8 — Evaluation harness & fixtures

Extend gates from `specs/049-improve-figure-extraction/004-acceptance-and-tests.md`:

| Gate | Assert |
|------|--------|
| G1 | No invented asset paths |
| G2 | Full-page `page-NNNN.png` never Drawing-eligible |
| G3 | Crop area ≤ 55% of page |
| G6 | Labeled figure recall on 048 e2e corpus (no regression > 1 fig) |
| G7 | Unlabeled crop rate bounded |
| **G-prune** | After filter, `\|figure_map\| == kept count` |
| **G-layout** | Layout rejects absent from final assets; L0 preserved |
| **G-layout-coord** | Synthetic page: known figure rect IoU≥0.5 after projection |
| **G-industrial** | Logo/stamp discarded; real diagram kept |
| **G-cost** | VLM calls/page ≤ budget when gates on |

**Fixtures:** keep `ideas_*`, `hierar_*`, `lighrad_*`, tagged/embedded/vector samples; **add** synthetic multi-object page (logo + stamp strip + large diagram).

**Estimate:** 1–2 days

---

## Suggested sequence

```text
Week 1:  WP-0 prune · WP-1 default filter · WP-2 geometry · WP-3 taxonomy · WP-8 fixtures start
Week 2:  WP-4 concurrency · WP-5 router · WP-7 telemetry · WP-8 CI gates
Week 3+: WP-6 layout-onnx (optional) · docs · CHANGELOG
```

**Precision release (ship without L2):** WP-0, WP-1, WP-2, WP-3, G-prune + G6/G7 green.  
**Layout release:** above + WP-6 feature-flagged, weights pinned, G-layout / G-industrial green.

---

## Configuration surface

| Key | Default | Meaning |
|-----|---------|--------|
| `extract_figures` | true | Existing |
| `figure_filter` | true if LLM present | WP-1 |
| `EDGEQUAKE_FIGURE_FILTER` | unset | `0` forces off |
| `min_image_area_frac` | 0.008–0.02 | WP-2 |
| `max_figure_aspect` | 8.0 | WP-2 |
| `figure_filter_concurrency` | 4 | WP-4 |
| `max_figure_vlm_per_page` | e.g. unlimited / 12 | WP-4 |
| `layout_onnx` | false | WP-6 |
| `layout_onnx_model_path` | — | Pinned path |
| `layout_onnx_model_sha256` | — | Integrity |
| `layout_conf` | 0.25–0.4 | Detector |
| `layout_iou` | 0.3 | Match to L1 |
| `layout_imgsz` | 1024 | Match export |

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Area gate drops small real charts | Corpus lock; start low (`0.008`); raise only with G6 Δ |
| VLM Pass-1 mislabels rare diagrams | Fail-open for large Form-backed crops; log kind distribution |
| ONNX binary / EP portability | Feature-flag; CPU default; document ort link strategy |
| Pdfium thread safety | Keep Pdfium on `spawn_blocking`; no ONNX holding page locks |
| Cost regression if filter always on | Geometry first; concurrency; per-page VLM cap |
| License of layout weights | Verify before bundling; prefer self-hosted pinned file |
| Layout coordinate bugs | Golden tests: synthetic page with known figure rect |

---

## Explicit non-goals

- Replacing pdfium-render as primary render/StructTree engine in this plan
- English caption regex as **primary** region detector
- Raising `MAX_AREA_FRAC` to chase recall
- Unpinned runtime download of layout weights in production
- Full substitution of EdgeQuake conversion by external Python parsers (offline A/B only)
- Treating unlabeled crop count as success

---

## File-level checklist

### edgequake-pdf2md
- [ ] `geometry.rs` — `MIN_IMAGE_AREA_FRAC`, `MAX_ASPECT`, helpers
- [ ] `object_cluster.rs` — image branch uses new gates
- [ ] `struct_tree.rs` — align `l0_area_ok`
- [ ] `extract_images.rs` — page-area + aspect after bbox
- [ ] `layout_page.rs` — **new** (`layout-onnx` feature)
- [ ] `visual/mod.rs` — optional L2 refine hook
- [ ] `Cargo.toml` — optional `ort`

### edgequake-pdf
- [ ] `backend/vision.rs` — prune `figure_map`; routing; telemetry; layout hook
- [ ] `figure_filter.rs` — concurrent run; new kinds
- [ ] `vision_prompts.rs` — Pass-1 taxonomy
- [ ] config defaults / env
- [ ] tests: G-prune, industrial fixture

### specs
- [ ] Update `specs/049-improve-figure-extraction/` with L2 page layout, prune requirement, gate table
- [ ] `specs/CHANGELOG` user-facing release notes

---

## Success metrics

| Metric | Target |
|--------|--------|
| Labeled figure recall (048 corpus) | No regression vs baseline (G6) |
| Noise / unlabeled crop rate | Material drop on synthetic industrial page |
| VLM calls per dense page | Down after geometry + prune (+ layout) |
| Manifest vs `figure_map` | Identical kept set (G-prune) |
| Full-page Drawing | Still zero (G2) |

---

## Reference links

| Topic | Link |
|-------|------|
| EdgeQuake | https://github.com/raphaelmansuy/edgequake |
| pdfium-render | https://github.com/ajrcarey/pdfium-render |
| pdfium-render crate | https://crates.io/crates/pdfium-render |
| ONNX Runtime | https://onnxruntime.ai/ |
| ort (Rust) | https://docs.rs/ort |
| ort guide | https://ort.pyke.io/ |
| DocLayout-YOLO | https://github.com/opendatalab/DocLayout-YOLO |
| DocLayout-YOLO paper | https://arxiv.org/abs/2410.12628 |
| DocStructBench weights | https://huggingface.co/juliozhao/DocLayout-YOLO-DocStructBench |
| DocStructBench ONNX (community) | https://huggingface.co/wybxc/DocLayout-YOLO-DocStructBench-onnx |
| PDF-Extract-Kit | https://github.com/opendatalab/PDF-Extract-Kit |
| MinerU | https://github.com/opendatalab/MinerU |
| usls | https://github.com/jamjamjon/usls |
| OpenTelemetry GenAI | https://opentelemetry.io/docs/specs/semconv/gen-ai/ |
| ISO 32000 overview | https://pdfa.org/resource/iso-32000-2/ |
| image crate | https://docs.rs/image |

---

## Immediate next actions

1. Land **WP-0** (prune) + tests
2. Land **WP-2** constants + corpus run
3. Land **WP-1** + **WP-3**
4. Schedule **WP-6** only after precision-release metrics are green
5. Keep this issue as the tracking umbrella for SPEC-049 precision work

---

## Related in-tree paths

- `specs/049-improve-figure-extraction/001-first-principles.md`
- `specs/049-improve-figure-extraction/002-architecture.md`
- `specs/049-improve-figure-extraction/003-implementation-plan.md`
- `specs/049-improve-figure-extraction/004-acceptance-and-tests.md`
- `specs/049-improve-figure-extraction/005-non-flaky-improvement-brainstorm.md`
- `crates/edgequake-pdf/src/figure_filter.rs`
- `crates/edgequake-pdf/src/backend/vision.rs`
```

---

### How to publish

1. Open https://github.com/raphaelmansuy/edgequake/issues/new  
2. Paste the title and body above  
3. Or **reconnect GitHub** with permission to create issues on `raphaelmansuy/edgequake`, then ask again to create the issue automatically  

If you reconnect with write access, I can file this issue for you in one step.