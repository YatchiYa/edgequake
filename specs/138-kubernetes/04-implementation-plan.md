# 04 — Implementation plan

> **Cross-refs:** [Hub](README.md) · [E2E matrix](05-e2e-test-matrix.md)

## Phases (completed in this PR)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| 1 | SPEC-138 doc pack | Done |
| 2 | `helm/edgequake` chart | Done |
| 3 | `helm/edgequake-stack` umbrella + Langfuse | Done |
| 4 | kind scripts + Makefile targets | Done |
| 5 | `spec138_kubernetes_e2e.sh` + shared lib | Done |
| 6 | EC mitigations (preStop, NetworkPolicy, mock LLM) | Done |
| 7 | `deployment.md` + `deploy/kubernetes/README.md` | Done |

## File checklist

```
deploy/kubernetes/
├── README.md
├── kind/kind-config.yaml
├── helm/edgequake/          (Chart + templates + values*)
├── helm/edgequake-stack/    (umbrella + values-kind)
├── scripts/*.sh
└── ci/spec138-kind-smoke.yml

specs/138-kubernetes/        (this pack)
scripts/langfuse_e2e_common.sh
```

## Acceptance

- [x] `helm template` renders without error
- [ ] `make spec138-kubernetes-proof` exits 0 (requires kind + ~16GB RAM)
- [x] Docs cross-linked
