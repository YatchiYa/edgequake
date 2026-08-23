# Lens 002 — Fullstack

## Stake

One ingest path (`source_type=pdf` / `.pdf` / page markers) must change
strategy, MM append, persistence, OpenAPI, and a small UI badge — without
forking a second packer or breaking Acc R/F text tests.

## Wiring (v1)

```ascii
  registry.rs
    Pdf => PageAwareChunking::new(Box::new(MarkdownPacking))  // default
    Pdf + PDF_PACK=0 => PageAwareChunking::default()          // Recursive

  page_aware.rs
    P1: inner.chunk(page) still hard-split if CROSS_PAGE_PACK=0
    P2: feed page-marked markdown to packer as soft units; stamp span

  chunks.rs
    enrich: skip sidecar ids already inlined
    append: omit <!-- multimodal-chunks --> if leftover list empty

  atomic_blocks.rs
    skip comment-only regions

  relational_chunk_writer.rs + domain Chunk
    bind page_start / page_end

  OpenAPI ChunkDetail
    page_end may be > page_start

  document-hierarchy-tree.tsx
    badge p.N–M; deeplink page_start
```

## DRY

- Packer math: **only** `markdown_pack.rs`.
- Token count: **only** `count_tokens` for product Pdf/MD packing.
- Page parse: existing `page_marker.rs` / `PageMarkerWriter`.
- MM persist KV: keep; only change **text append** dedupe.

## Failure modes to code for

| Mode | Behavior |
|------|----------|
| Packer panic / empty | Fail ingest with error (do not silently fall back without log) |
| `PDF_PACK=0` | Recursive inner; MM dedupe still on unless `MM_CHUNKS=0` |
| No page markers | Pdf strategy still packs; `page_*` stay NULL (non-PDF-like MD via Pdf is rare) |
| Oversize atomic | Split/keep atomic per LAW-125; never drop |

## Tests the fullstack owns

`U-135-FILL`, `U-135-PROBE`, `U-135-TIKTOKEN`, `U-135-KILL`, `U-135-ACC-R`,
`E2E-135-01`. Commands in [../08-test-protocol.md](../08-test-protocol.md).

## Cross-refs

- As-is: [../03-code-as-is.md](../03-code-as-is.md)
- Target: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
