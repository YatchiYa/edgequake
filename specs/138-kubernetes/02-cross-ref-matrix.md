# 02 — Cross-ref matrix

> **Cross-refs:** [Laws](01-first-principles.md) · [Hub](README.md)

## This pack → code → tests

| This pack | Code / artifact | Contract | Test |
|-----------|-----------------|----------|------|
| LAW-138-1 | `deploy/kubernetes/helm/` | Single SSOT | `helm template` render |
| LAW-138-3 | `edgequake` + `langfuse` namespaces | Separate Postgres | E2E-138-04 |
| LAW-138-4 | [`langfuse.rs`](../../edgequake/crates/edgequake-observability/src/langfuse.rs) | OTLP/HTTP | E2E-138-09, `spec124-proof` |
| LAW-138-5 | `api-deployment.yaml` ConfigMap | In-cluster DNS | E2E-138-08 |
| LAW-138-6 | [`health.rs`](../../edgequake/crates/edgequake-api/src/handlers/health.rs) | `/live`, `/ready` | E2E-138-05 |
| LAW-138-7 | quickstart `EDGEQUAKE_API_URL` | Runtime injection | E2E-138-06 |
| LAW-138-8 | `api-deployment.yaml` preStop | 15s flush | E2E-138-14 |
| Init keys | [`docker-compose.langfuse.yml`](../../edgequake/docker/docker-compose.langfuse.yml) | `LANGFUSE_INIT_*` | E2E-138-07 |

## Other specs

| Spec | Relevance |
|------|-----------|
| [SPEC-124](../124-langfuse-support/) | OTLP export, Settings DTO, Playwright |
| [SPEC-018](../018-observability/) | Jaeger gRPC (optional, orthogonal) |
| [SPEC-027](../027-api-contract/) | Auth env vars for prod profile |
| [SPEC-057](../057-task-delivery/) | Multi-replica API |

## Divergence rule

If this matrix and a Helm template disagree, **Helm template + E2E gate** win. Update this file in the same PR.
