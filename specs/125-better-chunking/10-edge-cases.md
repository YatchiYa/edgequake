# 10 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| E1 | Parent `##` with no body before `###` | Pack with children; never orphan | heading-dense unit |
| E2 | Consecutive heading skip (`#` then `###`) | Stack truncate | unit |
| E3 | ATX inside fence | Ignore | unit |
| E4 | Unclosed fence | Rest of doc atomic | unit |
| E5 | Pipe table overflow | Row batches + header/sep repeat | unit |
| E6 | Table without separator | Repeat header; synth sep if needed | unit |
| E7 | Table in list | Prefer pipe-row atomic still | unit |
| E8 | Fenced table vs pipe | Fence wins (atomic code) | unit |
| E9 | MM `[Table Name]` | Atomic MM; split only if oversize (C-16) | existing + pack |
| E10 | Page markers | Pdf strategy; packer not used | non-goal |
| E11 | `structure_induce` FAQ `##` | Pack induced headings | unit with env |
| E12 | VLM `# Figure` + `**Type:**` | Atomic MM region | unit |
| E13 | CJK | tiktoken, not word count | unit |
| E14 | Heading > 80 chars | Prefix uses full ATX; breadcrumb cap stays extract-only | unit |
| E15 | Overlap duplicating ATX | Prefix once per continuation; last sentence of previous body after ATX | unit |
| E16 | Budget < ATX prefix tokens | Emit prefix+as much body as fits; never prefix-only if body exists | unit |
| E17 | `min_chunk_size` > `chunk_size` | Treat floor as `min(min, size)` | unit |
| E18 | Adaptive 600 still packs heading-dense fixture | 1 chunk | unit |
| E19 | Kill switch | 4 chunks | unit |
| E20 | Empty / whitespace | 0 chunks | unit |
| E21 | `##` only file | 1 remainder chunk | unit |
| E22 | CRLF | Same as LF | unit |
| E23 | Unicode / emoji headings | Intact | unit |
| E24 | YAML frontmatter | Preface packed | unit |
| E25 | Nested lists | Prose pack | unit |
| E26 | Blockquote `> ## x` | Not a heading (unquoted ATX only) | unit |
| E27 | HTML `<h2>` | Not a heading | unit |
| E28 | Setext | Not a heading v1 | unit |
| E29 | Rebuild vs future-only | Copy + no auto rebuild | Playwright |
| E30 | PDF converted MD | Pdf strategy unchanged | contract existing |

## Residual risk

Setext-heavy wikis and HTML-exported “markdown” will pack as prose (recursive inside budget). Call out in honest assessment.
