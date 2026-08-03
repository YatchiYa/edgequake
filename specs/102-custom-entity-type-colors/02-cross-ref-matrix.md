# 02 — Cross-ref matrix (SPEC-102)

| Finding | Code | Test / Gate | Law |
|---------|------|-------------|-----|
| F-102-01 | `helpers.rs` `apply_entity_type_colors_metadata`; Create/Update requests; `WorkspaceResponse.entity_type_colors`; `workspace_to_response` | Rust: `spec102_entity_type_colors_persist` | LAW-102-2 |
| F-102-02 | `entity-type-colors.ts`; delete dupes in expansion/search/label-search; re-export `label-utils` | unit: `entity-type-colors.test.ts` / `spec102-resolver-unit` | LAW-102-1/5 |
| F-102-03 | Expanded `ENTITY_TYPE_COLORS` defaults | unit: default types resolve ≠ DEFAULT | LAW-102-1 |
| F-102-04 | `EntityTypeSelector` swatch props; legend editable swatch; wizard payloads | Playwright: `spec102-selector-picker`, `spec102-legend-recolor` | LAW-102-6 |
| F-102-05 | Hex validate in Rust + TS `isValidEntityTypeHex` | Rust invalid hex 400; unit invalid hex | LAW-102-3 |
| F-102-06 | `e2e/spec102-entity-type-colors.spec.ts` | Playwright suite | LAW-102-8 |
| F-102-07 | `graph-renderer` community branch | Playwright: `spec102-community-mode` | LAW-102-4 |
| F-102-08 | Reset removes key; `stripDefaultOverrides` | Playwright: `spec102-reset-default` | LAW-102-3 |

## Non-regression anchors

| Prior | Gate |
|-------|------|
| SPEC-100 | `spec100-graph-cls` |
| SPEC-085 | `entity-type-selector.spec.ts` |
| Graph UX | `graph-responsive.spec.ts` |
| SPEC-013 | `spec013_issue216_update_workspace_entity_types` |

## Issue ↔ finding

| Issue | Findings |
|-------|----------|
| ISSUE-resolver-dry | F-102-02, F-102-03 |
| ISSUE-api-persist | F-102-01, F-102-05 |
| ISSUE-selector-ui | F-102-04 |
| ISSUE-legend-edit | F-102-04, F-102-07, F-102-08 |
| ISSUE-e2e-gates | F-102-06 + all |
