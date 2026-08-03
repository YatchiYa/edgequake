# 09 — Edge Cases (SPEC-104)

Post-harden statuses: [13-fix-assessment.md](13-fix-assessment.md) v2 · [14-harden-notes.md](14-harden-notes.md).

| ID | Edge case | Status | Evidence |
|----|-----------|--------|----------|
| EC-01..04 | INV-D2 UUID parse / fail-visible | CLOSED | unit + source |
| EC-05 | Multi-ws graphs | **PARTIAL** | GIN all `eq_%_graph`; INV-C default only |
| EC-06 | Namespace sanitize | CLOSED | `PostgresConfig` helpers |
| EC-07..08 | Post-125 / with chunks | CLOSED | dual INV-03 |
| EC-09 | KV-only legacy text | OPS | dual-read clears if KV keys exist |
| EC-10 | Concurrent same slug | CLOSED | atomic RETURNING |
| EC-11 | Different name same slug | CLOSED | service `Error::Conflict` → HTTP 409 |
| EC-12 | Missing GIN | CLOSED | per-graph check |
| EC-13 | 57014 with GIN | OPS | SPEC-089 capacity |
| EC-14..15 | Admin / tutorial | CLOSED | wired + docs |
| EC-16 | KV-era false INV-03 | CLOSED | dual chunks\|KV |
| EC-17 | DO NOTHING race | CLOSED | DO UPDATE RETURNING |
| EC-18 | INV-01 silent | CLOSED | chunk_embeddings / Warning |

```ascii
 INV-03 dual presence (EC-16 CLOSED)
 indexed doc
    ├─ has public.chunks row ──────────▶ clear
    ├─ else KV table + chunk keys ─────▶ clear
    └─ else ───────────────────────────▶ INV-03 fire
```
