# Lens 007 — Markdown Expert

## Parse rules (v1)

| Construct | Behavior |
|-----------|----------|
| ATX `#`–`######` | Soft pack boundary (not hard split) |
| ATX inside ` ``` ` / `~~~` fences | **Not** a heading |
| Unclosed fence | Treat remainder as atomic fence |
| Pipe table | Atomic until oversize → row split + header repeat |
| Table without separator | Still a table if consecutive `|` rows; synthesize sep if missing when repeating |
| Fenced table | Fence atomic (do not parse inner pipes as table) |
| Setext (`===` / `---`) | Non-goal v1 — body text |
| HTML `<h2>` | Non-goal v1 — body text |
| YAML frontmatter | Preface block; pack with following content |
| Blockquote `> ##` | Treat as ATX if CommonMark would (trimmed `>` then ATX) — prefer: heading only on unquoted lines (safer) |
| Lists | Not atomic; pack as prose |
| Skip heading levels (`#` then `###`) | Stack truncate; path is `#` + `###` |

## ATX prefix format

Emit real ATX lines (not `A → B` breadcrumbs) so markdown-aware embedders and humans in lineage see hierarchy.

Do not duplicate an ATX line that already starts the packed window.

## Continuation vs first window

- First packed window: keep original heading lines as they appear in source.
- Later windows of the **same** oversized section: prepend ancestor ATX + current heading, then body continuation (no duplicate if body already starts with that heading).

## Cross-refs

- Edges: [../10-edge-cases.md](../10-edge-cases.md)
- Atomic: SPEC-047 `atomic_blocks.rs`
