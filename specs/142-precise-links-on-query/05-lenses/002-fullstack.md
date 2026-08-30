# Lens 002 — Full Stack Developer

## Stake

Citation must be one contract from retrieval → prompt → rewrite → DTO → UI →
viewer. Divergent paths (panel vs answer vs MCP) reintroduce hallucination.

## Implementation map

| Layer | Change |
|-------|--------|
| Query crate | `citation_verify.rs` + catalog from chunks |
| Prompt | `grounding.rs` + `context_format` title |
| API | Rewrite in execute / stream / chat / MCP |
| WebUI | Mapper fix; Next.js document links; chips |
| Tests | Mock LLM + fixture page — no live model |

## DRY / SOLID

- One rewriter; handlers only call it.
- Catalog built once per response; sources and rewrite share entries.
- Acc path branches on gold flag only — no second rewriter.

## Pitfalls

- Stream token split `[` / `1]` → mid-stream client rewrite from catalog; Done replaces with server text.
- `include_references` false omitting `reference_id` → stamp always on product path.
- Mapper ignoring `document_id` → wrong `/documents/{chunkUuid}`.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
