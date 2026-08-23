# 08 — Test Protocol

Unfakable means: **cannot** pass by mocking the packer, asserting `chunk_count > 0`,
or swapping the fixture. Tests load **committed bytes** and check the SHA-256
below before asserting gold.

## Fixture hashes (SHA-256 of file bytes)

| File | SHA-256 |
|------|---------|
| `fixtures/freetoken_like.md` | `0f3b59fffe97a005c5d063075845699e1c42eda8d92fa7cab78efcd580c33be5` |
| `fixtures/span_tiny.md` | `6c35a71bf672ce91f26b2bbfb04ba46958555b7cc6d7885be445cdd1605d1f44` |
| `fixtures/oversize_page.md` | `0e840925e3134fb10e2149bb7ee976f9b920e2e2f7dfad602258559d06ba1c72` |
| `fixtures/mm_once.md` | `322742ae94d56a3c0d712c40b5a9b05146472fca31c3a1366190aece29f89a1c` |
| `fixtures/h1_block_span.md` | `90ac09bc62c76c5f7e7e4a6e83d5077fc00c2f231fcc97fe9cbcde8c2f907a8f` |

Gold: `fixtures/freetoken_like.gold.json` (`n_min`/`n_max`/`fill_p50_min`/probe ids).

Recompute:

```bash
shasum -a 256 specs/135-chunking/fixtures/*.md
```

Mismatch ⇒ fail the suite (do not “update expected” inside the test).

## Unit / contract (pipeline + api)

```bash
cargo test -p edgequake-pipeline --test contract_spec135_pdf_pack
cargo test -p edgequake-api --test contract_spec135_mm_once
```

| ID | Gate | Why unfakable |
|----|------|----------------|
| `U-135-FILL` | `freetoken_like.md`, Fixed 1200/100, Pdf: `token_p50 / 1200 ≥ fill_p50_min` **and** `n_min ≤ N ≤ n_max` from gold | Fill, not just fewer chunks |
| `U-135-PROBE` | `PROBE_FIG_A` and `PROBE_PROSE_A` appear in the **same** `ChunkResult.content` | Pack-with-neighbor, not emit-atomic |
| `U-135-NO-COMMENT` | Zero chunks whose trim equals `<!-- multimodal-chunks -->` or a lone `<!-- edgequake-page:N -->` | Control plane not extract |
| `U-135-MM-ONCE` | `mm_once.md`: at most **one** chunk contains `[Chart Name]cost_capability_synthetic_a` | Dedupe (inline XOR sidecar) |
| `U-135-TIKTOKEN` | Every `ChunkResult.tokens == count_tokens(content)` on Pdf path | Honest budget |
| `U-135-SPAN` | `span_tiny.md` → **1** chunk, `page_start=1`, `page_end=2` | P2 |
| `U-135-NO-SPAN-OVERSIZE` | `oversize_page.md` (single page) still **splits**; no silent drop; all sentences retained | Safety |
| `U-135-KILL` | `EDGEQUAKE_PDF_PACK=0` on `freetoken_like.md` → frozen gold `n_legacy` (20) and at least one chunk with `token_count != count_tokens` (Recursive word-count). Packed N is 30; do not require `N > n_max` on this fixture. | Rollback |
| `U-135-ACC-R` | Existing Acc text path green | Non-PDF geometry |

Negative sibling (E1): `h1_block_span.md` under P2 → **2** chunks, no span across the new H1.

## Acc unchanged (non-PDF)

```bash
cargo test -p edgequake-pipeline --test contract_spec026_recursive_chunking
cargo test -p edgequake-pipeline --test e2e_spec116_chunk_geometry
```

These two **are** `U-135-ACC-R`. Do not weaken them.

## E2E ingest (Postgres)

```bash
# Requires DATABASE_URL (make backend / make test-e2e as used in-repo)
cargo test -p edgequake-api --test e2e_spec135_pdf_pack -- --ignored
```

| ID | Gate |
|----|------|
| `E2E-135-01` | API ingest `source_type=pdf` (or `.pdf`) with page-marked MD from `freetoken_like.md` (or a truncated span fixture), mock extract. `SELECT page_start, page_end, count(*) FROM chunks WHERE document_id = $1 GROUP BY 1, 2` matches gold spans. **Zero** rows with `page_start IS NULL` when markers were present. |

## Playwright

```bash
cd edgequake_webui && pnpm exec playwright test e2e/spec135-chunk-span.spec.ts
```

| ID | Gate |
|----|------|
| `E2E-135-UI` | Document with a span chunk shows badge text `p.1–2` (`data-testid=chunk-page-badge`). Click / href uses `#page=1` (start), not page 2. Workspace card shows `chunking-pdf-pack-hint` and `chunking-future-only-hint`. |

## OTEL / Langfuse (LAW-135-10)

InMemory exporter after Pdf ingest of `freetoken_like.md`:

`ingest.chunking` output JSON contains `chunks`, `token_min`, `token_p50`, `token_max`,
`fill_p50`, `orphan_heading_chunks`, `mm_sidecar_appended`. **No** chunk body.

## Honesty

Do not claim Acc PDF geometry is unchanged. Do not satisfy `U-135-FILL` with
`chunk_count < 70` alone. Do not vendor the live FreeToken paper as a fixture.
