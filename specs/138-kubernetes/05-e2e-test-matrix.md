# 05 — E2E test matrix

> **Cross-refs:** [Edge cases](06-edge-cases.md) · [Ops runbook](09-ops-runbook.md)

**Proof target:** `make spec138-kubernetes-proof`

| ID | Gate | Method | Artifact |
|----|------|--------|----------|
| E2E-138-01 | Cluster prereqs | `k8s_prereqs.sh` | `e2e138-prereqs.txt` |
| E2E-138-02 | kind cluster | `k8s_kind_up.sh` | `e2e138-kind.txt` |
| E2E-138-03 | Helm install | `k8s_install_stack.sh` | `e2e138-helm-install.txt` |
| E2E-138-04 | Postgres health | `pg_isready` exec | `e2e138-postgres.txt` |
| E2E-138-05 | API readiness | `GET /ready` | `e2e138-api-ready.txt` |
| E2E-138-06 | Web health | HTTP 200 `/` | `e2e138-web.txt` |
| E2E-138-07 | Langfuse health | adapted smoke script | `e2e138-langfuse-smoke.txt` |
| E2E-138-08 | Settings DTO | `export_active`, `ui_url` | `e2e138-settings-dto.txt` |
| E2E-138-09 | **Trace delivery** | query + observations poll | `e2e138-trace-delivery.txt` |
| E2E-138-10 | Playwright settings | `spec124-langfuse-settings.spec.ts` | `e2e138-playwright.txt` |
| E2E-138-11 | Playwright sessions | `spec124-langfuse-sessions.spec.ts` | `e2e138-playwright.txt` |
| E2E-138-12 | Secret leak guard | settings JSON scan | (in settings-dto step) |
| E2E-138-14 | Pod restart | delete API pod, re-query | (optional in proof) |
| E2E-138-15 | Helm test | `helm test edgequake-stack` | `e2e138-helm-test.txt` |

## Skip policy

- Requires: `kind`, `kubectl`, `helm` >= 3.17, Docker, ~16GB host RAM.
- Without kind: run `helm template` + `spec124-proof` only.

## Existing contracts (reuse)

- [`spec124-proof`](../../Makefile) — InMemory OTEL unit tests
- [`spec124-langfuse-*.spec.ts`](../../edgequake_webui/e2e/) — Playwright
