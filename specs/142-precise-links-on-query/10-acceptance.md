# 10 — Acceptance

## Product checklist

- [x] Answer links show page chips (`p.N`); full document name on hover `title` (rewriter)
- [x] Click / deeplink URL carries `?page=` from catalog (not LLM)
- [x] Hallucinated `[N]` never becomes a link (stripped)
- [x] Query, Chat, stream Done, MCP catalog share SSOT
- [x] Acc gold still citation-free

## Technical checklist

- [x] `CitationCatalog` + `rewrite_verified_citations` in query crate
- [x] Prompt forbids LLM locators; headers include doc title when known
- [x] Surfaces wired; `reference_id` stamped
- [x] Mapper prefers `document_id`
- [x] Next.js `/documents/` navigation (no `target=_blank`); citation chips
- [x] U/HTTP/PW/MCP matrix green (Rust contract + Playwright URL/href)

## Acceptance language

> “When I ask a question, the answer includes compact page chips like `p.4`
> (full name on hover). Clicking opens that document on page 4 and selects the
> cited chunk. The system never invents a page number in those links.”

## Cross-refs

- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
