# 00 — Environment assessment (`gcloud`, 2026-09-19)

> **Account:** `raphael.mansuy@elitizon.com` (active)  
> **Project:** `saas-app-001` · number **898717877025** · org `elitizon.com` (`284641495597`)  
> **Lifecycle:** ACTIVE · billing **enabled** (`billingAccounts/012EAA-D99E3E-5B4C06`)  
> **Isolation rule:** treat this project as a **shared tenancy**. SPEC-148 may only create `edgequake-*` objects plus VM `elitizon-db`. VPC is `edgequake-host-vpc` because leftover Option B already owns `edgequake-vpc`.

## Identity

| Fact | Value |
|------|--------|
| `gcloud config get-value project` | `saas-app-001` |
| Create time | 2025-09-20 |
| Label | `generative-language=enabled` |

## APIs already enabled (do not disable)

`compute`, `run`, `sqladmin`, `artifactregistry`, `secretmanager`, `storage`, `iam`, `iamcredentials`, `logging`, `monitoring`, `oslogin`, `cloudbuild`, `cloudresourcemanager`.

**Not enabled (Terraform will enable, `disable_on_destroy=false`):** `iap.googleapis.com`, `sts.googleapis.com` (WIF).

## Existing EdgeQuake leftovers (DO NOT TOUCH)

| Resource | State | Why leftover |
|----------|-------|----------------|
| GCE `edgequake-db-vm` (`us-central1-a`, `e2-standard-2`) | **TERMINATED** | Custom `postgres:16` + AGE build; `--network host`; DB `graph_db`; not GHCR |
| Disk `edgequake-data-disk` 50 GiB `pd-standard` | READY, attached | Nightly snapshots exist |
| VPC `edgequake-vpc` + firewall `edgequake-allow-cloud-run-to-db` :5432 from `10.8.0.0/28` | live | Option B Cloud Run → GCE Postgres |
| Cloud Run `edgequake-api` / `edgequake-webui` | Ready | Image pin **v0.5.4**; `DATABASE_URL` to `10.0.0.12:5432/graph_db`; **plaintext `OPENAI_API_KEY` in env — rotate at provider; do not copy** |
| SA `edgequake-vm@`, `edgequake-cloud-run@` | live | Old Option B identities |
| AR repo `edgequake-images` | live | Old Cloud Run images |

## Sibling apps in the same project (DO NOT TOUCH)

Cloud Run: `edgewhisper-website`, `hk-smartfolio`, `nutrition-coach`, `saas-app-001-api`, `saas-app-001-web`.  
Cloud SQL: `saas-app-001-db` (PG15 `db-f1-micro`), `edgewhisper-direct-db` (PG16).  
GCS: `saas-app-001-tf-state` (reuse as **backend prefix `eq148/` only**), `saas-app-001-documents`, others.  
Default VPC still has `default-allow-ssh` **`0.0.0.0/0` :22** — another reason this stack uses a **new** VPC (`edgequake-host-vpc`, not leftover `edgequake-vpc`).

## Identity gaps

| Gap | Action |
|-----|--------|
| No Workload Identity Pool for this stack | Create `edgequake-github` pool + GitHub OIDC provider |
| `github-actions@saas-app-001` exists | Do **not** mint a JSON key; bind WIF to a **new** `edgequake-github@` SA |
| IAP API off | Enable via Terraform |

## Budget

Existing alert: **\$5 HKD / month**. Option A infra is ~**\$35–45 USD/mo UNCONFIRMED**. Raise **\$50 and \$100 USD** alerts on this project before first apply ([07-cost-risks](07-cost-risks.md)). Google budgets **alert**; they do not always stop VMs — still treat \$5 HKD as a paging trap.

## AGE feasibility (official, same day)

| Platform | pgvector / vector | Apache AGE | Verdict |
|----------|-------------------|------------|---------|
| Cloud SQL Postgres | Yes (`pgvector` 0.8.5 on PG13+) | **Not listed** | Unusable as sole DB |
| AlloyDB | Yes (`vector` / ScaNN) | **Not listed** | Unusable as sole DB |
| GCE + GHCR `edgequake-postgres` | Yes | Yes | **Required path** |

Sources: [Cloud SQL extensions](https://docs.cloud.google.com/sql/docs/postgres/extensions) · [AlloyDB extensions](https://docs.cloud.google.com/alloydb/docs/reference/extensions) · [AGE discussion #2224](https://github.com/apache/age/discussions/2224).

## Commands to re-verify (read-only)

```bash
gcloud config get-value account
gcloud config get-value project
gcloud projects describe saas-app-001 --format='yaml(projectId,projectNumber,lifecycleState,parent)'
gcloud compute instances list --project=saas-app-001
gcloud run services list --project=saas-app-001 --region=us-central1
gcloud services list --enabled --project=saas-app-001
```
