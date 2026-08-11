# 03 — Code As-Is → Target (models)

## PDF (post-SPEC-123)

```ascii
  resolve_pdf_parser_choice(upload, workspace, tenant, env)
       → ResolvedPdfParser { choice, source, runtime, allows_auto_route }
  apply_workspace: no-op for vision/parser mutation
```

## Model SSOT (new)

```ascii
  edgequake-core/src/model_resolution.rs
       ├─ resolve_llm_choice
       ├─ resolve_embedding_choice
       └─ resolve_vision_llm_choice
              ▲
              ├─ PdfUploadOptions::resolved_vision_llm(ws, tenant)
              ├─ resolve_inherited_model_fields (tenant vision fill)
              ├─ WorkspaceProviderResolver (request gap-fill + embedding)
              └─ FE: resolve-model-choice.ts
```

## Historical PDF bug (kept for repro)

```ascii
  Workspace UI: Server Default (Vision)  ⇒ pdf_parser_backend = None
  Upload: Workspace Default (Vision)     ⇒ form field omitted
       → resolved=Vision, explicit=false → EdgeParse lineage  (pre-fix)
```

## Critical snippet semantics (legacy PDF)

| File | Behavior |
|------|----------|
| `pdf_upload/types.rs` | `resolved_vision_llm` / `resolved_backend` via SSOT |
| `large_document_profile.rs` | Auto-only gate for EdgeParse fast path |
| `resolve-pdf-parser-backend.ts` | FE mirror + honest labels |
| `model_resolution.rs` | LLM / embedding / vision LLM cascade |
