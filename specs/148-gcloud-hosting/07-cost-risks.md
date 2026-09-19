# 07 — Cost, risks, similar specs

> All **\$** figures are **UNCONFIRMED**. Verify in the [GCP Pricing Calculator](https://cloud.google.com/products/calculator) before purchase. As-of 2026-09-19 (Notion + public price pages).

## BOM (quiet POC, us-central1)

| Item | Spec | Est. USD/mo | Link |
|------|------|-------------|------|
| GCE VM | `e2-medium` on-demand | ~24–25 | [VM pricing](https://cloud.google.com/compute/vm-instance-pricing) |
| Persistent Disk | 50 GiB `pd-balanced` | ~7–10 | [Disk pricing](https://cloud.google.com/compute/docs/disks/pricing) |
| Snapshots | nightly, 14d | ~1–3 | [Scheduled snapshots](https://cloud.google.com/compute/docs/disks/scheduled-snapshots) |
| Static IP | 1 attached | ~3–4 | [Network pricing](https://cloud.google.com/vpc/network-pricing) |
| GCS / Secret Manager / AR (not used for runtime images) | Always Free envelope | ~0 | [Free tier](https://cloud.google.com/free/docs/free-cloud-features) |
| Caddy HTTPS | on-VM | 0 (LB would be ~18+) | — |
| **Infra subtotal** | | **~35–45** | |
| LLM/embeddings | external APIs | **variable, often dominates** | budget separately |

Always Free `e2-micro` is **too small**. `e2-small` (2 GiB) is an OOM risk — not the default.

Existing project alert is **\$5 HKD**. LAW-148-12: add **\$50** and **\$100 USD** alerts.

## Risks / kill criteria

| Risk | Mitigation | Kill / pivot |
|------|------------|--------------|
| AGE never on Cloud SQL | Track [288350484](https://issuetracker.google.com/issues/288350484); stay on GCE | Azure Flexible Server (AGE upstream) if multi-cloud |
| Disk corruption | Snapshots + optional `pg_dump` to GCS | Restore drill; halt ingest |
| OOM | Stay on e2-medium | Split API/DB (Option B) |
| Spot on DB | Terraform `preemptible=false` | Restore |
| LLM bill | Caps; secret unset until needed | Pause provider keys |
| Open 5432 | No compose `ports` | Immediate firewall audit |
| Sibling blast radius | `edgequake-*` + VM `elitizon-db` + state prefix `eq148/` | Never widen this state |
| HTTP plaintext | LAW-148-13 e2e | Fail deploy |

**Kill this hosting approach if:** monthly infra **> \$80** without paying users; restore untested **> 90 days**; AGE break on PG major with no rollback snapshot.

## Scaling path (Notion)

```text
A: All-in-one GCE          ← this spec
  -> B: Cloud Run + GCE Postgres (AGE)   when API bursty, workers stay on GCE
    -> C: GKE Autopilot (SPEC-138 charts) when HA/GitOps demanded
      -> Managed Cloud SQL/AlloyDB only if Google adds AGE
```

## Similar specs

| Spec | Lesson |
|------|--------|
| [SPEC-138](../138-kubernetes/) | Helm SSOT, LD-15 migrate Job, GHCR images; left cloud IaC out of scope |
| [SPEC-091](../091-simplify-data-layer/) | Boot never silent-migrates |
| [SPEC-027](../027-api-contract/) | Prod auth env |
| Notion Option A children | Terraform tree originally `/workspace/edgequake-gcp/` — **in-repo SSOT is `deploy/gcp/`** |

## Official links (pricing / feasibility)

- [Cloud SQL extensions](https://docs.cloud.google.com/sql/docs/postgres/extensions)
- [AlloyDB extensions](https://docs.cloud.google.com/alloydb/docs/reference/extensions)
- [Free cloud features](https://cloud.google.com/free/docs/free-cloud-features)
- [Cloud Run pricing](https://cloud.google.com/run/pricing) (Option B later)
- [VM instance pricing](https://cloud.google.com/compute/vm-instance-pricing)
- [Spot pricing](https://cloud.google.com/spot-vms/pricing) — rebuildable workers only
