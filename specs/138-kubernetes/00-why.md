# 00 — Why SPEC-138

## Trigger

EdgeQuake ships production-ready GHCR images and Docker Compose stacks, but **no deployable Kubernetes manifests**. Operators who need HA, namespace isolation, and GitOps cannot run EdgeQuake + Langfuse on K8s from this repo today.

## Product WHY

```ascii
  Operator: "Run EdgeQuake on our EKS cluster with LLM trace debugging."
       │
       ▼
  Today:
       make stack          → Docker Compose only
       deployment.md       → illustrative YAML, "Helm Coming Soon"
       make langfuse-up    → separate Compose project, not K8s
              │
              ▼
  Blind spot: no one-command K8s path; no proof traces reach Langfuse in-cluster
```

## Five WHYs

1. **Why Kubernetes?** Scale, HA, enterprise platform standards, namespace isolation.
2. **Why not only Compose?** Compose does not integrate with cluster ingress, PDB, NetworkPolicy, or GitOps.
3. **Why Langfuse in-cluster?** Self-hosted trace UI without Cloud egress; matches local dev init-key pattern.
4. **Why prove trace delivery?** SPEC-124 works in Docker; K8s DNS and secrets wiring is a new failure surface.
5. **Root cause:** Deployment investment stopped at GHCR + Compose; K8s was documented aspiration only.

## Job to be done

> `helm install edgequake-stack` on a kind or production cluster yields a healthy web + API + Postgres stack, with Langfuse receiving OTLP traces from the API, verifiable by automated E2E.

## Success criteria

1. Helm charts in `deploy/kubernetes/` install EdgeQuake + Langfuse.
2. `make spec138-kubernetes-proof` exits 0 on kind.
3. Langfuse Public API returns observations for a query `session_id`.
4. SPEC-138 doc pack with cross-ref lenses complete.

## Non-goals

- Managed cloud Terraform
- Langfuse Cloud-only deployment
- Shipping stdout logs to Langfuse

## Cross-refs

- Architecture: [00-architecture-data.md](00-architecture-data.md)
- Laws: [01-first-principles.md](01-first-principles.md)
- Proof: [05-e2e-test-matrix.md](05-e2e-test-matrix.md)
