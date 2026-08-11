# 07 — Implementation Plan

## Phase D — Model SSOT (LLM / embedding / vision LLM)

1. Add `edgequake-core::model_resolution` (`resolve_llm_choice`, `resolve_embedding_choice`, `resolve_vision_llm_choice`).
2. Stop mutating vision into `PdfUploadOptions` via `apply_workspace`; resolve at use with tenant.
3. Inherit tenant vision onto workspace DTO only when tenant has vision defaults (env-only stays unset for UI honesty).
4. Wire query resolver request gap-fill + embedding create through SSOT.
5. Align VLM workspace resolve with vision SSOT when `llm_roles.vlm` absent.
6. FE mirror `resolve-model-choice.ts` + unit tests.
7. Keep Extract/Keyword Acc env-first unless product unlocks.

## Phase A — SSOT + law (backend)

1. Introduce `PdfParserChoice` (`vision` | `edgeparse` | `auto`) for config layers; keep `PdfParserBackend` as runtime Vision|EdgeParse.
2. Add pure `resolve_pdf_parser_choice(upload, workspace, tenant, env) -> ResolvedPdfParser`.
3. Persist tenant `pdf_parser_backend` via metadata (or field).
4. Wire apply layers: upload → workspace → tenant → env.
5. Set `allows_auto_route` / evolve `pdf_parser_backend_explicit` semantics: true for vision|edgeparse winners; auto sets `allows_auto_route=true`.
6. Gate `should_try_edgeparse_before_vision` on Auto only.
7. Gate failure fallback on Auto only.
8. Update recovery / reprocess / `/parse` / PDF batch; reject or route PDFs on `/upload/batch`.

## Phase B — FE honesty + batch

1. Mirror resolver in `resolve-pdf-parser-backend.ts` (+ tenant).
2. Settings + dropzone: Auto option; honest labels.
3. Large admission: per-file override for large only.
4. Replace path: preserve upload parser.
5. Detail: requested vs effective when Auto.

## Phase C — Tests

See [08-test-protocol.md](08-test-protocol.md).

## Edge cases (mitigate + test)

| EC | Case | Mitigation |
|----|------|------------|
| EC1 | Invalid string | Treat unset; prefer 400 on upload |
| EC2 | Env only set | Source=env; inviolable if vision/edgeparse |
| EC3 | Workspace none + env vision | Vision inviolable |
| EC4 | Workspace auto | Auto-route allowed |
| EC5 | Upload vision + workspace auto | Upload wins → Vision |
| EC6 | Tenant edgeparse + workspace none | EdgeParse |
| EC7 | Mixed large/small batch + EdgeParse confirm | Only large get override |
| EC8 | Replace duplicate | Keep upload override |
| EC9 | Recovery reconcile | Re-resolve via SSOT |
| EC10 | `AUTO_PDF_ROUTING=0` | Even Auto cannot fast-path |
| EC11 | Auto + scanned PDF | Fall through to Vision |
| EC12 | Multi-workspace concurrent | Tenant/workspace isolation |
| EC13 | Request LLM provider without model | Gap-fill from workspace/env |
| EC14 | Tenant vision + workspace llm only | Tenant vision wins |
| EC15 | No vision-embedding type | Use vision LLM + text embedding separately |

## Order of work

```ascii
  Spec pack (done)
    → Resolver + Auto gate (A)
    → FE + admission + Replace (B)
    → E2E matrix (C)
    → Model SSOT (D)
    → Acceptance
```
