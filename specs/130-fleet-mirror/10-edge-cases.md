# 10 — Edge cases

| ID | Case | Risk | Mitigation | Test |
|----|------|------|------------|------|
| E1 | Endpoint rename / name resolve miss while edge UUID known | Name resolve miss while edge exists | UUID map from sink | T2 |
| E2 | Workspace-scoped `ws::NAME` vs bare vector id | Endpoint resolve skew | Bare strip SSOT + map | T3 |
| E3 | `->` inside source entity name | Legacy parse split wrong | `rsplit_once` + map | T6 |
| E4 | `->` inside target name | Ambiguous parse | Document residual; prefer map | T6 / honest |
| E5 | ON CONFLICT existing relationship | Must return existing id | RETURNING after upsert | T3 |
| E6 | Sink missing entity FK (`missing_fk`) | Vector eligible without id | Fail-closed consistent skip | T3 |
| E7 | Empty relationship batch | No-op | Empty map OK | unit |
| E8 | Invalid / missing workspace_id in vector meta | Invalid workspace report | Keep SPEC-098 loud fail | T5 |
| E9 | Relation type case drift | SELECT miss | `normalize_relation_type_str` | existing 098 |
| E10 | Legacy (non-typed) RelVectors before RelGraph | True order race on legacy path | Product default typed; document | honest |
| E11 | Compensation after RelVectors fail | Leftover SQL spine | Expected (LAW-130-9); reprocess uses map | T2 |
| E12 | Concurrent workers same entity names | Index races | SPEC-120 oldest-wins; map still wins in-session | SPEC-120 + T2 |
| E13 | Placeholder AGE endpoints without entity row | No sink id | Residual sibling; fail-closed honest | out of scope |
| E14 | Partial chunk: some keys mapped, some not | Mixed resolved | Fail-closed if any eligible miss | T3 |
| E15 | Dense 350+ relationships | Volume / memory map size | HashMap fine; caps orthogonal | AI lens |
| E16 | Soften GraphMerge permanent | Masks bugs | Rejected (non-goal) | — |
| E17 | Bounded sleep retry only | Hides identity bugs; won't fix deterministic miss | Rejected as primary | LAW-130-8 |

## Out of scope

- Placeholder entity spine ensure (note for sibling spec).
- Documents status CHECK (#381 / SPEC-129).
- Extraction quality / LLM caps as substitute for identity.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Assessment: [11-honest-assessment.md](11-honest-assessment.md)
