# 09 — Ops runbook

> **Cross-refs:** [deploy/kubernetes/README.md](../../deploy/kubernetes/README.md) · [Hub](README.md) · [Troubleshooting](../../deploy/kubernetes/README.md#troubleshooting)

## Install (kind / dev)

```bash
make k8s-prereqs      # once per cluster: cert-manager + ClickHouse.com operator + nginx
make k8s-kind-up      # kind cluster edgequake-spec138
make k8s-install      # Langfuse (langfuse ns) → EdgeQuake (edgequake ns)
make k8s-status
```

**Expected boot order:** postgres → migrate Job → api → web. API will not start until migrations are applied (LD-15).

## Proof (trace delivery to Langfuse)

```bash
make spec138-kubernetes-proof
```

Artifacts: `specs/138-kubernetes/measurements/`

Manual trace check:

```bash
# port-forward api + langfuse-web, then:
curl -sf http://127.0.0.1:8080/api/v1/settings/langfuse | jq .export_active   # true
# POST /api/v1/query with session_id → poll Langfuse observations API
```

## Access (port-forward)

```bash
kubectl --context kind-edgequake-spec138 port-forward -n edgequake svc/edgequake-web 3000:3000
kubectl --context kind-edgequake-spec138 port-forward -n edgequake svc/edgequake-api 8080:8080
kubectl --context kind-edgequake-spec138 port-forward -n langfuse svc/langfuse-web 3310:3000
```

Ingress hosts (kind): `edgequake.local`, `langfuse.local` → `/etc/hosts` → `127.0.0.1`.

## Upgrade

```bash
helm upgrade edgequake-stack deploy/kubernetes/helm/edgequake-stack \
  --kube-context kind-edgequake-spec138 \
  -f deploy/kubernetes/helm/edgequake-stack/values-kind.yaml \
  -n edgequake

helm upgrade langfuse langfuse/langfuse \
  --kube-context kind-edgequake-spec138 \
  --version 2.0.0 \
  -f deploy/kubernetes/helm/langfuse-values-kind.yaml \
  -n langfuse
```

Post-upgrade: migrate Job runs automatically (Helm hook).

## Rollback

```bash
helm rollback edgequake-stack -n edgequake
helm rollback langfuse -n langfuse
```

## Uninstall

```bash
make k8s-uninstall
make k8s-kind-down    # optional
```

## Debug checklist

| Symptom | Check | Doc |
|---------|-------|-----|
| API mock forbidden | `EDGEQUAKE_ALLOW_MOCK_PROVIDER` in kind values | [README](../../deploy/kubernetes/README.md#kind--e2e-profile-vs-production) |
| API schema behind | migrate Job logs; run `edgequake migrate` | [README](../../deploy/kubernetes/README.md#api-crashloopbackoff-pending-migrations) |
| pgvector missing | postgres init ConfigMap; exec CREATE EXTENSION | [README](../../deploy/kubernetes/README.md#api-warns-pgvector-extension-not-found) |
| langfuse-web OOM | `NODE_OPTIONS` + 2Gi limit | [README](../../deploy/kubernetes/README.md#langfuse-on-kind-memory--oom) |
| export_active false | Secret keys + in-cluster LANGFUSE_BASE_URL | [15-lens-langfuse](15-lens-langfuse.md) |
| No traces in UI | Langfuse Ready; session_id on query | E2E-138-09 |

```bash
kubectl --context kind-edgequake-spec138 logs -n edgequake deploy/edgequake-api -f
kubectl --context kind-edgequake-spec138 logs -n langfuse deploy/langfuse-web -f
kubectl --context kind-edgequake-spec138 get jobs -n edgequake    # migrate job
curl http://localhost:8080/health | jq .observability
```

## Does not need

- Ollama in cluster for kind E2E (mock + `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1`)
- Shared Postgres between EdgeQuake and Langfuse
- Shipping stdout logs to Langfuse (traces only)

## Production differences

- Real LLM provider; **no** `EDGEQUAKE_ALLOW_MOCK_PROVIDER`
- Auth on; external Langfuse stores recommended
- See `values-production.yaml.example`
