# 01 — First principles (LAW-138)

> **Cross-refs:** [WHY](00-why.md) · [Architecture](00-architecture-data.md) · [Matrix](02-cross-ref-matrix.md)

## Axioms

1. **Compose parity.** K8s env wiring must match [`docker-compose.quickstart.yml`](../../docker-compose.quickstart.yml) semantics.
2. **Traces ≠ logs.** Langfuse ingests OTLP spans; stdout JSON logs need a separate pipeline.
3. **Dual Postgres.** EdgeQuake and Langfuse never share a `DATABASE_URL`.
4. **Secrets env-only.** Langfuse keys never enter EdgeQuake DB (LAW-124-2).
5. **HTTP OTLP only.** Langfuse rejects gRPC export.

## Causal diagram

```text
  helm install edgequake-stack
           │
           ├─ edgequake ns: postgres SS → api Deployment → web Deployment
           │
           └─ langfuse ns: langfuse-k8s v2 (web + worker + stores)
                    │
                    ▼
  API boot with LANGFUSE_* → edgequake-observability OTLP/HTTP
                    │
                    ▼
  Query with session_id → spans → Langfuse observations API
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-138-1** | **Helm SSOT** — All K8s manifests live under `deploy/kubernetes/`; specs link, never duplicate YAML. |
| **LAW-138-2** | **Namespace isolation** — EdgeQuake Postgres in `edgequake`; Langfuse stores in `langfuse`. |
| **LAW-138-3** | **No shared DATABASE_URL** — Never point Langfuse at EdgeQuake Postgres or vice versa. |
| **LAW-138-4** | **Trace contract** — E2E proves OTLP trace delivery, not stdout log ingestion. |
| **LAW-138-5** | **In-cluster LANGFUSE_BASE_URL** — API pods use cluster DNS (`*.svc.cluster.local`), never `localhost`. |
| **LAW-138-6** | **Probe semantics** — API liveness `/live`, readiness `/ready` (not `/health` for readiness). |
| **LAW-138-7** | **Runtime API URL** — Web uses `EDGEQUAKE_API_URL` at runtime, not baked `NEXT_PUBLIC_*`. |
| **LAW-138-8** | **Graceful trace flush** — API `preStop` delay + `ObservabilityGuard` on SIGTERM. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | `edgequake` chart = app only; Langfuse = upstream dependency; bootstrap Job = key wiring. |
| **O** | Production overrides via `values-production.yaml.example`. |
| **L** | External Postgres via `postgres.enabled: false` + external `DATABASE_URL`. |
| **I** | Helm test hooks vs shell E2E — separate interfaces. |
| **DRY** | Shared `langfuse_e2e_common.sh`; reuse SPEC-124 Playwright specs. |
