# 05 — Implementation plan

> **Cross-refs:** [Architecture](03-architecture.md) · [Runbook](06-e2e-and-runbook.md)

Ordered work. **Do not skip isolation checks.**

## 0. Preflight (human + gcloud)

1. Confirm active account is `raphael.mansuy@elitizon.com` and project `saas-app-001`.
2. Create/raise billing budgets **\$50 USD** and **\$100 USD** (LAW-148-12).
3. Verify calculator estimate ([GCP calculator](https://cloud.google.com/products/calculator)) — all \$ UNCONFIRMED.
4. Confirm sibling Cloud Run/SQL still listed (we will not destroy them).

## 1. Spec pack (this directory)

High-signal docs only. Code SSOT is `deploy/gcp/`.

## 2. Terraform + overlay

| Path | Role |
|------|------|
| `deploy/gcp/terraform/` | VPC, VM, disk, snapshots, IAP firewall, secrets, GCS, WIF |
| `deploy/gcp/compose/` | Production Compose + Caddyfile |
| `deploy/gcp/scripts/startup.sh` | Docker CE, data disk, fetch artifacts, first `deploy.sh` |
| `deploy/gcp/scripts/deploy.sh` | Pull, migrate, up, HTTPS/health/`\dx` gates |
| `.github/workflows/deploy-gcp-edgequake.yml` | WIF + IAP SSH |

Backend: `gs://saas-app-001-tf-state/eq148/` (existing bucket, **new prefix**).

`google_project_service.disable_on_destroy = false` so destroy cannot disable Compute for sibling apps.

## 3. Apply (gated)

```bash
cd deploy/gcp/terraform
cp terraform.tfvars.example terraform.tfvars   # project_id already saas-app-001
terraform init
terraform plan
terraform apply
```

**Will create:** `edgequake-*` plus VM `elitizon-db`.  
**Will not create:** Cloud SQL, Cloud Run, GKE.  
**Will not modify:** leftover `edgequake-db-vm`, leftover Option B VPC, existing Cloud Run services (except the separate **key rotation** step below).

## 4. First boot

1. `terraform output ssh_iap_command`
2. Wait for `/var/log/edgequake-startup.log` to reach `DEPLOY_OK` or fail clearly
3. Set LLM secret if ingest is needed:
   `printf '%s' "$OPENAI_API_KEY" | gcloud secrets versions add edgequake-openai-api-key --data-file=-`
4. Re-run `deploy.sh` on the VM

## 5. Leftover key rotation (orthogonal, required)

On Cloud Run `edgequake-api` (old v0.5.4): remove `OPENAI_API_KEY` from the service env. Revoke the leaked provider key. Do **not** copy it into `edgequake-openai-api-key`.

## 6. Smoke

See [06-e2e-and-runbook.md](06-e2e-and-runbook.md). Record [measurements/](measurements/) without secrets.

## Destroy (this stack only)

```bash
terraform destroy   # edgequake-* + elitizon-db only
# Data disk auto_delete=false — delete PD + snapshots intentionally after backup
```

Never run destroy against a state file that contains sibling resources. This state’s prefix is `eq148/` only.

## Explicit non-goals this cut

GKE, Langfuse-in-cluster, Cloud Armor/LB, GPU, importing Option B leftovers.
