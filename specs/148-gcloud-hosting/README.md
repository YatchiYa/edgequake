# SPEC-148 — Fresh cheapest EdgeQuake host on GCP (Option A)

> **Trigger:** Host the real product (AGE + pgvector + Axum + workers) on GCP without Cloud SQL/AlloyDB, without touching other apps in `saas-app-001`.
> **Method:** First principles + Terraform SSOT + GHCR Compose overlay + IAP SSH + WIF deploy.
> **Target cut:** **v0.26.5** (pinned GHCR tags). Project **saas-app-001** / **898717877025**.
> **Notion hub:** [SPEC: Cheapest EdgeQuake hosting on Google Cloud](https://app.notion.com/p/3e0887c3416481a682b6d134c4e00b29)

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  deploy/gcp — Terraform Option A (VM elitizon-db, objects edgequake-*).      │
│  GCE e2-medium + Docker Compose: postgres (AGE+pgvector) + API + web + Caddy │
│  Caddy :80 → 301/308 HTTPS only. App never served on plaintext HTTP.         │
│  Images: ghcr.io/raphaelmansuy/edgequake{,-frontend,-postgres}:0.26.5        │
│  Deploy: GitHub Actions WIF → IAP SSH → scripts/deploy.sh (LD-15 migrate)    │
│  Isolation: do NOT import/destroy edgequake-db-vm, Cloud Run, or other apps. │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| ID | Item | Verdict | Evidence |
|----|------|---------|----------|
| F1 | Spec pack | **This directory** | [02-cross-ref-matrix](02-cross-ref-matrix.md) |
| F2 | Terraform SSOT | **`deploy/gcp/terraform/`** | [LAW-148-11](01-first-principles.md) |
| F3 | Compose overlay + Caddy HTTPS | **`deploy/gcp/compose/`** | LAW-148-13 |
| F4 | WIF + IAP deploy | **`.github/workflows/deploy-gcp-edgequake.yml`** | [04-security-and-cd](04-security-and-cd.md) |
| F5 | Fresh isolation | **edgequake-* + elitizon-db** | [00-env-assessment](00-env-assessment.md) |
| APPLY | `terraform apply` | **Applied 2026-09-19** — VM `elitizon-db`, objects `edgequake-*`, VPC `edgequake-host-vpc` | [measurements/](measurements/) |

## Document map

```ascii
 README
   → 00-why
   → 00-env-assessment   (gcloud facts 2026-09-19)
   → 01-first-principles (LAW-148-1..13)
   → 02-cross-ref-matrix
   → 03-architecture
   → 04-security-and-cd
   → 05-implementation-plan
   → 06-e2e-and-runbook
   → 07-cost-risks
   → measurements/       (post-apply; no secrets)
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Shape | Notion **Option A** — one GCE VM + Compose. Not Cloud SQL, AlloyDB, GKE, Cloud Run |
| Isolation | VM **`elitizon-db`**; bucket/VPC/secrets/WIF **`edgequake-*`**; never import sibling leftovers |
| Images | GHCR pin **0.26.5** (never `latest`) |
| HTTPS | Caddy `:80` redirect-only; app on `:443`; `tls internal` until hostname |
| SSH | IAP TCP `35.235.240.0/20` + OS Login; no `0.0.0.0/0` on :22 |
| Migrate | Explicit `edgequake migrate` (LD-15); API never auto-migrates |
| CD | Workload Identity Federation; no JSON SA keys |
| Auth | `EDGEQUAKE_DEV_MODE=false`, `EDGEQUAKE_AUTH_ENABLED=true` |

## DRY rule

- **Terraform SSOT:** [`deploy/gcp/`](../../deploy/gcp/) — specs link, never duplicate HCL.
- **Compose semantics:** overlay of [`docker-compose.quickstart.yml`](../../docker-compose.quickstart.yml); do not fork a second product contract.
- **K8s remains SPEC-138:** [`deploy/kubernetes/`](../../deploy/kubernetes/) — this spec does not replace Helm.

## Out of scope

- Importing or destroying `edgequake-db-vm`, `edgequake-vpc`, Cloud Run `edgequake-api`/`edgequake-webui` (v0.5.4), Cloud SQL, or other saas-app-001 apps
- GKE Autopilot, Cloud Armor, Cloud Load Balancing, GPUs
- In-cluster Langfuse (SPEC-138)
- Spot VMs for Postgres

## Start here

1. [00-why.md](00-why.md)
2. [00-env-assessment.md](00-env-assessment.md)
3. [deploy/gcp/README.md](../../deploy/gcp/README.md)
4. [06-e2e-and-runbook.md](06-e2e-and-runbook.md)
