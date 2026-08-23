# 12 — Honest Assessment

## What this spec will fix

PDF ingest **underfill**: page-hard Recursive word-count + atomic-as-emit +
MM double-index + NULL page columns. After 135, product PDF packs to the
workspace tiktoken budget, indexes each figure once, and stores page spans
on the row the citation API can read.

## What it will not fix

| Residual | Why |
|----------|-----|
| Acc Recursive word-count on non-PDF | Publication invariant (`U-135-ACC-R`) |
| Acc TokenBased **F** on non-PDF | Unchanged |
| Byte-identical LightRAG **F** on PDF | Structure-aware pack ≠ sliding window |
| Already-ingested docs | Future-only (LAW-135-11); Rebuild KG explicit |
| Historical NULL `page_start` columns | No backfill v1 |
| Tenant cascade | SPEC-123 gap |
| LLM contextual prefixes | Anthropic Haiku “situate this chunk” stays opt-in `EDGEQUAKE_CONTEXTUAL_CHUNK` |
| Late chunking | Jina full-doc encode — non-goal |
| Setext / HTML headings | SPEC-125 residual |
| Pass-A OCR / VLM quality | Different specs (128/134) |

## Acc score may move

Wizard **Match LightRAG** today pins **size** (1200/100), not LightRAG **F**.
After 135, product PDF **N** moves toward F *fill* while remaining
structure-aware.

```ascii
  Dual-SUT Acc that ingests PDFs
       │
       ├─ pin EDGEQUAKE_PDF_PACK=0   → freeze pre-135 PDF geometry
       └─ re-run medical-mid         → publish new PDF Acc
```

This spec **will not** claim Acc-neutral PDF geometry. Publication notes must
pick one of the two bullets. Smoke n=40 is not the release Acc score
(release-and-cd SPEC-001).

SPEC-033 amendment: `page_end` **may** exceed `page_start`. Tests and OpenAPI
that require equality are **wrong** after P2 (unless `CROSS_PAGE_PACK=0`).

## Langfuse honesty

Until WP-5, `ingest.chunking` will not show `fill_p50`. Support cannot prove
underfill from traces alone. After WP-5, `fill_p50 < 0.4` on ≥8k docs is a
**warning**, not an ingest abort.

## Risk

- Cross-page pack can join two weak sections (mitigate: H1 / script / kill).
- Packing figure+methods into one chunk can dilute a figure-only query
  (mitigate: MM-once still keeps one figure unit; budget still splits long pages).
- Operators with `PDF_PACK=0` keep the 70-chunk class bug — document it.

## Success bar

All of [08-test-protocol.md](08-test-protocol.md) green, including
`U-135-FILL` closed N range + `fill_p50`, `U-135-PROBE` same-chunk,
`U-135-MM-ONCE`, `U-135-SPAN` columns, `E2E-135-01` Postgres, `U-135-ACC-R`.
Anything less (e.g. “chunk_count > 0”) is incomplete.
