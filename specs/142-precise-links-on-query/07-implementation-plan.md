# 07 — Implementation Plan

## Phase A — Spec pack (this directory)

Done when README + laws + lenses + matrices land.

## Phase B — Citation SSOT (query crate)

1. Add `citation_verify.rs`: `CitationEntry`, `CitationCatalog`, `rewrite_verified_citations`.
2. Rust URL builder mirroring `document-url.ts`.
3. Unit golden fixtures (U-142-01/02).
4. Export from `lib.rs`.

## Phase C — Prompt harness

1. Update `grounding_instructions()` — forbid locators; cite `[N]` only.
2. `format_chunk_meta` — emit `doc="{title}"` when `RetrievedChunk` carries display name (or API enriches before format).
3. Acc gold path unchanged.

## Phase D — Wire surfaces

1. Build catalog after `resolve_chunk_file_paths`.
2. Rewrite in `query_execute`, stream Done, chat completion/stream persist, MCP tools.
3. Always stamp `reference_id` on product sources.
4. OpenAPI notes; fix `#page=` stale copy.

## Phase E — WebUI

1. Fix `mapChunkSources` → prefer `s.document_id`.
2. Document-link renderer: Next.js push for `/documents/`.
3. Align panel `reference_id`.
4. Optional mid-stream `[N]` → chip via catalog.

## Phase F — Unfakable tests

See [08-e2e-test-matrix.md](08-e2e-test-matrix.md).

## Order

```ascii
  B (rewriter) → C (prompt) → D (wire) → E (UI) → F (e2e)
       │              │
       └─ unit gates ─┘
```

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Edge cases: [09-edge-cases.md](09-edge-cases.md)
