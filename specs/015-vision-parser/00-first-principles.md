# 00 — First Principles (SPEC-015V)

## Axioms

1. **Vision extract policy is a first-class config** — peer to parser backend, not a hidden env flag.  
2. **Absent bools mean ON** — backward compatible with today’s always-on Vision asset path.  
3. **Empty prompt means SSOT** — never ship an empty system message to the VLM.  
4. **Upload wins per field** — explicit multipart overrides workspace; omit inherits.  
5. **Gates are causal** — a disabled modality must not write assets, inject markdown, or run Pass B for that modality.  
6. **Prompts inject at builder boundaries only** — do not fork pdf2md or duplicate prompt files.  
7. **Evidence beats vibes** — every finding maps to a gate.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-015V-1** | Extract SSOT — `VisionExtractConfig` owns images/charts/figures bools + four optional system prompts. |
| **LAW-015V-2** | Resolve order — upload explicit → workspace metadata → built-in defaults (bools true; prompts None). |
| **LAW-015V-3** | Vision-only — EdgeParse ignores extract flags and prompt overrides. |
| **LAW-015V-4** | Crop gate ≠ analyze-only — extract flags gate writers; `process_options=i` still required for analyze tags, AND extract_images must be true for image analyze. |
| **LAW-015V-5** | Prompt replace — non-empty override replaces that modality’s system prompt; max 32 KiB. |
| **LAW-015V-6** | Ingest snapshot — resolved config persisted on document metadata for audit/reprocess honesty. |
| **LAW-015V-7** | Future-only apply — workspace PUT does not rewrite completed docs; reprocess is explicit. |
| **LAW-015V-8** | CI is proof — every F-015V-* has unit, Playwright, or Rust e2e gate. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | One `VisionExtractConfig::resolve`; one `VisionExtractControls` UI; prompt SSOT remains in `vision_prompts.rs` / `multimodal/prompts.rs`. |
| **SRP** | Resolve ≠ persist ≠ convert gate ≠ Pass B messages ≠ UI. |
| **OCP** | New modality flag = extend config + gate; consumers use resolve. |
| **LSP** | Memory + Postgres workspace services share same request/response shape. |
| **ISP** | Upload may send only dirty fields; workspace update is sparse. |
| **DIP** | Pipeline depends on `VisionExtractConfig` / `PageDrawingAssetsConfig`, not HTTP DTOs. |

## Inheritance (do not break)

| Prior | Constraint |
|-------|------------|
| SPEC-047 | Fig/chart/page asset identity and markdown assembly remain correct when flags ON |
| SPEC-049 | Figure filter still optional via provider injection |
| SPEC-038 | Parser upload dropdown + large-PDF admission unchanged |
| SPEC-101 | Document parsing wizard step composition preserved |
| SPEC-096/114 | Metadata JSONB pattern; no new SQL columns |
