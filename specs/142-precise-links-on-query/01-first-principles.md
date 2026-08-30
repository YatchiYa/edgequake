# 01 — First Principles (LAW-142)

## Axioms

| ID | Law | Operational meaning |
|----|-----|---------------------|
| **LAW-142-1** | Locators are storage properties | `page_start` / document name come from chunk + document metadata KV — never from LLM tokens |
| **LAW-142-2** | Product cites only `[N]` | Handle = prompt `citation_id` / API `reference_id` |
| **LAW-142-3** | Deterministic rewriter | Valid `[N]` → `[Document Name, p.P](href)` via `CitationCatalog` |
| **LAW-142-4** | Href = SPEC-033 schema | `/documents/{docId}?chunk={chunkId}&page={page_start}`; omit `page` if absent; always `page_start` (SPEC-135) |
| **LAW-142-5** | Fail closed on unknown `[N]` | Strip; never leave a fake link |
| **LAW-142-6** | Acc gold skips rewrite | `is_gold_answer_extension` → existing strip only (SPEC-082) |
| **LAW-142-7** | Bypass / empty RAG | No citation links |
| **LAW-142-8** | Click selects page + chunk | `pdf-page-indicator[data-page=N]` + selected hierarchy row |
| **LAW-142-9** | One SSOT all surfaces | Query JSON, SSE, Chat persist, MCP use same catalog + rewrite |
| **LAW-142-10** | P0 = locator validity | Href page == stored `page_start`; claim NLI is P1 |
| **LAW-142-11** | Uncatalogued prose pages | Strip `page N` / `p.N` not in catalog allow-list; never promote to chips |
| **LAW-142-12** | Multi-doc chip stem | When rewritten cites span >1 `document_id`, visible text is `stem p.N` |
| **LAW-142-13** | Observe citation quality | Emit rewritten / stripped / prose-scrub / validity counts (not assumed) |

## P0 “verified”

A link is verified when:

1. `document_id` ∈ retrieved chunk set
2. Display name resolved from document metadata (title / file_name)
3. `page` query param == that chunk’s `page_start` (or omitted if null)
4. Hallucinated pages cannot appear in **href** or **link text**

## P0.5 prose channel

Uncatalogued numeric prose pages (`page 999`, `p.12` outside links/fences) are **stripped**.
Catalogued prose pages may remain as text but are **not** auto-linked (would invent attribution).
Non-numeric hallucinations (“the fifth folio”) remain residual until P1.

## Anti-patterns

| Anti-pattern                                 | Violates       |
| ----------------------------------------------| ----------------|
| Ask LLM to write `DocName, p.12` or URLs     | LAW-142-1      |
| Trust `[N]` without catalog membership       | LAW-142-5      |
| Put `page=` on entity SourceReference        | SPEC-047 L1/L5 |
| `target=_blank` for `/documents/`            | LAW-142-8      |
| Renumber `citation_id` after format          | SPEC-083 X-20  |
| Acc gold with rewritten markdown             | LAW-142-6      |
| Auto-link prose “page 5” because 5 ∈ catalog | LAW-142-11     |

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Prompt: [12-prompt-harness.md](12-prompt-harness.md)
