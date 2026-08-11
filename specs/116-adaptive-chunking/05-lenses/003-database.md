# Lens — Database Expert

## Decision: metadata JSON only

No new Postgres columns / migrations. Keys live in workspace `metadata` jsonb (same as `extraction_language`).

```ascii
  workspaces.metadata
    {
      "chunking_mode": "fixed",
      "chunk_token_size": 1200,
      "chunk_overlap_token_size": 100
    }
```

## Why

- Zero migration risk for multi-tenant fleets
- Matches SPEC-096 language pattern
- Inherit = key absence (sparse)

## Ops notes

- Existing rows: absent keys → Inherit
- Rebuild KG does not need schema change
- Optional future: typed columns if analytics require SQL filters — out of scope
