# 01 — First principles (LAW-148)

> **Cross-refs:** [WHY](00-why.md) · [Assessment](00-env-assessment.md) · [Matrix](02-cross-ref-matrix.md)

## Axioms

1. **One Postgres.** KV + pgvector + AGE share one instance ([data-layer](../../docs/deep-dives/data-layer.md)).
2. **Managed GCP SQL is incomplete** until `age` appears on the official extension lists.
3. **Shared project.** `saas-app-001` is not a greenfield EdgeQuake project.
4. **Compose semantics.** Production overlay matches [`docker-compose.quickstart.yml`](../../docker-compose.quickstart.yml) plus auth-on and Caddy.
5. **HTTPS is the only origin.** `:80` redirects; the app is not served in plaintext.

## Causal diagram

```text
  terraform apply (edgequake-* + VM elitizon-db)
           │
           ├─ VPC + IAP firewall + static IP
           ├─ e2-medium (STANDARD, never Spot) + pd-balanced + snapshots
           ├─ Secret Manager + GCS artifacts
           └─ WIF pool for GitHub
                    │
                    ▼
  startup.sh → Docker CE → compose postgres → edgequake migrate
                    │
                    ▼
  api + frontend + Caddy (:80 301 → :443)
                    │
                    ▼
  GitHub Actions WIF → IAP SSH → deploy.sh (pull, migrate, health, \dx)
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-148-1** | **AGE gate** — Cloud SQL / AlloyDB MUST NOT be the sole EdgeQuake database until `age` is on the official supported-extension lists. |
| **LAW-148-2** | **Fresh isolation** — Terraform manages only `edgequake-*` plus VM `elitizon-db`. Never import, start, or `terraform destroy` sibling resources (`edgequake-db-vm`, leftover `edgequake-vpc`, Cloud Run apps, Cloud SQL, default VPC). |
| **LAW-148-3** | **No Spot for Postgres** — This VM and its data disk are `STANDARD` / `preemptible=false`. |
| **LAW-148-4** | **Pin images** — GHCR tags equal product `VERSION` (0.26.5). Never `latest` in this overlay. |
| **LAW-148-5** | **LD-15** — API never auto-migrates. Deploy runs `edgequake migrate dry-run` then `edgequake migrate` then start. Boot refuse (exit 78) if schema is behind. |
| **LAW-148-6** | **Secrets** — Secret Manager + VM SA `secretmanager.secretAccessor`. No secrets in git, instance metadata, or Cloud Run plaintext env. |
| **LAW-148-7** | **IAP SSH** — Ingress `:22` only from `35.235.240.0/20`. OS Login `enable-oslogin=TRUE` on **this instance**. No `0.0.0.0/0` SSH on `edgequake-host-vpc`. |
| **LAW-148-8** | **Postgres private** — No host publish of `:5432`. Docker network only. |
| **LAW-148-9** | **Prod auth** — `EDGEQUAKE_DEV_MODE=false`, `EDGEQUAKE_AUTH_ENABLED=true`. Quickstart open-API defaults are forbidden here. |
| **LAW-148-10** | **Runtime API URL** — Browser uses `EDGEQUAKE_API_URL=https://$PUBLIC_ORIGIN` (not baked `NEXT_PUBLIC_*`). |
| **LAW-148-11** | **Terraform SSOT** — All GCP resources live under `deploy/gcp/`. Specs link, never duplicate HCL. |
| **LAW-148-12** | **Budgets** — \$50 and \$100 USD alerts on this project before first apply. Dollar figures in docs are **UNCONFIRMED**. |
| **LAW-148-13** | **HTTPS redirect** — Caddy `:80` is redirect-only (`redir … permanent`). Reverse-proxy exists only on `:443`. Fail the gate if `curl -I http://$ORIGIN/` is not 301/308 with `Location: https://…`. Until DNS: `tls internal` (browser warning expected). **No HSTS** until Let’s Encrypt on a real hostname. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | Terraform = cloud; Compose = process graph; Caddy = TLS/edge; `deploy.sh` = release procedure |
| **O** | Hostname tfvar switches Caddy from `tls internal` to automatic HTTPS without changing the VM shape |
| **L** | GHCR images are the same artifacts as `make stack` / Helm |
| **I** | WIF deploy job does **not** run `terraform apply` on first landing |
| **DRY** | Quickstart compose contract; Helm migrate command `edgequake migrate` |

## Kill switches (first principles)

- Stop the VM when idle (dev) — disk/snapshots still bill.
- Never enable Cloud SQL as default until LAW-148-1 flips.
- Infra > \$80/mo without paying users → revisit Option A ([07-cost-risks](07-cost-risks.md)).
