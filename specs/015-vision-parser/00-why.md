# 00 — Why (SPEC-015V)

## Symptom

Operators selecting **Vision** (workspace Document parsing or per-upload parser) cannot turn off Image / Chart / Figure extraction, and cannot override the Vision extraction system prompts for domain-specific corpora. Vision always writes figure crops, page PNGs, and chart crops; Pass A/B prompts are hardcoded.

## Evidence

| Evidence | Location |
|----------|----------|
| Vision always sets `page_drawing_assets` | `pdf_processing.rs` → `page_drawing_assets_config_for_vision` |
| No extract_* flags on config | `PageDrawingAssetsConfig` — only `emit_analyze_tags` |
| Pass A prompt constant | `vision_prompts.rs` → `RAG_PAGE_VISION_SYSTEM_PROMPT` |
| Pass B prompt constants | `multimodal/prompts.rs` |
| `process_options=i` ≠ crop gate | `document_assets.rs` — analyze tags only |
| UI: parser dropdown only | `document-parsing-step.tsx`, upload `ParserSelect` |

## Job to be done

> Configure Vision so this workspace (or this upload) extracts only the visual modalities I need, with prompts tuned to my domain — without forking the parser backend.

## Five WHYs

1. **Why is Vision expensive / noisy on text-heavy PDFs?** Asset extractors always run for Vision uploads.  
2. **Why can’t operators disable Charts or Figures?** Product never modeled modality extract flags — only backend choice and `i/t/e` analyze flags.  
3. **Why do domain Acc regressions need code changes?** Pass A/B system prompts are compile-time constants with no workspace/upload override.  
4. **Why does `process_options=i` confuse users?** It gates VLM analyze tags, not crop extraction — incomplete mental model.  
5. **Root cause:** Missing workspace- and upload-scoped **Vision extract policy** (bools + prompt overrides) threaded through resolve → convert → analyze.

## Causal ASCII

```ascii
              Always-on fig/page/chart writers (Vision)
                              +
              Hardcoded Pass A / Pass B system prompts
                              +
         UI = parser dropdown only (no modality toggles)
                              +
              process_options=i ≠ extract gate
                              │
                              ▼
         Cost/latency + weak domain Acc + no operator control
```

## Success criteria

- Workspace + upload can set Images/Charts/Figures On/Off (default On).  
- Workspace + upload can override page/image/chart/figure system prompts (empty = SSOT).  
- Edge cases EC-015V-1…12 mitigated and tested.  
- DRY resolve helper; SOLID builder-boundary injection only.
