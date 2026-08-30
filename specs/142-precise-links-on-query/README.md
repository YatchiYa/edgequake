# SPEC-142 — Precise verified links on Query

> **Mission:** When users ask EdgeQuake in Query Mode (and all answer surfaces),
> replies contain **precise, verified** links that include the **document name**
> and **page number**. Clicking a link opens that document and selects that page —
> without hallucinated page numbers.
>
> **Method:** LLM emits only `[N]`; a deterministic rewriter attaches locators
> from retrieval metadata. Acc gold stays citation-free.
>
> **Target cut:** next patch after 0.26.3.

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  LAW: Page + document name come from chunk storage — never from LLM tokens. │
│                                                                              │
│  Product path:                                                               │
│    LLM emits [N]  →  rewrite_verified_citations(catalog)                     │
│                   →  [DocName, p.P](/documents/{id}?chunk=&page=P)           │
│                                                                              │
│  Unknown [N] → stripped. Acc gold → no rewrite (SPEC-082).                   │
│  Bypass / empty RAG → no citation links.                                     │
│                                                                              │
│  P0 verified = locator validity (href page == stored page_start).            │
│  P1 claim-level NLI = out of v1.                                             │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Document map

```ascii
  README
    → 00-why (5 WHY)
    → 01-first-principles (LAW-142-1..10)
    → 02-cross-ref-matrix
    → 03-code-as-is
    → 04-target-architecture
    → 05-lenses/ (PO, fullstack, DB, UX, front, AI, prompt)
    → 06-ux-ui-spec
    → 07-implementation-plan
    → 08-e2e-test-matrix
    → 09-edge-cases
    → 10-acceptance
    → 11-honest-assessment
    → 12-prompt-harness
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| I1 | CitationCatalog + rewriter (query crate) | Done |
| I2 | Prompt harness (grounding + doc=title) | Done |
| I3 | Wire sync / stream / chat / MCP | Done |
| I4 | WebUI mapper + inline links + Next.js nav | Done |
| T1 | Unfakable Rust / HTTP / Playwright / MCP | Done |
| A1 | Acceptance | Done (checklist in 10-acceptance) |

## Locked decisions

| Decision | Choice |
|----------|--------|
| Citation handle | `[N]` matching `citation_id` only |
| Locator attach | Deterministic rewriter (not LLM) |
| Href schema | SPEC-033 `/documents/{id}?chunk=&page=` |
| Cross-page span | Badge may show `p.N–M`; href always `page_start` |
| Acc gold | Skip rewrite; keep strip |
| NLI faithfulness | P1 / out of v1 |

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-033](../033-page-lineage/) | Deeplink + controlled PDF viewer |
| [SPEC-047 L-B1](../047-rag-evaluation/021-lineage-first-principles-query.md) | Answer-inline entity→chunk→page |
| [SPEC-082](../001-benchmark/001-edgquake-improvements/082-gold-citation-compat.md) | Acc gold citation freeze |
| [SPEC-135](../135-chunking/) | `page_end` badge vs deeplink start |
| [SPEC-028](../028-edgequake-query-service/) | Query DTO / MCP |
| [SPEC-083 X-20](../083-improvements/) | Stable `citation_id` |
