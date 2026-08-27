# 16 — Lens: Ops Expert

> **Cross-refs:** [Runbook](09-ops-runbook.md) · [deploy/kubernetes/README.md](../../deploy/kubernetes/README.md)

## Profiles

| Profile | Use | Values file |
|---------|-----|-------------|
| kind | E2E proof, local demo | `values-kind.yaml` |
| production | External stores, auth on | `values-production.yaml.example` |

## Secrets rotation

- Langfuse keys: update Secret, rolling restart API Deployment
- Postgres password: requires coordinated update of Secret + postgres env

## Upgrades

1. Bump `EDGEQUAKE_VERSION` in values
2. `helm upgrade edgequake-stack ...`
3. Watch API migration logs on first pod start

## Host requirements (kind)

- RAM: >= 16 GB recommended (Langfuse bundled stores; web needs ~2Gi limit + NODE_OPTIONS)
- CPU: >= 4 cores
- Disk: >= 30 GB free
- Tools: `kind`, `helm` >= 3.17 or v4.x, `kubectl`

## Kind profile env notes

- `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1` — required for mock LLM in v0.26+ (E2E only)
- Langfuse web: `NODE_OPTIONS=--max-old-space-size=1536` (prevents Next.js OOM)
- Migrate: Helm post-install Job runs `edgequake migrate` before API serves traffic

## Monitoring

- API: `GET /health`, `GET /metrics`
- Langfuse UI: traces tab for session verification
