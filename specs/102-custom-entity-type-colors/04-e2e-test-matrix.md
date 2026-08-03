# 04 — E2E test matrix (SPEC-102)

## Non-regression

| Suite | Assert |
|-------|--------|
| `entity-type-selector.spec.ts` | Type chips still work |
| `graph-responsive.spec.ts` | Legend / layout |
| `spec100-graph-cls.spec.ts` | Graph header CLS slot |

## New gates

| Gate ID | Wave | Type | Assert | Finding |
|---------|------|------|--------|---------|
| `spec102-resolver-unit` | W0 | bun unit | override > default > DEFAULT; hex validate | F-102-02/03/05 |
| `spec102-persist-api` | W1 | Rust | PUT/GET colors; invalid hex 400 | F-102-01/05 |
| `spec102-selector-picker` | W3/W4 | Playwright | picker → payload hex | F-102-04 |
| `spec102-legend-recolor` | W3/W4 | Playwright | swatch + graph uses custom | F-102-04/06 |
| `spec102-community-mode` | W4 | Playwright | community ignores type overrides | F-102-07 |
| `spec102-reset-default` | W4 | Playwright | reset removes override | F-102-08 |
| `spec102-invalid-hex` | W4 | unit/API | reject `#gg0000` / empty | F-102-05 |

## Suggested file layout

```
edgequake_webui/e2e/spec102-entity-type-colors.spec.ts
edgequake_webui/src/lib/graph/entity-type-colors.test.ts
edgequake/crates/edgequake-api/tests/e2e_spec013_github_issues.rs  # spec102_* tests
edgequake/crates/edgequake-core/src/workspace_service_impl/helpers_tests.rs
```

## CI wiring

```bash
cd edgequake_webui && bun test src/lib/graph/
cd edgequake_webui && pnpm exec playwright test e2e/spec102-entity-type-colors.spec.ts
cargo test -p edgequake-core --lib apply_entity_type_colors
cargo test -p edgequake-api --test e2e_spec013_github_issues spec102
```
