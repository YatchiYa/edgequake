# 04 — Target Architecture

## Model cascade (LAW-123-2)

```ascii
  Request/Upload
       │ unset
  Workspace (vision_* for VLM; llm_* / embedding_*)
       │ unset
  Tenant defaults
       │ unset
  Env (EDGEQUAKE_* / DEFAULT_*)
       │ unset
  Compiled default
```

Vision LLM special: after workspace `vision_*`, tenant vision wins **before** falling back to workspace main LLM.

```ascii
  edgequake-core::model_resolution
    resolve_llm_choice / resolve_embedding_choice / resolve_vision_llm_choice
         ▲
         ├─ pdf_upload types (vision)
         ├─ providers/resolver (LLM request gap-fill, embedding)
         ├─ vlm_provider_resolver (workspace VLM)
         └─ FE resolve-model-choice.ts
```

## Priority cascade (PDF — LAW-123-2)

```ascii
  ┌─────────────────────────────────────────────┐
  │ Upload.pdf_parser_backend                   │  highest
  └──────────────────┬──────────────────────────┘
                     │ if unset
  ┌──────────────────▼──────────────────────────┐
  │ Workspace.pdf_parser_backend                │
  └──────────────────┬──────────────────────────┘
                     │ if unset
  ┌──────────────────▼──────────────────────────┐
  │ Tenant.pdf_parser_backend                   │
  └──────────────────┬──────────────────────────┘
                     │ if unset
  ┌──────────────────▼──────────────────────────┐
  │ EDGEQUAKE_PDF_PARSER_BACKEND                │
  └──────────────────┬──────────────────────────┘
                     │ if unset
  ┌──────────────────▼──────────────────────────┐
  │ Vision (compiled default)                   │  lowest
  └─────────────────────────────────────────────┘
```

## Choice vs runtime

```ascii
  PdfParserChoice = vision | edgeparse | auto

  resolve(...) -> ResolvedPdfParser {
    choice,           // provenance winner
    source,           // upload|workspace|tenant|env|default
    runtime_backend,  // vision|edgeparse (auto starts as vision intent)
    allows_auto_route // true iff choice == auto && AUTO_PDF_ROUTING
  }
```

## SSOT (DRY)

```ascii
  edgequake-pdf
    resolve_pdf_parser_choice(upload, workspace, tenant, env)
         ▲
         │ called by
         ├─ pdf_upload helpers / apply layers
         ├─ reprocess / recovery
         ├─ /parse backends
         └─ FE mirror: resolve-pdf-parser-backend.ts
```

## Auto-route gate (LAW-123-4)

```ascii
  BEFORE: AUTO && !explicit && Vision
  AFTER:  AUTO_PDF_ROUTING && choice == auto
```

Failure fallback to EdgeParse: only when `allows_auto_route` (never for resolved Vision).

## Tenant storage

Store `pdf_parser_backend` on tenant metadata (or first-class field mirroring workspace). Default: unset (inherit env). No forced Vision pin on tenant (workspace create still pins Vision for new workspaces per existing product rule).

## Batch / multi-file

```ascii
  Single-file admit ──┐
                      ├── same SSOT + same FormData knobs
  /pdf/batch ─────────┘

  Large admission:
    large files → optional parser override
    non-large   → keep upload/workspace resolution
```

## Lineage honesty

| Mode | Lineage |
|------|---------|
| vision | `pdf_extraction_method=vision` |
| edgeparse | `pdf_extraction_method=edgeparse` |
| auto → EdgeParse | `edgeparse` + warning/note “auto-routed” |
| auto → Vision | `vision` + note “auto fell through to Vision” |
