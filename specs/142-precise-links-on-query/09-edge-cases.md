# 09 — Edge Cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | Hallucinated `[99]` | Strip | U-142-01 |
| EC-02 | Compound `[1,2]` / `[1][2]` | Expand each id | U-142-01 |
| EC-03 | Fenced code `[1]` | Skip rewrite inside fences | U-142-02 |
| EC-04 | Missing title | `file_name` then short id | unit |
| EC-05 | Missing `page_start` | Name link; omit `?page=` | PW-142-03 |
| EC-06 | Span 3–4 | Text `p.3–4`; href `page=3` | U-142-01 |
| EC-07 | Injection sources | Excluded from catalog | existing + unit |
| EC-08 | Deleted document | Honest href; viewer 404 | manual / HTTP |
| EC-09 | Workspace isolation | Catalog from scoped retrieval only | existing scope |
| EC-10 | XSS in title | Markdown-escape `]` / specials | unit |
| EC-11 | LightRAG `### References` | Gold strip / ignore invented titles | Acc + unit |
| EC-12 | `target=_blank` regression | Document links use client nav | PW-142-01 |
| EC-13 | Stream split `[` `1]` | Catalog chips mid-stream; Done replaces | HTTP-142-02 |
| EC-14 | Dual-list Mix-only in panel | Rewrite only ids in prompt catalog | unit |
| EC-15 | Entity sources no pages | No entity page in v1 | SPEC-047 |
| EC-16 | `include_references` false | Still stamp on product path | HTTP |
| EC-17 | Pre-032 NULL pages | Omit page; still name link | PW-142-03 |
| EC-18 | Acc gold | Skip rewrite | U-142-02 / Acc |
| EC-19 | Prose `page 999` / `p.12` not in catalog | Strip phrase (LAW-142-11); keep catalogued prose; never auto-link | U-142-01 |
| EC-20 | Multi-doc rewritten cites | Chip text `stem p.N` (LAW-142-12) | U-142-12 |

## Cross-refs

- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
- Laws: [01-first-principles.md](01-first-principles.md)
