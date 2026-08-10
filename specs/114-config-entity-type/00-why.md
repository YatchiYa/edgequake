# 00 — Why (SPEC-114)

## Symptom

Admins configuring a workspace on **Extraction preferences** (wizard step 3) can pick entity types via presets/chips, but **cannot configure relation types**. The UI presents a flat chip list, not a KG schema. Domain presets feel incomplete; extracted graphs use free-form relationship labels that diverge across documents and domains.

## Evidence

| Evidence | Location |
|----------|----------|
| Entity types persist via workspace metadata | `metadata.entity_types` / `entity_types_strict` |
| No `relation_types` on Create/Update DTOs | `edgequake-core` / `edgequake-api` workspace requests |
| Prompt treats relation `type` as free-form | `json_prompts.rs` → `RELATIONSHIP_TYPE` |
| Domain presets are UI-only (entities only) | `entity-presets.ts` |
| FE General ≠ Rust `default_entity_types()` | FE includes DATE/TECHNOLOGY/PRODUCT; Rust has CREATURE/METHOD/…/OTHER |
| Observed graph `relationship_types` ≠ config | `GET /graph/labels` post-hoc stats |
| Extraction step: chips + bulk edit only | `EntityTypeSelector` + `workspace-extraction-step.tsx` |

## Job to be done

> Configure this workspace for a **predefined KG vocabulary** (entity types + relation types) with exceptional, visual UX — under two minutes via a domain preset — so future extractions produce a coherent graph.

## Five WHYs

1. **Why are graphs inconsistent across domains?** Relation labels are unconstrained; entity types alone do not define the vocabulary.  
2. **Why no relation config?** Product/API never modeled relation allow-lists; only entity types shipped (SPEC-085/#216).  
3. **Why does the UI feel like a string list?** Extraction step optimized for chip CRUD, not schema mental model (types + relations + preview).  
4. **Why do presets mislead?** Presets are client-only, entity-only, and General drifts from server defaults.  
5. **Root cause:** Missing workspace-scoped **KG schema contract** (entity + relation allowlists + honest presets + hybrid visual UX) wired through API → pipeline → wizard.

## Causal ASCII

```ascii
                    Free-form RELATIONSHIP_TYPE in prompts
                              +
              No metadata.relation_types / strict
                              +
         UI presets = entity chips only (no schema preview)
                              +
              FE General ≠ Rust default_entity_types()
                              │
                              ▼
        Workspace "configured" but vocabulary incomplete
        Domain experts cannot express expected edges
        Graphs diverge; rebuild cannot target a schema
                              │
                              ▼
        Symptom: cannot visually configure KG Types+Relations
```

## Desired after state (v1)

```ascii
Domain card (Manufacturing / Healthcare / …)
        │
        ├─ entity_types[] + strict + colors
        └─ relation_types[] + strict
                │
                ▼
        Dual panels + mini schema preview
                │
                ▼
        PUT workspace metadata → ExtractionSchema
                │
                ▼
        Future ingest: guided/strict entity AND relation types
```

## Non-goals (v1) — see [ISSUE-typed-edges-v2.md](issues/ISSUE-typed-edges-v2.md)

- Typed edges `Source --REL--> Target` enforcement  
- Full React Flow canvas  
- OWL/RDF/SHACL  
