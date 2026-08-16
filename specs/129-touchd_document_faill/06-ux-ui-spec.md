# 06 — UX / UI spec

## Planes

| Plane | Source | Example during slim resume |
|-------|--------|----------------------------|
| Stage chip | KV / WS `current_stage` | `re_embedding` |
| Coarse list status | `public.documents.status` | `processing` |
| Terminal success | column | `indexed` |

## Rules

1. **Display ≠ column** — FE may show rich stage; SQL stays CHECK-safe.
2. **Freshness** — after any non-terminal `update_document_status`, SQL column must leave prior `failed` when a documents row exists.
3. **No new SQL badge** for `re_embedding` in v1.
4. **Accessibility** — status text remains readable; no reliance on color alone (existing design system).

## Copy

| Stage (KV) | Suggested chip copy (existing mapper) |
|------------|----------------------------------------|
| `re_embedding` | Generating vector embeddings… (same family as embedding) |

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md), [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Mapper: `ingestion_status_mapper.rs`
