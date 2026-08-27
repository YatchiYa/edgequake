# 00 — Architecture data

> **Cross-refs:** [WHY](00-why.md) · [Hub](README.md) · [deploy/kubernetes/README.md](../../deploy/kubernetes/README.md)

## Topology

```ascii
                    ┌──────────────── Ingress ────────────────┐
                    │  edgequake.local    langfuse.local     │
                    └──────────┬─────────────────┬────────────┘
                               │                 │
         namespace: edgequake  │                 │  namespace: langfuse
    ┌──────────────────────────┼─────────────────┼──────────────────────────┐
    │  edgequake-web:3000      │                 │  langfuse-web:3000       │
    │         │                │                 │         ▲                │
    │         ▼                │                 │         │ OTLP/HTTP        │
    │  edgequake-api:8080 ─────┼─────────────────┼─────────┘                │
    │         │                │                 │  langfuse-worker         │
    │         ▼                │                 │  postgres / clickhouse   │
    │  edgequake-postgres:5432 │                 │  redis / seaweedfs       │
    └──────────────────────────┴─────────────────┴──────────────────────────┘
```

## Boot sequence (LD-15)

```ascii
  postgres (init-extensions.sql on first PVC)
       │
       ▼
  Helm Job: edgequake migrate          ← API never auto-migrates at boot
       │
       ▼
  edgequake-api Deployment
       │
       ▼
  edgequake-web Deployment
```

Langfuse installs in parallel in `langfuse` namespace; API OTLP export retries until langfuse-web is Ready.

## Namespaces

| Namespace | Components |
|-----------|------------|
| `edgequake` | web Deployment, api Deployment, postgres StatefulSet, migrate Job |
| `langfuse` | Langfuse Helm release (web, worker, stores) — **separate Postgres** |
| `ingress-nginx` | nginx ingress controller (kind profile) |

## Ports

| Service | Port | Protocol |
|---------|------|----------|
| edgequake-api | 8080 | HTTP (`/live`, `/ready`, `/health`) |
| edgequake-web | 3000 | HTTP |
| edgequake-postgres | 5432 | TCP |
| langfuse-web | 3000 | HTTP (UI + OTLP `/api/public/otel/v1/traces`) |

## Image pins (GHCR)

| Component | Image |
|-----------|-------|
| API + migrate Job | `ghcr.io/raphaelmansuy/edgequake:${EDGEQUAKE_VERSION}` |
| Web | `ghcr.io/raphaelmansuy/edgequake-frontend:${EDGEQUAKE_VERSION}` |
| Postgres | `ghcr.io/raphaelmansuy/edgequake-postgres:${EDGEQUAKE_VERSION}` |
| Langfuse | Upstream `langfuse/langfuse:4` via Helm v2 |

## Env SSOT (API)

| Variable | Source | Notes |
|----------|--------|-------|
| `DATABASE_URL` | Secret | `edgequake-postgres.edgequake.svc.cluster.local` |
| `LANGFUSE_PUBLIC_KEY` | Secret | Must match Langfuse init keys |
| `LANGFUSE_SECRET_KEY` | Secret | Never exposed via API |
| `LANGFUSE_BASE_URL` | ConfigMap | In-cluster DNS; **never** `localhost` in pods |
| `LANGFUSE_PROJECT_ID` | ConfigMap | e.g. `edgequake-k8s` |
| `EDGEQUAKE_ALLOW_MOCK_PROVIDER` | ConfigMap | `"1"` kind/E2E only (v0.26+) |
| `EDGEQUAKE_API_URL` | Web ConfigMap | Browser-reachable URL (ingress) |

## Cluster prerequisites

| Prerequisite | Purpose |
|--------------|---------|
| Kubernetes >= 1.28 | Langfuse ClickHouse operator |
| Helm >= 3.17 or v4.x | Langfuse seaweedfs subchart |
| cert-manager | Langfuse / ClickHouse webhooks |
| **ClickHouse.com operator** | `clickhouseclusters.clickhouse.com` CRDs (not Altinity) |
| nginx ingress | kind profile |

## Observability

| Channel | Content |
|---------|---------|
| Langfuse | OTLP/HTTP **traces** (spans, sessions) |
| Pod stdout | Application logs (not sent to Langfuse) |
