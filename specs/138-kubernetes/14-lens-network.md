# 14 — Lens: Network Expert

> **Cross-refs:** [Architecture](00-architecture-data.md) · [LAW-138-5](01-first-principles.md)

## DNS (in-cluster)

| From | To | URL |
|------|-----|-----|
| API pod | EdgeQuake Postgres | `edgequake-postgres.edgequake.svc.cluster.local:5432` |
| API pod | Langfuse OTLP | `http://langfuse-web.langfuse.svc.cluster.local:3000` |
| Web pod | API | `http://edgequake-api.edgequake.svc.cluster.local:8080` |

## Ingress (kind)

- Controller: nginx (`ingress-nginx` namespace)
- Hosts: `edgequake.local`, `langfuse.local`
- Browser `EDGEQUAKE_API_URL`: `http://edgequake.local/api` or port-forward URL

## NetworkPolicy (optional)

When `networkPolicy.enabled: true`:

- API egress: postgres:5432, langfuse-web:3000, DNS:53
- Web egress: api:8080

## OTLP path

```
POST {LANGFUSE_BASE_URL}/api/public/otel/v1/traces
Authorization: Basic base64(pk:sk)
```

HTTP only — gRPC to Langfuse will fail (EC2).
