# 10 — E2E Test Matrix (SPEC-104)

Assessment: [13-fix-assessment.md](13-fix-assessment.md) v3.

| ID | Asserts | Location | Status |
|----|---------|----------|--------|
| E2E-104-01 | `workspace_id`; no fail-open | `contract_spec104_datalayer` | Pass |
| E2E-104-02 | graph ≠ `edgequake` | same | Pass |
| E2E-104-03 | INV-03 dual + INV-01 embeddings + safe idents | same | Pass |
| E2E-104-04 | Atomic upsert + service Conflict→409 | same | Pass |
| E2E-104-05 | GIN via ag_catalog | same | Pass |
| E2E-104-06 | Naming helpers == inspector | same + storage config test | Pass |
| E2E-104-07 | KV chunk clears INV-03 | PG if `DATABASE_URL` | Pass/skip |
| E2E-104-08 | True orphan fires INV-03 | PG | Pass/skip |
| E2E-104-09 | Same name idempotent; diff name Conflict | PG service | Pass/skip |
| E2E-104-10 | Extra graph missing GIN → schema issue | PG | Pass/skip |

Related: `e2e_issue331_*`, `e2e_issue336_*`, `contract_spec089_*`.
