# SPEC-123 — Env / Config Priority (PDF Parser + Model Law)

> **Mission:** Make configuration inviolable: **Upload → Workspace → Tenant → Env → Default**. Kill silent SPEC-038 EdgeParse rewrite when the UI says “Resolves to Vision”. Convert auto-route into an explicit `auto` mode. Extend the same priority law to **LLM**, **embedding**, and **vision LLM** (not a separate “vision embedding”). Close sibling priority leaks on batch / replace / recovery / parse / query.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Batch upload of born-digital PDF uses EdgeParse while workspace UI shows “Server Default (Vision)” / “Resolves to Vision” |
| Classification | **Config-law violation + honesty gap** — not a missing FormData field on multi-file upload |
| Root cause | `workspace.pdf_parser_backend = None` → resolved Vision + `explicit=false` → SPEC-038 auto-routes to EdgeParse |
| Model sibling | Vision LLM previously mutated workspace into upload; tenant vision skipped full cascade; LLM/embedding lacked shared provenance SSOT |
| Fix posture | Resolved value is law; Auto is explicit; one SSOT resolver per domain; e2e gates |

```ascii
  Upload > Workspace > Tenant > Env > Default
       │
       ├─ PDF: vision | edgeparse  → RUN THAT (inviolable)
       ├─ PDF: auto                → SPEC-038 may try EdgeParse then Vision
       ├─ LLM / embedding          → resolve_*_choice (request → … → env)
       └─ Vision LLM (VLM)         → upload → ws vision → tenant vision → ws llm → env
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-123-1..7 + model domains)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, marketing)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance (honest: PDF Done / Models Done / Partner Open)
   → 10-reproduction
   → 11-honest-assessment
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack (PDF) | Done |
| D2 | Doc pack (models + LAW-123-8) | Done |
| I1 | SSOT PDF resolver + Tenant + Auto | Done |
| I2 | Kill silent auto-route | Done |
| I3 | FE honesty + per-file admission + Replace | Done |
| I4 | SSOT model resolvers + wire upload/query/chat/MCP/VLM/pipeline + FE Resolves-to | **Done** |
| T1 | API + WebUI PDF matrix | Done (Playwright thin) |
| T2 | Unit model priority | Done |
| T3 | Model HTTP / lineage e2e (GET provenance + upload) | **Done** (8/8) |
| A1 | Partner / operator acceptance | **Open** — see [10-reproduction.md](10-reproduction.md) + [11-honest-assessment.md](11-honest-assessment.md) |

## Related

- [SPEC-038](../038-ingestion-large-pdf/) — large PDF / auto-routing (re-scoped: Auto must be explicit)
- [SPEC-015](../015-vision-parser/) — Vision extract overlays
- [SPEC-014](../014-multi/) — PDF `/batch`
- [SPEC-101](../101-wizard-mode-tenant-workspace/) — never-silent server default labels
- [SPEC-109](../109-configurable-reasoning-effort/) — request→…→tenant→env pattern
- [SPEC-122](../122-implementation/) — bulk throughput (orthogonal; same multi-file UX surface)
- [issue-231](../013-fix-issues-05-2026/issue-231/) — batch forgot workspace context
- Ops: [`mission/03-pdf-parser.md`](../../mission/03-pdf-parser.md)

## Non-goals (v1)

- Vision extract quality / prompts (SPEC-015V)
- Bulk throughput clamps (SPEC-122)
- Inventing a “vision embedding” model type (does not exist — VLM ≠ embedding)
- Override Acc extract/keyword **env-first** pins without an explicit product decision
