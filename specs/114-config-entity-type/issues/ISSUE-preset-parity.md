# ISSUE — Preset parity

**Findings:** F-114-03, F-114-04, F-114-11  
**Wave:** W3  
**Laws:** LAW-114-4

## Goal

Domain presets ship **entity + relation** lists; General entity list matches Rust `default_entity_types()`.

## Work

1. Introduce/extend `kg-schema-presets` (or evolve `entity-presets.ts`).  
2. Align General with Rust 12-type LightRAG list (+ OTHER).  
3. Curate relation lists per domain (English tokens v1).  
4. Fix stale “matches backend” / “5 presets” comments.  
5. Unit: general ≡ known Rust list; each preset has relations.

## Suggested relation starters (v1)

| Domain | Sample relations |
|--------|------------------|
| general | RELATED_TO, PART_OF, LOCATED_IN, WORKS_AT, CREATED_BY |
| manufacturing | PART_OF, PRODUCED_BY, HAS_DEFECT, MEASURED_BY, LOCATED_IN |
| healthcare | TREATS, DIAGNOSED_WITH, ADMINISTERED_BY, LOCATED_IN, RELATED_TO |
| legal | PARTY_TO, GOVERNED_BY, CITES, REPRESENTED_BY, RELATED_TO |
| research | AUTHORED_BY, CITES, FUNDED_BY, PART_OF, RELATED_TO |
| finance | OWNED_BY, TRANSACTS_WITH, REGULATED_BY, PART_OF, RELATED_TO |
