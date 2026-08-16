# 09 — Acceptance

## Must pass

| ID | Criterion | Proof |
|----|-----------|-------|
| A1 | zz-raw five keys parse correctly with index | unit |
| A2 | Source-arrow key still parses correctly | unit + existing contract |
| A3 | Fleet mirror completes for target-arrow without known map when spine exists | contract_spec133 |
| A4 | Fail-closed when neither split resolves | unit/contract |
| A5 | SPEC-130 known-map hit path unchanged | e2e_spec130 green |
| A6 | Ops residual text updated | doc review |
| A7 | Classifier still treats fleet miss as GraphMerge permanent | existing unit |

## Nice to have

| ID | Criterion |
|----|-----------|
| N1 | Fail message distinguishes parse-class vs spine-class |
| N2 | Escaped v2 key format design spike |

## Exit

SPEC-133 is **Done** when A1–A7 pass and [10-edge-cases.md](10-edge-cases.md) mitigations are checked.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
