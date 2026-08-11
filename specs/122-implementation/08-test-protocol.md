# 08 — Test Protocol

## Gates

| Gate | Meaning | Pass criteria |
|------|---------|---------------|
| G0 | Spec pack complete | README..12 + 8 lenses exist |
| G1 | Arm A Ollama measured | Numbers in 10-reproduction |
| G2 | Arm B Mistral measured | Numbers in 10-reproduction |
| G3 | H1–H5 decided | 12-pdf + 10-reproduction |
| G4 | Phase A docs/FAQ | Concurrency SSOT published |
| G5 | Harness runnable | Script exits 0 on healthy stack |
| G6 | Fairness e2e | Query under ingest does not starve (spot) |
| G7 | Issue update | #361/#365 commented |

## Normative tests

| ID | Kind | Assertion |
|----|------|-----------|
| T1 | Measure | Arm C single-doc baseline recorded |
| T2 | Measure | Arm A N=3+ text: overlap ≤1 ingest (local) |
| T3 | Measure | Arm A N=10: t_all ≈ N × t_single (±slack) under tenant=1 |
| T4 | Measure | Arm B shows higher docs/min or wider overlap than Arm A |
| T5 | Measure | PDF arm vs text arm stage share |
| T6 | Unit/contract | `MAX_CONCURRENT_FILE_UPLOADS === 3` |
| T7 | API | Batch > max files rejected |
| T8 | E2E | Cancel one Pending; others proceed |
| T9 | Integration | Tenant park waiters rise when N > MAX_TASKS_PER_TENANT |
| T10 | Docs | FAQ contains concurrency matrix + queue-metrics pointer |
| T11 | Live (ignored) | Ollama measurement job |
| T12 | Live (ignored) | Mistral measurement job |
| T13 | Playwright (optional) | Bulk progress copy visible |
| T14 | Ops | Harness writes JSON artifact under specs/122-implementation/measurements/ |
| T15 | Product | UI/FAQ never claim searchable on admit alone |

## Live arms (detail)

See [10-reproduction.md](10-reproduction.md). Live tests are `@ignore` / script-driven when keys missing — never fail CI for absent `MISTRAL_API_KEY`.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Repro: [10-reproduction.md](10-reproduction.md)
