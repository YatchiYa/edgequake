# SPEC-138 — Kubernetes Full Stack (EdgeQuake + Langfuse)

> **Trigger:** Production deployments need a reproducible Kubernetes path; Docker Compose does not scale to HA clusters.
> **Method:** First principles + Helm charts + kind E2E proof with OTLP trace delivery to in-cluster Langfuse.
> **Target cut:** **v0.26.2+** (charts ship with repo; images from GHCR).

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  deploy/kubernetes/helm/edgequake-stack — umbrella Helm chart.               │
│  edgequake namespace: web + API + postgres (GHCR images).                    │
│  langfuse namespace: Langfuse v4 via official langfuse-k8s Helm v2.          │
│                                                                              │
│  API exports OTLP/HTTP traces → Langfuse (SPEC-124 contract).                  │
│  E2E proof: make spec138-kubernetes-proof (kind + Playwright + API poll).     │
│  Stdout logs ≠ Langfuse — traces only (LAW-138-4).                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | EdgeQuake Helm chart | **Implemented** | [deploy/kubernetes/helm/edgequake](../../deploy/kubernetes/helm/edgequake/) |
| F2 | Umbrella stack + Langfuse | **Implemented** | [edgequake-stack](../../deploy/kubernetes/helm/edgequake-stack/) |
| F3 | kind E2E proof target | **Gates passed** | [measurements/SUMMARY.md](measurements/SUMMARY.md) |
| F4 | Trace delivery to Langfuse | **Verified** | E2E-138-09 (kind manual proof) |
| F5 | Dual Postgres isolation | **Locked** | LAW-138-3, [12-lens-database](12-lens-database.md) |
| E2E | Full proof chain | **Gates** | `make spec138-kubernetes-proof` |

## Document map

```ascii
 00-why / 00-architecture-data
   → 01-first-principles (LAW-138-1..8)
   → 02-cross-ref-matrix
   → 03-component-inventory
   → 04-implementation-plan
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-similar-specs
   → 09-ops-runbook
   → 10-lens-product-owner
   → 11-lens-fullstack
   → 12-lens-database
   → 13-lens-kubernetes
   → 14-lens-network
   → 15-lens-langfuse
   → 16-lens-ops
   → measurements/
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Orchestration | Helm 3.17+; Langfuse separate release in `langfuse` ns |
| Langfuse | In-cluster self-hosted via `langfuse/langfuse-k8s` v2 (bundled stores for kind) |
| E2E cluster | `kind` with nginx ingress |
| E2E LLM | `mock` + `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1` (v0.26+ test escape hatch) |
| Migrations | Helm post-install Job `edgequake migrate`; API never auto-migrates (LD-15) |
| Postgres init | `init-extensions.sql` on first PVC via ConfigMap |
| Observability | OTLP/HTTP traces to Langfuse (not stdout logs) |
| Postgres | Separate instances: EdgeQuake (`edgequake` ns) vs Langfuse (`langfuse` ns) |
| Proof | `make spec138-kubernetes-proof` |

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|-----------|
| [SPEC-124](../124-langfuse-support/) | OTLP export, Settings DTO, Playwright E2E |
| [SPEC-018](../018-observability/) | Prometheus, Jaeger gRPC (orthogonal) |
| [SPEC-027](../027-api-contract/) | Auth bootstrap env vars |
| [SPEC-057](../057-task-delivery/) | Multi-replica `EDGEQUAKE_TASK_DELIVERY` |
| [deployment.md](../../docs/operations/deployment.md) | Operator entry point |
| [OBSERVABILITY.md](../../docs/OBSERVABILITY.md) | Langfuse env reference |

## DRY rule

- **Helm SSOT:** `deploy/kubernetes/` — docs link here, never duplicate YAML in specs.
- **Langfuse E2E SSOT:** `scripts/langfuse_e2e_common.sh` — Docker and K8s proofs share it.
- **Playwright SSOT:** Reuse `spec124-langfuse-*.spec.ts`; do not fork.

## Out of scope

- Cloud IaC (EKS/GKE Terraform)
- Langfuse Cloud-only profile (bundled in-cluster is v1)
- In-cluster Ollama
- stdout log shipping to Langfuse

## Start here

1. [00-why.md](00-why.md)
2. [00-architecture-data.md](00-architecture-data.md)
3. [deploy/kubernetes/README.md](../../deploy/kubernetes/README.md)
4. [09-ops-runbook.md](09-ops-runbook.md)
