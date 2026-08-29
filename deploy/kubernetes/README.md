# EdgeQuake on Kubernetes (SPEC-138)

Deploy EdgeQuake (web + API + PostgreSQL) with in-cluster Langfuse v4 for OTLP trace observability. To attach EdgeQuake to an **existing Langfuse 3.1.x**, see [Existing Langfuse 3.1.x](#existing-langfuse-31x) and [docs/operations/langfuse-3.1.md](../../docs/operations/langfuse-3.1.md).

**Spec pack:** [specs/138-kubernetes/README.md](../../specs/138-kubernetes/README.md)

---

## What you get

| Component | Namespace | Purpose |
|-----------|-----------|---------|
| edgequake-web | `edgequake` | Next.js UI |
| edgequake-api | `edgequake` | Rust API (OTLP to Langfuse v4; native ingestion on 3.1.x 404) |
| edgequake-postgres | `edgequake` | pgvector + Apache AGE |
| langfuse-web + worker + stores | `langfuse` | Self-hosted Langfuse v4 |

```ascii
  edgequake namespace          langfuse namespace
  ┌─────────────────┐          ┌──────────────────┐
  │ web → api → pg  │─ OTLP ──▶│ langfuse-web     │
  └─────────────────┘          │ worker + stores  │
                               └──────────────────┘
```

---

## Observability contract (read this first)

EdgeQuake sends **traces** to Langfuse (SPEC-124), **not** application stdout logs.

The **kind/Helm profile in this repo pins Langfuse v4** and uses **OTLP/HTTP** at `/api/public/otel/v1/traces`. Self-hosted **Langfuse 3.1.x has no OTLP path** (404). Default `EDGEQUAKE_LANGFUSE_API=auto` (Helm `api.langfuse.api`) probes once and falls back to native `POST /api/public/ingestion` **only on HTTP 404**.

**How to wire EdgeQuake to Langfuse 3.1.x:** [docs/operations/langfuse-3.1.md](../../docs/operations/langfuse-3.1.md) · Helm knobs below: [Existing Langfuse 3.1.x](#existing-langfuse-31x). Upgrade to ≥ 3.22 remains recommended (ingestion is deprecated; Cloud sunset 2026-11-16).

| Data | Destination |
|------|-------------|
| LLM/query spans, sessions (v4 / ≥ 3.22) | Langfuse OTLP `/api/public/otel/v1/traces` |
| LLM/query spans, sessions (3.1.x) | Langfuse ingestion `/api/public/ingestion` (`api_resolved=ingestion`) |
| JSON/plain logs | Pod stdout only (use Loki/Cloud Logging separately) |

Verify export is active **and** which transport is live:

```bash
curl http://localhost:8080/api/v1/settings/langfuse \
  | jq '{export_active, base_url, api, api_resolved, project_id}'
# export_active must be true
# v4 / ≥ 3.22: api_resolved = "otlp"
# 3.1.x:       api_resolved = "ingestion"  and base_url is *this* cluster, not Cloud
```

Verify traces arrived (after a query with `session_id`):

```bash
# Poll Langfuse Public API — see scripts/langfuse_e2e_common.sh
make spec138-kubernetes-proof   # automated
```

---

## Prerequisites

### Tools

| Tool | Version | Notes |
|------|---------|-------|
| Kubernetes | >= 1.28 | kind, OrbStack, EKS, GKE, … |
| Helm | >= 3.17 or v4.x | Langfuse chart requires 3.17+ |
| kubectl | matches cluster | |
| kind | latest | local proof only (`brew install kind`) |
| Host RAM | **>= 16 GB** | Langfuse bundled stores are heavy |

### Cluster-wide (once per cluster)

`make k8s-prereqs` installs:

1. **cert-manager** — required by Langfuse ClickHouse operator webhooks
2. **ClickHouse.com operator** (`ghcr.io/clickhouse/clickhouse-operator-helm`) — **not** the Altinity operator; Langfuse Helm v2 preflights `clickhouseclusters.clickhouse.com` CRDs
3. **nginx ingress** — kind profile only. Chart defaults set `proxy-buffering: "off"` so SSE (`text/event-stream`) is not gzip-buffered at the ingress.

---

## Boot sequence (how the stack starts)

EdgeQuake v0.26+ **never auto-migrates at API boot** (LD-15). The Helm chart enforces this order:

```ascii
  1. postgres StatefulSet starts
       └─ initdb runs init-extensions.sql (pgvector, age, …) on first PVC only

  2. migrate Job (Helm post-install/post-upgrade hook)
       └─ edgequake migrate  → applies SAFE SCHEMA migrations

  3. api Deployment starts
       └─ refuses boot if schema still behind (exit 78)

  4. web Deployment starts

  (parallel) langfuse namespace: langfuse-web must be Ready before traces persist
```

**Fresh install:** empty PVC → extensions created automatically.  
**Existing PVC without extensions:** delete PVC or run `init-extensions.sql` manually (see Troubleshooting).

---

## Quick start (kind)

```bash
make k8s-prereqs          # cert-manager + ClickHouse.com operator + nginx
make k8s-kind-up          # cluster: edgequake-spec138
make k8s-install          # Langfuse (langfuse ns) then EdgeQuake (edgequake ns)
make k8s-status           # all pods Running/Ready
```

Full E2E proof (trace delivery + Playwright):

```bash
make spec138-kubernetes-proof
```

Render charts without a cluster:

```bash
make spec138-helm-template
```

---

## Kind / E2E profile vs production

The **kind profile** (`values-kind.yaml`) uses settings that are **not** for production:

| Setting | Kind/E2E value | Production |
|---------|----------------|------------|
| `EDGEQUAKE_LLM_PROVIDER` | `mock` | `ollama`, `openai`, … |
| `EDGEQUAKE_ALLOW_MOCK_PROVIDER` | `"1"` | **unset** (mock forbidden in v0.26+) |
| `EDGEQUAKE_DEV_MODE` | `"true"` | `"false"` |
| `EDGEQUAKE_AUTH_ENABLED` | `"false"` | `"true"` |
| Langfuse stores | bundled (single-replica) | external / managed recommended |

**Why `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1`?**  
v0.26+ rejects mock as the server-default LLM unless this test escape hatch is set. Without it the API crash-loops with:

```text
Mock LLM provider is forbidden as the server default.
```

Production must use a real provider and credentials — see `helm/edgequake/values-production.yaml.example`.

---

## Langfuse on kind (memory / OOM)

Langfuse v4 web (Next.js) can OOM at the default Node heap (~256 MB) even when the container limit is higher. The kind values file sets:

```yaml
# deploy/kubernetes/helm/langfuse-values-kind.yaml
langfuse:
  web:
    resources:
      limits:
        memory: 2048Mi
    pod:
      additionalEnv:
        - name: NODE_OPTIONS
          value: "--max-old-space-size=1536"
```

**Symptom:** `langfuse-web` CrashLoopBackOff, logs show `JavaScript heap out of memory`.  
**Fix:** apply the values above; `helm upgrade langfuse … -f langfuse-values-kind.yaml`.

Also use `langfuse.features.telemetryEnabled: false` — do **not** duplicate `TELEMETRY_ENABLED` in `additionalEnv` (Helm v2 chart rejects both).

---

## Langfuse keys (init → EdgeQuake API)

Init keys in `langfuse-values-kind.yaml` must match EdgeQuake Secret:

| Variable | Kind/E2E value |
|----------|----------------|
| `LANGFUSE_INIT_PROJECT_ID` | `edgequake-k8s` |
| `LANGFUSE_INIT_PROJECT_PUBLIC_KEY` | `pk-lf-edgequake-k8s` |
| `LANGFUSE_INIT_PROJECT_SECRET_KEY` | `sk-lf-edgequake-k8s-dev` |

API wiring (in-cluster DNS — **never** `localhost` inside pods):

| Variable | Value |
|----------|-------|
| `LANGFUSE_BASE_URL` | `http://langfuse-web.langfuse.svc.cluster.local:3000` |
| `LANGFUSE_PROJECT_ID` | `edgequake-k8s` |

`api.langfuse.api` defaults to **`auto`** (ConfigMap `EDGEQUAKE_LANGFUSE_API`). Kind v4 does not need to force `otlp`.

---

## Existing Langfuse 3.1.x

This chart does **not** install Langfuse 3.1. Use it when the cluster already runs Langfuse **3.1.x** (customer chart / VM) and EdgeQuake must export traces there.

Step-by-step (env, local compose, verify, limits): **[docs/operations/langfuse-3.1.md](../../docs/operations/langfuse-3.1.md)**.

Helm overlay against **that** Service (never `localhost` in the API pod):

```yaml
api:
  langfuse:
    baseUrl: "http://<langfuse-3.1-svc>.<ns>.svc.cluster.local:3000"
    projectId: "<project-id-from-that-instance>"
    existingSecret: edgequake-langfuse-secret   # keys issued by *this* Langfuse
    api: auto   # probe OTLP → 404 → native ingestion
```

After rollout:

1. `api.langfuse.baseUrl` must match the in-cluster 3.1 origin (empty → Cloud).
2. Settings JSON: `api_resolved` is **`ingestion`**, `export_active` is `true`.
3. Confirm OTLP 404: `POST {base}/api/public/otel/v1/traces` → 404.
4. Run a query; 3.1 UI shows **GENERATION** + **SPAN** only (`retriever`/`embedding`/`chain` collapse to SPAN).

Do **not** set `api: otlp` against 3.1.x. Prefer upgrading that Langfuse to ≥ 3.22 so `auto` resolves to `otlp`.

---

## Access

**/etc/hosts** (ingress):

```
127.0.0.1 edgequake.local langfuse.local
```

**Port-forward:**

```bash
kubectl port-forward -n edgequake svc/edgequake-web 3000:3000
kubectl port-forward -n edgequake svc/edgequake-api 8080:8080
kubectl port-forward -n langfuse svc/langfuse-web 3310:3000
```

Use `--context kind-edgequake-spec138` when not on the default kubeconfig context.

---

## Charts

| Chart | Path | Purpose |
|-------|------|---------|
| `edgequake` | `helm/edgequake/` | App: postgres, api, web, migrate Job, ingress |
| `edgequake-stack` | `helm/edgequake-stack/` | Wrapper (edgequake subchart) |
| Langfuse values | `helm/langfuse-values-kind.yaml` | Standalone install into `langfuse` ns |

Langfuse installs **separately** from EdgeQuake (separate namespace + Postgres — LAW-138-3).

---

The API image is **distroless** (no shell). `kubectl exec` cannot open `/bin/sh`. Use `kubectl logs`, HTTP probes (`/live`, `/ready`), or a debug sidecar. Helm `preStop` runs `edgequake pre-stop`, not `sleep`.

## Troubleshooting

### API CrashLoopBackOff: mock LLM forbidden

```text
Mock LLM provider is forbidden as the server default.
```

**Fix (kind only):** set `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1` in values; `helm upgrade edgequake-stack …`  
**Fix (prod):** set `EDGEQUAKE_LLM_PROVIDER=openai` (or ollama) + credentials.

### API CrashLoopBackOff: pending migrations

```text
BOOT_GATE_REFUSAL: STOP — database schema is behind this binary
```

**Fix:** run migrate Job or manually:

```bash
kubectl run edgequake-migrate --rm -i --restart=Never -n edgequake \
  --image=ghcr.io/raphaelmansuy/edgequake:0.26.3 \
  --env="DATABASE_URL=postgres://edgequake:edgequake_secret@edgequake-postgres:5432/edgequake" \
  --env="EDGEQUAKE_ALLOW_MOCK_PROVIDER=1" \
  --command -- edgequake migrate
```

On fresh installs the Helm post-install migrate Job should handle this automatically.

### API warns: pgvector extension not found

Postgres PVC was created before `init-extensions.sql` was mounted.

**Fix:** apply extensions once:

```bash
kubectl exec -n edgequake edgequake-postgres-0 -- psql -U edgequake -d edgequake -c \
  "CREATE EXTENSION IF NOT EXISTS vector; CREATE EXTENSION IF NOT EXISTS pg_trgm;"
```

Or delete the postgres PVC and reinstall (data loss).

### Langfuse Helm install fails: ClickHouse CRDs not found

**Fix:** `make k8s-prereqs` (installs ClickHouse.**com** operator, not Altinity).

### Langfuse web OOM

See [Langfuse on kind (memory / OOM)](#langfuse-on-kind-memory--oom) above.

### Traces not in Langfuse UI

1. `curl …/api/v1/settings/langfuse` → `export_active: true` **and** `base_url` is this cluster (not Cloud)
2. Check `api_resolved`: `otlp` on v4/≥3.22, **`ingestion` on 3.1.x** ([langfuse-3.1.md](../../docs/operations/langfuse-3.1.md))
3. `LANGFUSE_BASE_URL` must be in-cluster DNS (not `localhost`)
4. Langfuse web pod must be Ready; on first 3.1 boot restart the worker after Prisma
5. Query must include `session_id`; poll observations API (see SPEC-124 E2E)

### Helm: namespace langfuse ownership conflict

Do not create `langfuse` namespace from the edgequake-stack chart. Langfuse release owns that namespace.

---

## Teardown

```bash
make k8s-uninstall
make k8s-kind-down    # optional: destroy kind cluster
```

---

## Production

Copy and customize:

- [`helm/edgequake/values-production.yaml.example`](helm/edgequake/values-production.yaml.example)
- Langfuse: point stores to external Postgres/ClickHouse/Redis/S3 (`*.deploy: false`)
- Real LLM provider + auth enabled; **do not** set `EDGEQUAKE_ALLOW_MOCK_PROVIDER`

Further reading: [specs/138-kubernetes/09-ops-runbook.md](../../specs/138-kubernetes/09-ops-runbook.md), [docs/operations/deployment.md](../../docs/operations/deployment.md), [Langfuse 3.1.x](../../docs/operations/langfuse-3.1.md).
