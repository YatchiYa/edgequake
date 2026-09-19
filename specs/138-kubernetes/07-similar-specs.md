# 07 — Similar specs

> **Cross-refs:** [Hub](README.md)

| Spec | Symptom / topic | Lesson for SPEC-138 |
|------|-----------------|---------------------|
| [SPEC-124](../124-langfuse-support/) | Langfuse OTLP export | Reuse env contract, Playwright, observation poll |
| [SPEC-018](../018-observability/) | Prometheus, Jaeger | Optional; independent of Langfuse K8s path |
| [SPEC-137](../137-issue-migration-25-to-26/) | Doc pack template | Hub + lenses + measurements pattern |
| [deployment.md](../../docs/operations/deployment.md) | K8s YAML snippets | Replaced by real Helm charts |
| [SPEC-148](../148-gcloud-hosting/) | Cheapest GCP host (GCE Compose + Terraform) | Fills the “Cloud IaC” out-of-scope item; does not replace Helm |

**Boundary:** SPEC-138 does not replace SPEC-124; it extends deployment surface to Kubernetes. SPEC-148 is the GCP Terraform path (Option A), not a Helm substitute.
