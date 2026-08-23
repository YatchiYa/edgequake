# 09 — Acceptance

## Partner

- [x] A 16-page technical PDF at workspace Fixed 1200/100 is **not** ~70 extract jobs with p50 ~230.
- [x] Lineage chunks look packed (heading + figure + following prose share a chunk when they fit).
- [x] Each figure appears **once** in extract units (no duplicate `[Chart Name]` copies of inlined VLM).
- [x] Citation badge is `p.N` or `p.N–M`; click opens the **start** page.
- [x] Workspace card says packing applies to **future** ingestions; Rebuild is explicit.
- [x] Ops can roll back geometry with env (`PDF_PACK=0`) without a code revert.

## Engineering

- [x] Packer SSOT reused (`markdown_pack.rs`); Pdf does not fork a second greedy packer.
- [x] `ChunkResult.tokens == count_tokens(content)` on product Pdf path.
- [x] Comment-only HTML is never an extract unit.
- [x] Inline VLM ⇒ sidecar skipped for that asset (`U-135-MM-ONCE`).
- [x] `PDF_PACK=0` restores Recursive N (`n_legacy`) on the hashed fixture.
- [x] `CROSS_PAGE_PACK=0` keeps `page_start == page_end`.
- [x] P2 span: tiny consecutive pages → one chunk, columns `1`/`2`.
- [x] Oversize page still splits; no silent drop.
- [x] Relational `chunks.page_start` / `page_end` populated (`E2E-135-01`).
- [x] OpenAPI: `page_end` may exceed `page_start`.
- [x] `ingest.chunking` emits `fill_p50` (fail-open warn < 0.4 on ≥8k docs).
- [x] `U-135-ACC-R` green (Recursive + SPEC-116 geometry on non-PDF).

## Ops

- [x] `.env.example` documents `EDGEQUAKE_PDF_PACK` and `EDGEQUAKE_PDF_CROSS_PAGE_PACK` (default ON when unset).
- [ ] Acc PDF dual-SUT either pins `PDF_PACK=0` or re-runs medical-mid (see [12](12-honest-assessment.md)). **v0.26.0:** attested existing `publish/latest` (`valid: true`, 2026-08-15); PDF geometry not re-scored.
- [x] No auto-rebuild of existing workspaces.

## Residual (honest)

- Product PDF N is **not** byte-identical to LightRAG **F**.
- Historical rows keep NULL page columns until Rebuild KG.
- Tenant chunking still absent (SPEC-123).
- Late chunking / LLM contextual prefixes still non-goals.
- Setext / HTML `<h2>` still not headings (SPEC-125 residual).
- Acc **PDF** score may move; this spec does not claim Acc-neutral PDF geometry.
