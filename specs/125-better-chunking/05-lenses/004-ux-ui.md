# Lens 004 — UX / UI

## Honesty

The workspace Chunking card today talks only about Adaptive/Fixed **size**. Partners will still think “I set 1200, why 4 chunks?”

Add one sentence:

> Markdown files pack headings together until the token budget. A heading is not its own chunk unless the next section would overflow. Applies to future ingestions — rebuild to re-chunk.

Keep the existing future-only hint (`data-testid="chunking-future-only-hint"`).

## Surfaces

| Surface | Change |
|---------|--------|
| Workspace chunking card | Description + markdown pack hint (`data-testid="chunking-markdown-pack-hint"`) |
| Wizard | Same card |
| Lineage chunk list | No new chrome required; packed content should not start with a heading-only first row on heading-dense docs |
| Settings Langfuse | No change; ingest.chunking output richer for support |

## Non-goals

- Per-document strategy picker in v1 (API already has `chunk_strategy`)
- Tenant chunking UI
