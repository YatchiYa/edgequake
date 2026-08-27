# 13 — Lens: Kubernetes Expert

> **Cross-refs:** [Helm](../../deploy/kubernetes/helm/) · [Implementation plan](04-implementation-plan.md)

## Charts

| Chart | Role |
|-------|------|
| `edgequake` | App stack (SRP) |
| `edgequake-stack` | Umbrella: edgequake + langfuse |

## Probes

| Workload | Liveness | Readiness |
|----------|----------|-----------|
| API | `/live:8080` | `/ready:8080` |
| Web | HTTP `/` | HTTP `/` |
| Postgres | `pg_isready` | `pg_isready` |

## Resources (kind profile)

- API: 256Mi–512Mi request
- Web: 128Mi–256Mi
- Postgres: 512Mi–1Gi

## Future hooks

- PDB: enable in production values
- HPA: API replicas + `EDGEQUAKE_TASK_DELIVERY=bridged`

## Prerequisites

- cert-manager, ClickHouse operator (Langfuse v2 chart)
