# 08 — Test Protocol

## Unit (pipeline)

```bash
cargo test -p edgequake-pipeline --lib markdown
cargo test -p edgequake-pipeline --test contract_spec125_markdown_pack
```

| Case | Expected |
|------|----------|
| Heading-dense fixture @ 600/800/1200 | 1 chunk; not heading-only first |
| Kill switch OFF | 4 chunks (legacy) |
| Continuation of oversized `###` | chunk 2+ starts with ATX path |
| Sibling split | ATX once + last sentence of previous body |
| Pipe table overflow | every piece has header + `\| ---` |
| Fence overflow | every piece reopens + closes the fence |
| ATX inside fence | not a split |
| Unclosed fence | remainder atomic |
| Empty / whitespace | 0 chunks |
| `##` only document | 1 small chunk (last remainder) |
| CRLF | same pack as LF |
| Unicode headings | pack + prefix intact |
| YAML frontmatter | packed with body |
| Nested lists | pack as prose |
| HTML `<h2>` | not a heading (passthrough) |
| Setext | not a heading v1 |
| `min_chunk_size` | no emit below floor except last/atomic undersize |
| tiktoken | `ChunkResult.tokens` == `count_tokens(content)` |

## Acc unchanged

```bash
cargo test -p edgequake-pipeline --test contract_spec026_recursive_chunking
cargo test -p edgequake-pipeline --test e2e_spec116_chunk_geometry
```

## API / e2e ingest

Ingest heading-dense markdown (`source_type=markdown`) → `chunk_count == 1` (or ≤2 if overlap artifacts). Lineage `token_count` matches tiktoken.

## Playwright

1. Open workspace settings  
2. Assert `chunking-markdown-pack-hint` visible  
3. Assert `chunking-future-only-hint` still present  

## OTEL (LAW-125-10)

InMemory exporter: `ingest.chunking` output JSON contains `token_min`, `token_p50`, `token_max`, `orphan_heading_chunks`. No chunk body.

## Honesty

Do not claim Acc density changed. Packing is Markdown-strategy only.
