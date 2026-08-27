# 15 — Lens: Langfuse Expert

> **Cross-refs:** [SPEC-124](../124-langfuse-support/) · [deploy/kubernetes/README.md](../../deploy/kubernetes/README.md)

## Init keys (kind / E2E)

| Env | Value |
|-----|-------|
| `LANGFUSE_INIT_PROJECT_ID` | `edgequake-k8s` |
| `LANGFUSE_INIT_PROJECT_PUBLIC_KEY` | `pk-lf-edgequake-k8s` |
| `LANGFUSE_INIT_PROJECT_SECRET_KEY` | `sk-lf-edgequake-k8s-dev` |

EdgeQuake API Secret must use matching `LANGFUSE_PUBLIC_KEY` / `LANGFUSE_SECRET_KEY`.

## In-cluster OTLP wiring

| Setting | Value |
|---------|-------|
| `LANGFUSE_BASE_URL` (API pod) | `http://langfuse-web.langfuse.svc.cluster.local:3000` |
| OTLP endpoint | `{BASE}/api/public/otel/v1/traces` |
| Transport | **HTTP only** — gRPC will fail |

## Langfuse Helm v2 on kind

- Prereq: ClickHouse.**com** operator (`make k8s-prereqs`)
- Web OOM fix: `langfuse.web.resources.limits.memory: 2048Mi` + `NODE_OPTIONS=--max-old-space-size=1536`
- Use `langfuse.features.telemetryEnabled: false` — do not duplicate `TELEMETRY_ENABLED` in `additionalEnv`

## E2E verification (traces, not logs)

1. `GET /api/v1/settings/langfuse` → `export_active: true`
2. `POST /api/v1/query` with `session_id`
3. Poll `GET /api/public/v2/observations?filter=sessionId`

Playwright: reuse `spec124-langfuse-*.spec.ts` with `LANGFUSE_BASE_URL` pointing at port-forward URL.

## Not in Langfuse

- stdout application logs (use a log aggregator separately)
- USD cost attributes (by design, LAW-124)
