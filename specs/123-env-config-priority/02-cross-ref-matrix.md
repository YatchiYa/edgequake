# 02 — Cross-Ref Matrix

## Spec / issue map

| Spec / Issue | Relationship to SPEC-123 |
|--------------|--------------------------|
| [SPEC-038](../038-ingestion-large-pdf/) | Auto-routing + large-PDF admission; re-scope: Auto must be explicit |
| [SPEC-015 / 015V](../015-vision-parser/) | Vision extract overlays; upload > workspace (same priority spirit) |
| [SPEC-014](../014-multi/) | `/documents/pdf/batch` must share SSOT with single PDF |
| [SPEC-101](../101-wizard-mode-tenant-workspace/) | Never-silent “Server Default (X)” labels — keep honesty |
| [SPEC-109](../109-configurable-reasoning-effort/) | request → roles → workspace → **tenant** → env pattern to mirror |
| [SPEC-116](../116-adaptive-chunking/) | Doc → workspace → env (audit sibling; no god-merge) |
| [SPEC-117](../117-extraction-budget/) | Doc → workspace → env (audit sibling) |
| [SPEC-096](../096-multi-language-extraction/) | Workspace → env language |
| [SPEC-122](../122-implementation/) | Bulk throughput; same multi-file surface; orthogonal |
| [issue-231](../013-fix-issues-05-2026/issue-231/) | Batch forgot workspace — class of “second path drops context” |
| [`mission/03-pdf-parser.md`](../../mission/03-pdf-parser.md) | Historical Upload>Workspace>Env (extend with Tenant + Auto) |

## Violation register (V1–V11)

| ID | Leak | Spec law | Fix |
|----|------|----------|-----|
| V1 | Non-explicit Vision → EdgeParse auto-route | LAW-123-1,4 | Auto-only gate |
| V2 | Vision failure → EdgeParse when `!explicit` | LAW-123-1 | Fallback only for Auto |
| V3 | Large admission EdgeParse on whole batch | LAW-123-6 | Per-file override |
| V4 | Replace drops upload parser | LAW-123-6 | Pass override through |
| V5 | Recovery `explicit: false` | LAW-123-5 | Resolve via SSOT |
| V6 | `/upload/batch` no PDF knobs | LAW-123-5,6 | Reject PDF or route through PDF admit |
| V7 | `/parse` ignores workspace/tenant | LAW-123-5 | Call SSOT |
| V8 | Scattered PDF resolvers | LAW-123-5 | One Rust + FE mirror |
| V9 | `apply_workspace` mutates vision into upload | LAW-123-5 | No-op; resolve at use |
| V10 | Tenant vision skipped in PDF/VLM cascade | LAW-123-2 | `resolve_vision_llm_choice` |
| V11 | LLM/embedding without shared provenance SSOT | LAW-123-5 | `model_resolution.rs` + FE mirror |
| V12 | Inherit-paint → false `source=workspace` | LAW-123-8 | Metadata gate; stop vision paint |

## Domain priority audit (no god-resolver)

| Domain | Current chain | SPEC-123 action |
|--------|---------------|-----------------|
| PDF parser | Upload → Workspace → Tenant → Env → Vision | **Done** (+ Auto law) |
| Vision LLM | Upload → WS vision → Tenant vision → WS LLM → Env | **Done** SSOT |
| LLM | Request → Workspace (role) → Env | **Done** `resolve_llm_choice` + role after request |
| Embedding | Workspace → Env (+ inherit tenant) | **Done** `resolve_embedding_choice` |
| Reasoning effort | Request → … → Tenant → Env | Pattern already aligned |
| Chunking | Doc → Workspace → Env | Follow-up if leak found |
| Extract caps | Doc → Workspace → Env | Follow-up |
| Extraction language | Workspace → Env | Follow-up |
| Extract/Keyword Acc | Env-first pin | **Out of scope** unless product unlocks |

## ASCII dependency

```ascii
  SPEC-123 (this)
     │
     ├─ constrains SPEC-038 auto-route semantics
     ├─ extends mission/03-pdf-parser priority table
     ├─ mirrors SPEC-109 tenant layer
     ├─ shares surface with SPEC-122 multi-file UX
     └─ inherits SPEC-101 never-silent label duty
```
