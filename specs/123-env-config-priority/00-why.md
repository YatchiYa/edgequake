# 00 — Why SPEC-123

## Trigger

Operator batch-uploaded multiple documents (including arXiv PDF `argus_2808.05144v1.pdf`) in a workspace whose settings UI showed:

- PDF Parser: **Server Default (Vision)**
- Badge: **Resolves to Vision**
- Upload selector: **Workspace Default (Vision)**

Document detail lineage showed **EdgeParse**.

## Product WHY

```ascii
  Operator intent: “This workspace parses with Vision”
       │
       ▼
  UI truth (labels): Resolves to Vision
       │
       ▼
  Code truth (today):
       workspace.pdf_parser_backend = None
       resolved = Vision
       explicit = false
       SPEC-038 auto-route → EdgeParse for born-digital PDFs
              │
              ▼
  Trust break: what the UI promises ≠ what the pipeline runs
```

## Five WHYs

1. **Why did EdgeParse run?** SPEC-038 tried EdgeParse before Vision.
2. **Why was auto-route allowed?** `pdf_parser_backend_explicit == false`.
3. **Why was it non-explicit?** Workspace set to “Server Default” (`None`), upload omitted field.
4. **Why did the UI say Vision?** Labels print the *resolved* default (Vision), not “may auto-route”.
5. **Root cause:** Unset + Vision default was treated as permission to silently rewrite the parser — violating “what you resolve is what you run.”

## Batch is not the field-drop bug

WebUI multi-file upload issues **N×** `POST /documents/pdf` with the same resolver. Batch made the failure visible across many born-digital PDFs. Secondary batch leak: large-PDF admission can apply EdgeParse to **all** files in the drop.

## Job to be done

> When I configure Vision (or see Resolves to Vision), every upload — single or batch — must use Vision unless I explicitly choose EdgeParse or Auto.

## Success criteria

1. Priority law: Upload > Workspace > Tenant > Env > Vision.
2. Resolved Vision/EdgeParse is inviolable; Auto is the only silent-route mode.
3. UI never claims Vision when Auto can pick EdgeParse.
4. Sibling leaks V1–V8 closed or gated by tests.
5. E2E matrix makes the chain inviolable.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Repro: [10-reproduction.md](10-reproduction.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
