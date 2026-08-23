# 07 — Implementation Plan

Spec-only step = **WP-0** (this tree). WP-1..WP-6 are product work **after** spec merge.
No product code in WP-0.

## Work packages

```ascii
  WP-0 Spec pack          ← this tree + synthetic fixtures + gold JSON
  WP-1 P0                 comment-only skip + MM inline dedupe
  WP-2 P1                 PageAware inner = packer; tiktoken; hard pages
  WP-3 P2                 soft page units + span; CROSS_PAGE_PACK kill
  WP-4 Persist            bind page_start/page_end; OpenAPI; UI span
  WP-5 Observability      fill_p50 + mm_sidecar_appended on ingest.chunking
  WP-6 Tests              U-135-* + E2E-135-01 + Playwright + Acc R
```

### WP-0 — Spec pack (this PR)

Write `specs/135-chunking/**`. Synthetic fixtures only. Do **not** vendor the live
FreeToken paper. Freeze SHA-256 of fixture bytes in [08-test-protocol.md](08-test-protocol.md).

**DoD:** document map complete; gold JSON has `n_min`/`n_max`/`fill_p50_min`/`n_legacy`/probe ids.

### WP-1 — P0 MM-once + comment skip

| File | Change |
|------|--------|
| `edgequake-api/src/services/multimodal/chunks.rs` | `enrich_processed_text_with_mm_chunks`: skip sidecar when asset already inlined; omit `<!-- multimodal-chunks -->` if leftover empty |
| `edgequake-pipeline/src/chunker/atomic_blocks.rs` | Never extract HTML-comment-only units |
| Tests | `U-135-NO-COMMENT`, `U-135-MM-ONCE` |

P0 is valuable **before** packer swap: live 70 → 61 on the trigger class.

### WP-2 — P1 packer inner

| File | Change |
|------|--------|
| `chunker/registry.rs` | `Pdf` default inner = markdown packer |
| `chunker/page_aware.rs` | `Default` uses packer unless `EDGEQUAKE_PDF_PACK=0` |
| `.env.example` | Document `EDGEQUAKE_PDF_PACK` |
| Tests | `U-135-FILL` (may still miss span-only fill), `U-135-PROBE`, `U-135-TIKTOKEN`, `U-135-KILL` |

Pages still hard-split. `page_start == page_end` still true.

### WP-3 — P2 cross-page remainder

| File | Change |
|------|--------|
| `chunker/page_aware.rs` | Soft page units; stamp `page_end ≥ page_start` |
| Module docs + equality tests | Amend “MUST NOT span”; equality only when kill=0 |
| `.env.example` | `EDGEQUAKE_PDF_CROSS_PAGE_PACK` |
| Tests | `U-135-SPAN`, `U-135-NO-SPAN-OVERSIZE`, E1–E4 blockers in [10](10-edge-cases.md) |

### WP-4 — Persist + citation

| File | Change |
|------|--------|
| `edgequake-storage/.../domain/types.rs` `Chunk` | Add `page_start`/`page_end` |
| Postgres insert bind | Columns, not JSON-only |
| `relational_chunk_writer.rs` | Copy from `ChunkResult` |
| OpenAPI `ChunkDetail` | `page_end` may exceed `page_start` |
| `edgequake-query/src/context.rs` | Same copy |
| `document-hierarchy-tree.tsx` | Badge `p.N–M`; deeplink start |
| Tests | `E2E-135-01`, `E2E-135-UI` |

### WP-5 — Observability

| File | Change |
|------|--------|
| `langfuse_meta.rs` / ingest span | `fill_p50`, `mm_sidecar_appended` |
| Warn if `fill_p50 < 0.4` on docs ≥ 8k tiktoken | Fail-open |

### WP-6 — Unfakable tests

Implement every ID in [08-test-protocol.md](08-test-protocol.md). Cannot mock the packer.
Fixture hash must match. Acc R/F text tests stay green.

## File list (product, after merge)

| File | WP |
|------|----|
| `chunker/registry.rs` | 2 |
| `chunker/page_aware.rs` | 2, 3 |
| `chunker/markdown_pack.rs` | reuse (no fork) |
| `chunker/atomic_blocks.rs` | 1 |
| `services/multimodal/chunks.rs` | 1 |
| `persistence/relational_chunk_writer.rs` | 4 |
| `storage/.../domain/types.rs` | 4 |
| postgres `insert_batch` | 4 |
| OpenAPI + `codegen-openapi-refresh` | 4 |
| `document-hierarchy-tree.tsx` | 4 |
| `workspace-chunking-card.tsx` | 4 (hint) |
| `langfuse_meta.rs` / processing span | 5 |
| `.env.example` | 2, 3 |
| `contract_spec135_*.rs` + Playwright | 6 |
| Amend SPEC-033 / SPEC-125 E10/E30 copy | 3, 4 |

## Edge-case matrix

See [10-edge-cases.md](10-edge-cases.md).

## Definition of done

See [09-acceptance.md](09-acceptance.md). Unfakable bar is [08](08-test-protocol.md).
All U-135-* + E2E-135-01 + E2E-135-UI + U-135-ACC-R green. Acc PDF honesty in
[12-honest-assessment.md](12-honest-assessment.md) published with the product PR.
