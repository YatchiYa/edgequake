# Lens 007 — Knowledge Graph Expert

## Dual identity is intentional

```ascii
  Graph entity / relation source_ids
       └── document_id = injection::{ws}::{id}   ← enrichment provenance

  Citation / context bundle
       └── filter document_id.starts_with("injection::")  ← never cite glossary

  Relational chunk spine
       └── document_id = injection UUID           ← FK + serving fence
```

Changing graph ids to bare UUIDs would either:

- leak injection chunks into citations, or
- require rewriting `is_injection_source` to a weaker heuristic (fragile).

## Merge / gleaning

`tag_injection_sources` must continue to stamp the **composite** id so graph cleanup and citation filters stay aligned with SPEC-0002.

## Delete parity

| Store | Key used on delete |
|-------|--------------------|
| Typed documents/chunks | injection UUID |
| Graph / vector legacy keys | composite `injection::` doc_id |

Both paths already exist; SPEC-118 only makes the typed path succeed on write.

## Acceptance for KG

- Entities created from injection remain queryable for enrichment
- Answer `sources[]` contain **no** `injection::` document ids
- Rebuild / cleanup does not orphan typed chunks under a different UUID
