# 01 — Finding register (SPEC-114)

## Severity legend

| Level | Meaning |
|-------|---------|
| P0 | Blocks correct shared KG vocabulary / data integrity |
| P1 | Missing product capability or consistency |
| P2 | Polish / completeness |

## Findings

| ID | Severity | Finding | Law | Inherits |
|----|----------|---------|-----|----------|
| F-114-01 | P0 | No workspace persistence for relation type allow-list | LAW-114-1 | SPEC-085 metadata pattern |
| F-114-02 | P0 | Pipeline never prompts/enforces relation types from workspace | LAW-114-3/5 | entity_type_policy |
| F-114-03 | P1 | Domain presets are entity-only; no relation lists | LAW-114-4 | entity-presets.ts |
| F-114-04 | P1 | FE General preset ≠ Rust `default_entity_types()` | LAW-114-4 | defaults drift |
| F-114-05 | P1 | Extraction UX is flat chips — no dual-panel schema + preview | LAW-114-7 | SPEC-101 step |
| F-114-06 | P1 | No shared normalize helper for type lists (entity-only path) | LAW-114-2 | helpers.rs |
| F-114-07 | P1 | Wizard/payload/diff ignore relation fields | LAW-114-1/6 | model-payload |
| F-114-08 | P2 | Workspace cards do not show relation vocabulary | LAW-114-7 | workspace page |
| F-114-09 | P2 | No E2E asserting relation config → persist → prompt path | LAW-114-8 | e2e |
| F-114-10 | P2 | Preset identity not stored — UX cannot show "Manufacturing" honestly after reload | LAW-114-4 | kg_schema_preset |
| F-114-11 | P2 | Doc drift: max 20 vs 50; "5 presets" vs 6 | LAW-114-4 | comments/README |
| F-114-12 | P0 | Visual schema invents Source—REL→Target via modulo (not data) | LAW-114-12 | kg-schema-preview |
| F-114-13 | P0 | No `relation_edges` persistence / pipeline endpoint enforce | LAW-114-9/11 | type_list / policy |
| F-114-14 | P1 | Cannot Add/Edit/Delete associations by entity type in wizard | LAW-114-13 | TypedEdgeEditor |
| F-114-15 | P1 | Domain presets lack curated typed edges | LAW-114-4/10 | EDGE_PRESETS |

## Evidence map

| Finding | Evidence |
|---------|----------|
| F-114-01 | Create/Update DTOs lack `relation_types` |
| F-114-02 | `json_prompts.rs` free-form `RELATIONSHIP_TYPE`; no `enforce_relation_type` |
| F-114-03 | `ENTITY_PRESETS.*.types` only |
| F-114-04 | FE general: DATE/TECHNOLOGY/… vs Rust CREATURE/METHOD/OTHER/… |
| F-114-05 | `EntityTypeSelector` tabs Presets/Bulk; no relation panel / preview |
| F-114-06 | `normalize_entity_types` name-bound; not reused for relations |
| F-114-07 | `WizardDraft` / `buildWorkspaceUpdatePayload` entity-only |
| F-114-08 | `workspace-entity-types-card.tsx` only |
| F-114-09 | No `spec114-*` Playwright / Rust e2e |
| F-114-10 | Client detects preset by list equality only |
| F-114-11 | Stale comments in API docs / entity-presets header |
