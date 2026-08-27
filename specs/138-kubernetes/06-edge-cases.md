# 06 — Edge cases

> **Cross-refs:** [E2E matrix](05-e2e-test-matrix.md) · [Laws](01-first-principles.md)

| EC | Case | Mitigation | Test |
|----|------|------------|------|
| EC1 | `LANGFUSE_BASE_URL=localhost` in pod | ConfigMap uses cluster DNS | E2E-138-08 |
| EC2 | OTLP gRPC to Langfuse | Document HTTP-only; no gRPC endpoint to LF | `langfuse.rs` unit tests |
| EC3 | Langfuse not ready at API start | OTLP retries; `/ready` independent of LF | E2E-138-14 |
| EC4 | Init keys mismatch | Fixed keys in stack values; bootstrap Secret | E2E-138-07/08 |
| EC5 | Browser can't reach API | Ingress + external `EDGEQUAKE_API_URL` in prod | E2E-138-06 |
| EC6 | Postgres PVC pending | Document StorageClass; kind default | E2E-138-04 |
| EC7 | Trace flush on SIGTERM | `preStop` sleep 15s + ObservabilityGuard | E2E-138-14 |
| EC8 | Shared Postgres | Separate namespaces (LAW-138-3) | Lens DB |
| EC9 | kind OOM | `values-kind.yaml` low resources; RAM warning | README |
| EC10 | Multi-replica API | `EDGEQUAKE_TASK_DELIVERY=bridged` | Ops lens |
| EC11 | No LLM in cluster | `EDGEQUAKE_LLM_PROVIDER=mock` for kind | E2E-138-09 |
| EC12 | Missing ClickHouse operator | `k8s_prereqs.sh` preflight | E2E-138-01 |
| EC13 | NetworkPolicy blocks OTLP | Egress rule to langfuse ns | E2E-138-13 (optional) |
| EC14 | Langfuse web OOM on kind | `langfuse.web.resources` + `NODE_OPTIONS=--max-old-space-size=1536` | Manual kind proof |
| EC15 | Mock LLM forbidden in v0.26 | `EDGEQUAKE_ALLOW_MOCK_PROVIDER=1` in kind values | E2E-138-09 |
| EC16 | Boot refuses without migrate | Helm `migrate-job` post-install hook | E2E-138-05 |
| EC17 | Postgres extensions missing | `init-extensions.sql` via ConfigMap initdb mount | E2E-138-04 |
