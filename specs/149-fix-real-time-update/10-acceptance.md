# 10 — Acceptance

| ID | Criterion | Proof | Status |
|----|-----------|-------|--------|
| AC-149-01 | Spec pack complete under `specs/149-fix-real-time-update/` | Doc review | Done |
| AC-149-02 | Auth-on connect uses current token; login rebuilds socket | U-149-01, E-149-01 | Done |
| AC-149-03 | Owner subscription delivers StageTransition | C-149-01 | Done |
| AC-149-04 | Foreign workspace receives no track event | C-149-02 | Done |
| AC-149-05 | Unsubscribed track receives no event | C-149-03 | Done |
| AC-149-06 | Reconnect replays subscriptions | U-149-03 | Done |
| AC-149-07 | PDF fields normalize; UI advances without list GET | U-149-10, E-149-02 | Done |
| AC-149-08 | Banner/toast recover on Retry | Provider toast ids + reconnectRealtime | Done |
| AC-149-09 | SSE uses authenticated fetch | U-149-20 | Done |
| AC-149-10 | `make spec149-proof` green | Makefile | Done |
| AC-149-11 | SPEC-083 isolation preserved | C-149-02, C-149-05 | Done |
| AC-149-12 | AsyncAPI documents subscribe protocol | openapi_asyncapi.rs | Done |

## Done when

All AC-149-01..12 checked; README status board flipped to Done for I*/T*/A1.

## Cross-refs

- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
