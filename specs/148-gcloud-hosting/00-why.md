# 00 — Why SPEC-148

## Trigger

EdgeQuake’s data layer is **one PostgreSQL** holding KV + **pgvector** + **Apache AGE** + relational sidecars. Managed Cloud SQL and AlloyDB list vector search extensions but **not** Apache AGE, so they cannot be the sole database. Premature GKE Autopilot + AlloyDB is an order of magnitude above a quiet POC bill. SPEC-138 shipped Helm but explicitly left **cloud IaC** out of scope.

Existing leftovers in `saas-app-001` (Cloud Run v0.5.4 + terminated `edgequake-db-vm` with a custom AGE image) are a **different architecture**, an old pin, and store secrets in plaintext env. They must not be “upgraded in place.”

## Product WHY

```ascii
  Operator: "Run real EdgeQuake on GCP cheaply, update it securely."
       │
       ▼
  Blocked paths:
       Cloud Run + Cloud SQL     → AGE not in Cloud SQL extensions
       AlloyDB                   → AGE not in AlloyDB extensions
       GKE Autopilot (SPEC-138)  → correct later; not cheapest
       Reuse edgequake-db-vm     → Option B leftover, TERMINATED, not GHCR SSOT
              │
              ▼
  Required path: self-managed Postgres (GHCR edgequake-postgres) on GCE
                 + pinned Compose + Terraform + IAP/WIF deploy
```

## Five WHYs

1. **Why GCP at all?** The operator account is `saas-app-001` / `898717877025`; Vertex/GCS already live there.
2. **Why not Cloud SQL?** Official extension list includes `pgvector`, not `age` ([docs](https://docs.cloud.google.com/sql/docs/postgres/extensions)).
3. **Why not the existing Cloud Run stack?** It is v0.5.4, Option B (API scale-to-zero fights lease workers), DB VM is TERMINATED, and LLM keys are in plaintext env.
4. **Why one VM?** Notion Option A: quiet POC ~\$25–40/mo UNCONFIRMED; workers stay long-lived; AGE under our control.
5. **Root cause:** Product storage assumes **one Postgres**. Managed GCP Postgres is incomplete until Google allow-lists AGE ([issuetracker 288350484](https://issuetracker.google.com/issues/288350484)).

## Job to be done

> `terraform apply` in `deploy/gcp/terraform` yields a fresh e2-medium host whose Compose stack serves EdgeQuake **only over HTTPS**, with IAP SSH, Secret Manager, pinned GHCR v0.26.5, and a keyless GitHub Actions deploy that runs LD-15 migrate then `/health`.

## Success criteria

1. Spec pack with laws cross-referenced to official docs + repo SSOT.
2. Terraform creates **only** `edgequake-*` objects plus VM `elitizon-db` in `saas-app-001` (VPC `edgequake-host-vpc`; leftover Option B keeps `edgequake-vpc`).
3. HTTP `:80` never returns app HTML/JSON — only 301/308 to HTTPS.
4. `\dx` shows **vector** and **age**; `/health` is healthy on HTTPS.
5. Deploy path uses WIF + IAP; no JSON keys in the repo.

## Non-goals

- Multi-region DR, SOC2 shared tenancy, GPU embedding farms
- Destroying sibling apps in the shared project
- Making Cloud SQL the default

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Assessment: [00-env-assessment.md](00-env-assessment.md)
- Notion: [Cheapest host SPEC](https://app.notion.com/p/3e0887c3416481a682b6d134c4e00b29)
