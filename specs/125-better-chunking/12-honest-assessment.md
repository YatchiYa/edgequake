# 12 — Honest Assessment

## What this spec will fix

Heading-dense markdown: notes and wikis with many `###`, parent headings with no body. Packing + ATX continuation + table header repeat.

## What it will not fix

| Residual | Why |
|----------|-----|
| PDF page-aware recursive | Different strategy; headings in vision MD still page-split |
| Setext / HTML headings | Parse cost vs rarity; documented |
| Acc Recursive word-count tokens | Publication invariant |
| Tenant cascade | SPEC-123 gap; packing does not need it |
| Already-ingested docs | Future-only; must rebuild |
| LLM-quality context sentences | Anthropic Haiku prefixes not v1 |

## Langfuse honesty

This project’s Jul–Aug 2026 traces had **one** `ingest.chunking` (PDF adaptive 600 → 112 chunks) and **no markdown** ingest spans. Support cannot confirm heading-dense packing from Langfuse until distribution keys land **and** a markdown ingest is exported.

## Risk

Packing three `###` topics into one chunk can dilute a surgical query. For notes under the budget this is intended. For long manuals the token budget still splits. Kill switch exists.

## Success bar

Heading-dense fixture green + Acc recursive green + hint in UI + OTEL distribution + boundary overlap (ATX once, last sentence) + fence re-open. Anything less is incomplete.
