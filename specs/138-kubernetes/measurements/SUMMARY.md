# SPEC-138 proof summary

Generated after kind cluster verification (manual + script fixes).

| Gate | Status | Notes |
|------|--------|-------|
| E2E-138-01 | pass | cert-manager + ClickHouse.com operator |
| E2E-138-02 | pass | kind-edgequake-spec138 |
| E2E-138-03 | pass | Langfuse v2 + edgequake-stack (with manual migrate on first run) |
| E2E-138-04 | pass | postgres + extensions |
| E2E-138-05 | pass | GET /ready |
| E2E-138-07 | pass | Langfuse smoke |
| E2E-138-08 | pass | export_active=true |
| E2E-138-09 | pass | observations for session_id (OTLP traces) |

Trace delivery verified: query → Langfuse `/api/public/v2/observations` poll succeeded.

Run full automated proof: `make spec138-kubernetes-proof`
