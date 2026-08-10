# SPEC-015V — Vision Parser Extract Toggles + Prompt Overrides

> **Product pin**: EdgeQuake v0.24.3+  
> **Status**: Implemented (W0–W5)  
> **Folder**: `specs/015-vision-parser/` (peer of `015-java-sdk-maven-central`; Spec ID **SPEC-015V**)

> **Inherits**: SPEC-047 modality/crops · SPEC-049 figure filter · SPEC-038 parser upload · SPEC-101 Document parsing wizard  
> **Peers**: SPEC-096 / SPEC-114 metadata override pattern · SPEC-004 query `system_prompt` (ingest counterpart)

## Start here

1. [00-why.md](00-why.md) — Five WHYs + causal ASCII  
2. [00-first-principles.md](00-first-principles.md) — LAW-015V-1…N + SOLID/DRY  
3. [01-finding-register.md](01-finding-register.md) — F-015V-*  
4. [02-cross-ref-matrix.md](02-cross-ref-matrix.md) — code ↔ law ↔ test  
5. [03-implementation-roadmap.md](03-implementation-roadmap.md) — Waves W0–W6 + DoD  
6. [04-e2e-test-matrix.md](04-e2e-test-matrix.md) — gates  
7. [05-edge-cases.md](05-edge-cases.md) — EC-015V-*  
8. Issues → [`issues/`](issues/)  
9. Lenses → [`lenses/`](lenses/)

## One-screen verdict

```ascii
+------------------------------------------------------------------+
|  PROBLEM: Vision always extracts Images/Charts/Figures           |
|  - No workspace or upload On/Off for visual modalities           |
|  - Pass A/B prompts are compile-time constants only              |
|  - process_options=i gates ANALYZE tags, NOT crop extraction     |
+------------------------------------------------------------------+
|  SOLUTION:                                                       |
|  - vision_extract_{images,charts,figures} (default true)         |
|  - vision_{page,image,chart,figure}_system_prompt overrides      |
|  - Resolve: upload > workspace metadata > built-in SSOT          |
|  - UI: Document parsing wizard + upload dropzone (Vision only)   |
+------------------------------------------------------------------+
|  SSOT: workspaces.metadata JSONB (+ doc ingest snapshot)         |
|  APPLY: future ingestions; reprocess for existing docs           |
+------------------------------------------------------------------+
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Spec ID | **SPEC-015V** |
| Images | Page PNGs + Pass-B image/drawing modality |
| Figures | Embedded + caption figure crops + Pass-B figure |
| Charts | Chart ink crops (+ fig-as-chart) + Pass-B chart |
| Defaults | All extract flags **ON**; empty prompt → SSOT |
| Prompt model | Non-empty string **replaces** that modality’s system prompt |
| Persistence | `workspaces.metadata` JSONB (no new SQL columns) |
| Inheritance | Upload explicit → workspace → built-in |
| Parser scope | Vision only; EdgeParse ignores |

## Metadata contract

| Key | Semantics |
|-----|-----------|
| `vision_extract_images` | bool; absent → true |
| `vision_extract_charts` | bool; absent → true |
| `vision_extract_figures` | bool; absent → true |
| `vision_page_system_prompt` | Pass A page OCR; absent/`""` → SSOT |
| `vision_image_system_prompt` | Pass B image; absent/`""` → SSOT |
| `vision_chart_system_prompt` | Pass B chart; absent/`""` → SSOT |
| `vision_figure_system_prompt` | Pass B figure; absent/`""` → SSOT |

## Target composition

```ascii
Document parsing / Upload (Vision)
        │
        ├─► extract_images|charts|figures
        └─► page|image|chart|figure system_prompt
                │
                ▼
        VisionExtractConfig::resolve(upload, workspace)
                │
                ├─► PageDrawingAssetsConfig (gates + Pass A prompt)
                └─► multimodal analyze (Pass B prompts + modality_enabled)
```

## Cross-spec anchors

- [SPEC-047 RAG evaluation / modality](../047-rag-evaluation/)
- [SPEC-049 figure extraction](../049-improve-figure-extraction/)
- [SPEC-038 large PDF / parser upload](../038-ingestion-large-pdf/)
- [SPEC-096 multi-language](../096-multi-language-extraction/)
- [SPEC-114 KG schema](../114-config-entity-type/)
