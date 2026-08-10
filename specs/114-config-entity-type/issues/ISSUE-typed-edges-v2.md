# ISSUE — Typed edges (SPEC-114b) + canvas (later)

**Status:** Wave A (114b) — **implementing / shipped with compact editor**; canvas still deferred  
**Wave:** W7 → split into **W7a (114b code)** + **W7b (React Flow canvas, docs-only until approval)**  
**Depends on:** W1–W6 (v1 dual allowlists) complete

## Goal

Allow optional typed edge constraints and edit/add/delete associations by entity type:

```ascii
PERSON ──WORKS_AT──► ORGANIZATION
MACHINE ──HAS_DEFECT──► DEFECT
```

## Metadata (114b — implemented)

```json
{
  "relation_edges": [
    { "source": "PERSON", "relation": "WORKS_AT", "target": "ORGANIZATION" }
  ]
}
```

- Cap: 100 edges (separate from type max-50).
- Normalize: UPPER_SNAKE; dedupe; drop edges referencing unknown entity/relation when allow-lists present.
- Empty/absent ⇒ unconstrained endpoints (LAW-114-11).

## Pipeline (114b)

- When `relation_edges` non-empty: prompt lists allowed patterns; `enforce_relation_edge` after label enforce.
- Strict: remap relation to a matching `(src,*,tgt)` edge or `RELATED_TO` / first listed.
- Permissive: passthrough when endpoints do not match.

## UX (114b)

- Compact `TypedEdgeEditor` in Extraction step (source ▾ · relation ▾ · target ▾).
- Entity lens filter (All | PERSON | …).
- Honest `KgSchemaPreview` — real edges only (no modulo pairing).
- Domain presets load entities + relations + curated edges.

## Canvas (later — W7b)

- Expand → React Flow editor; sync back to same `relation_edges` SSOT.
- Progressive disclosure: “Open schema editor”.
- Exit criteria: product approval for canvas dependency size.

## Non-goals until separate approval

- OWL/RDF export  
- SHACL  
- Schema versioning table  
