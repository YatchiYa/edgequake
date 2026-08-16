# Lens 003 — Database Expert

## Storage

Chunks remain KV + vector rows. **No schema migration.** Packed content is a new `content` string; `token_count` stays on the JSON value.

```ascii
  documents.chunk_count     ← smaller N on heading-dense MD after rebuild
  chunk KV.content          ← may include repeated ATX / table header
  chunk KV.section.heading_path  ← unchanged metadata
  chunk_embeddings          ← must rebuild; old vectors are heading-orphan
```

## Invariants

- Offsets still document-relative when preserved; continuation prefixes are **synthetic** and must not claim false source spans for the prefix bytes. Prefer: offsets cover the **body** span; prefix is extra stored text.
- Rebuild Knowledge Graph re-chunks; no in-place rewrite of old chunk rows without rebuild.
- Workspace isolation unchanged.

## Non-goal

Typed SQL columns for packing flags. Env + strategy is enough.

## Cross-refs

- LAW-125-9 future-only
- SPEC-024 chunk KV SSOT
