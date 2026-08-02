# 01 — Finding register (SPEC-102)

## Severity legend

| Level | Meaning |
|-------|---------|
| P0 | Blocks correct shared analysis / data integrity |
| P1 | Inconsistency or missing product capability |
| P2 | Polish / completeness |

## Findings

| ID | Severity | Finding | Law | Inherits |
|----|----------|---------|-----|----------|
| F-102-01 | P0 | No workspace persistence for entity-type colors | LAW-102-2 | SPEC-085 metadata pattern |
| F-102-02 | P0 | Duplicate palettes drift (`expansion`, `graph-search`, `label-search`) | LAW-102-1/5 | DRY |
| F-102-03 | P1 | Default palette incomplete vs Rust `default_entity_types` | LAW-102-1 | pipeline defaults |
| F-102-04 | P1 | No admin/user UI to assign colors | LAW-102-6 | EntityTypeSelector / legend |
| F-102-05 | P1 | No hex validation at API boundary | LAW-102-3 | UpdateWorkspace |
| F-102-06 | P2 | No E2E asserting custom color → graph/legend | LAW-102-8 | graph e2e |
| F-102-07 | P2 | Community mode must ignore type overrides | LAW-102-4 | colorMode |
| F-102-08 | P2 | Reset-to-default must remove override key | LAW-102-3 | metadata size |

## Evidence map

| Finding | Evidence |
|---------|----------|
| F-102-01 | Workspace helpers only apply `entity_types` / strict / language |
| F-102-02 | Local `TYPE_COLORS` in expansion + search; Tailwind map in label-search |
| F-102-03 | `label-utils.ts` vs `default_entity_types()` |
| F-102-04 | EntityTypeSelector has chips without swatches |
| F-102-05 | No color field on Create/Update requests |
| F-102-06 | Graph e2e cover layout/search/CLS, not hex |
