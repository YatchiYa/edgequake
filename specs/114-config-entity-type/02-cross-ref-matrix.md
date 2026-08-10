# 02 — Cross-ref matrix (SPEC-114)

| Finding | Code | Test / Gate | Law |
|---------|------|-------------|-----|
| F-114-01 | `helpers.rs` `apply_relation_types_metadata`; Create/Update requests; `WorkspaceResponse.relation_types`; `workspace_to_response` | Rust: `spec114_relation_types_persist` | LAW-114-1 |
| F-114-02 | `entity_type_policy.rs` relation schema + `enforce_relation_type`; `json_prompts.rs` section; `workspace_pipeline_factory.rs` | Rust: `enforce_relation_type_*`; pipeline unit | LAW-114-3/5 |
| F-114-03 | `kg-schema-presets.ts` (entity+relation per domain) | unit: preset has both lists | LAW-114-4 |
| F-114-04 | General preset ≡ `default_entity_types()` | unit: `general_matches_rust_defaults` | LAW-114-4 |
| F-114-05 | `RelationTypeSelector`; `KgSchemaPreview`; `workspace-extraction-step.tsx` | Playwright: `spec114-dual-panels-preview` | LAW-114-7 |
| F-114-06 | Rust `normalize_type_list`; FE `normalizeEntityType` shared | unit: normalize parity | LAW-114-2 |
| F-114-07 | `wizard-state.ts`; `model-payload.ts`; `workspace-config-diff.ts`; review step | Playwright: `spec114-reconfigure-persist` | LAW-114-1/6 |
| F-114-08 | `workspace-relation-types-card.tsx` (or extended entity card) | Playwright: workspace card visible | LAW-114-7 |
| F-114-09 | `e2e/spec114-kg-schema.spec.ts`; API e2e | full suite | LAW-114-8 |
| F-114-10 | `kg_schema_preset` metadata + draft | unit/e2e preset badge | LAW-114-4 |
| F-114-11 | Comment/doc fixes in presets + API | review | LAW-114-4 |

## Non-regression anchors

| Prior | Gate |
|-------|------|
| SPEC-085 | entity-type-selector interactive |
| SPEC-096 | `spec096-extraction-language` |
| SPEC-101 | `spec101-reconfigure-wizard` |
| SPEC-102 | `spec102-entity-type-colors` |
| SPEC-013 | `spec013_issue216_update_workspace_entity_types` |

## Issue ↔ finding

| Issue | Findings |
|-------|----------|
| ISSUE-relation-allowlist-api | F-114-01, F-114-06, F-114-10 |
| ISSUE-pipeline-relation-enforce | F-114-02 |
| ISSUE-preset-parity | F-114-03, F-114-04, F-114-11 |
| ISSUE-kg-schema-selector-ui | F-114-05, F-114-07, F-114-08 |
| ISSUE-e2e-gates | F-114-09 + all |
| ISSUE-typed-edges-v2 | deferred (v2) |

## SPEC-114b typed edges

| Finding / Law | Code | Gate |
|---------------|------|------|
| LAW-114-9 / F-114-13 | `type_list::normalize_relation_edges`, workspace metadata | G-114-11, G-114-14 |
| LAW-114-11 / enforce | `entity_type_policy::enforce_relation_edge` | G-114-12 |
| LAW-114-12 / F-114-12 | `TypedEdgeEditor` (honest; no modulo) | G-114-13 |
| LAW-114-13 / F-114-14 | `typed-edge-editor.tsx` + extraction step | G-114-13 |
| F-114-15 EDGE_PRESETS | `kg-schema-presets.ts` | G-114-05 |
