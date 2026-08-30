# Lens — Database Expert

## Verdict

**No schema migration.** Page attribution already lives in:

| Store | Fields |
|-------|--------|
| Document markdown blob | `<!-- edgequake-page:N -->` lines |
| Chunk KV / vector meta | `page_start`, `page_end` |
| Postgres `chunks` | `page_start`, `page_end` columns (SPEC-135) |

## Read paths used by SPEC-143

1. Document detail loads markdown content → FE injects anchors client-side.
2. Hierarchy / citations use API `page_start` for deeplink (unchanged).
3. Layout overlay uses `GET /documents/{id}/pages/{n}/layout` (active `n` only).

## Non-goals

- New tables / columns for sync state (ephemeral UI).
- Persisting sync toggle preference (optional P1 localStorage later).
- Re-stamping markers on read.

## Integrity checks (ops)

| Check | Expectation |
|-------|-------------|
| Marker count ≈ PDF page count | Soft; empty pages still get markers |
| Chunk `page_start` ∈ [1, num_pages] | Existing chunker contract |
| Cross-page pack | `page_end >= page_start`; deeplink start |

## Cross-refs

- SPEC-083 X-13 marker SSOT
- SPEC-135 page span
- Edge cases: [09-edge-cases.md](../09-edge-cases.md)
