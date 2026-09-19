# 02 — Cross-ref matrix

> **Cross-refs:** [Laws](01-first-principles.md) · [Hub](README.md)

## This pack → code → official → test

| Law | Code / artifact | Official / product authority | Test |
|-----|-----------------|------------------------------|------|
| LAW-148-1 | No `google_sql_*` in `deploy/gcp/terraform` | [Cloud SQL extensions](https://docs.cloud.google.com/sql/docs/postgres/extensions) · [AlloyDB extensions](https://docs.cloud.google.com/alloydb/docs/reference/extensions) | `rg google_sql deploy/gcp/terraform` empty; E2E `\dx` on GCE |
| LAW-148-2 | Resource names `edgequake-*` + VM `elitizon-db`; GCS backend prefix `eq148/` | [gcloud assessment](00-env-assessment.md) | `terraform state list` is this stack only; sibling Cloud Run still listed |
| LAW-148-3 | `scheduling { provisioning_model = "STANDARD" preemptible = false }` | [Spot VMs](https://cloud.google.com/spot-vms/docs/preemptible) | `gcloud compute instances describe` |
| LAW-148-4 | `EDGEQUAKE_VERSION=0.26.5` in compose overlay | [GHCR release CD](../../docs/operations/release-and-cd.md) | `docker compose images` pins |
| LAW-148-5 | `deploy/gcp/scripts/deploy.sh` migrate then up | LD-15 · [migrate-job.yaml](../../deploy/kubernetes/helm/edgequake/templates/migrate-job.yaml) · `edgequake/src/main.rs` | API boot after migrate; no silent schema mutate |
| LAW-148-6 | `google_secret_manager_secret`; startup `gcloud secrets versions access` | [Secret Manager](https://cloud.google.com/secret-manager/docs/creating-and-accessing-secrets) | `.env` mode 600; not in git |
| LAW-148-7 | Firewall source `35.235.240.0/20` tcp/22 | [IAP TCP forwarding](https://docs.cloud.google.com/iap/docs/using-tcp-forwarding) · [OS Login](https://docs.cloud.google.com/compute/docs/oslogin/set-up-oslogin) | `gcloud compute firewall-rules describe edgequake-allow-iap-ssh` |
| LAW-148-8 | Compose: postgres has no `ports:` | Notion Option A risk table | `ss -lntp` on VM has no 5432 public |
| LAW-148-9 | Compose env `EDGEQUAKE_DEV_MODE=false` | [runtime-auth-hardening.md](../../docs/operations/runtime-auth-hardening.md) | Login required |
| LAW-148-10 | `EDGEQUAKE_API_URL=https://$PUBLIC_ORIGIN` | [resolve-runtime-api-url.ts](../../edgequake_webui/src/lib/server/resolve-runtime-api-url.ts) · Helm `web.apiUrl` | UI talks HTTPS same origin via Caddy |
| LAW-148-11 | `deploy/gcp/terraform/*.tf` | [Terraform get started](https://cloud.google.com/docs/terraform/get-started-with-terraform) · [`google_compute_instance`](https://registry.terraform.io/providers/hashicorp/google/latest/docs/resources/compute_instance) | `terraform validate` |
| LAW-148-12 | Runbook budget step | [Billing budgets](https://cloud.google.com/billing/docs/how-to/budgets) · [Calculator](https://cloud.google.com/products/calculator) | `gcloud billing budgets list` |
| LAW-148-13 | `deploy/gcp/compose/Caddyfile` `:80` redir | [Caddy automatic HTTPS](https://caddyserver.com/docs/automatic-https) | `curl -I http://$IP/` → 301/308; HTTP 200 is **fail** |

## Other specs

| Spec | Relationship |
|------|----------------|
| [SPEC-138](../138-kubernetes/) | Helm/K8s path; **explicitly** left cloud IaC out of scope — this spec fills it |
| [SPEC-091](../091-simplify-data-layer/) | LD-15 boot gate; `edgequake migrate` SSOT |
| [SPEC-027](../027-api-contract/) | Auth / bootstrap env |
| [SPEC-124](../124-langfuse-support/) | Optional later; **not** in Option A compose |
| [SPEC-057](../057-task-delivery/) | Single-replica API on one VM; no bridged delivery required |

## Divergence rule

If this matrix and Terraform/Compose disagree, **Terraform + Compose + smoke gate** win. Update this file in the same PR.
