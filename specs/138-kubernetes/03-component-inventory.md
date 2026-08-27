# 03 — Component inventory (Compose → K8s)

> **Cross-refs:** [Architecture](00-architecture-data.md) · [Helm](../../deploy/kubernetes/helm/)

| Compose (quickstart / langfuse) | Kubernetes | Chart / template |
|--------------------------------|------------|------------------|
| `postgres` service | StatefulSet + PVC + Service | `edgequake/templates/postgres-*` |
| `api` service | Deployment + Service | `edgequake/templates/api-*` |
| `frontend` service | Deployment + Service | `edgequake/templates/web-*` |
| `edgequake` network | Namespace `edgequake` | `namespace.yaml` |
| Compose healthchecks | liveness/readiness probes | deployment templates |
| `LANGFUSE_*` env on api | ConfigMap + Secret | `configmap.yaml`, stack bootstrap |
| `langfuse-web` (compose.langfuse) | Langfuse Helm subchart | `edgequake-stack` dependency |
| `langfuse-worker` | Langfuse Helm subchart | upstream |
| Langfuse postgres/redis/ch/minio | Bundled in langfuse-k8s v2 | upstream values |
| `make stack` | `make k8s-install` | Makefile |
| `make spec124-langfuse-e2e` | `make spec138-kubernetes-proof` | scripts |

## Image mapping

| Compose image | K8s `values.yaml` key |
|---------------|----------------------|
| `ghcr.io/raphaelmansuy/edgequake` | `api.image.repository` |
| `ghcr.io/raphaelmansuy/edgequake-frontend` | `web.image.repository` |
| `ghcr.io/raphaelmansuy/edgequake-postgres` | `postgres.image.repository` |
