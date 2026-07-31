# SPEC-93 — v0.22.0 Migration Assessment (PG16 / PG17 / PG18)

> **Status:** ACTIVE  
> **Source pin:** [EdgeQuake v0.22.0](https://github.com/raphaelmansuy/edgequake/releases/tag/v0.22.0) (migrations ≤105, KV SSOT)  
> **Target:** HEAD (migrations 106–137, typed relational SSOT + irreversible drops 125/126/131)  
> **Proof command:** `make spec93-migration-assessment`

## Purpose

Formal, realistic, multi-tenant validation that operators can upgrade a published **v0.22.0** database to HEAD **with perfection** on each shipped Postgres major:

| PG major | GHCR postgres tag | AGE pin (release) |
| --- | --- | --- |
| 16 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg16` | AGE 1.6.x |
| 17 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg17` | AGE 1.7.x |
| 18 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg18` | AGE 1.8.x |

API image for seeding: `ghcr.io/raphaelmansuy/edgequake:0.22.0`.

## Documents

| File | Role |
| --- | --- |
| [00-charter.md](00-charter.md) | Scope, non-goals, inheritance |
| [01-test-protocol.md](01-test-protocol.md) | Step-by-step soak protocol |
| [02-acceptance-criteria.md](02-acceptance-criteria.md) | AC-M-01..08 |
| [03-execution-checklist.md](03-execution-checklist.md) | Operator / agent checklist |
| [reports/](reports/) | Per-major + matrix verdicts |

## How to run

```bash
# Full realism matrix (5 tenants × 3 workspaces × 40 docs = 600 docs × 3 majors)
make spec93-migration-assessment

# Single major
make spec93-migration-assessment-pg16
make spec93-migration-assessment-pg17
make spec93-migration-assessment-pg18

# Legacy smoke (tiny corpus, default PG tag) — still available
make spec091-upgrade-soak
```

Reports land under `specs/93-migration-assessment/reports/{pg16,pg17,pg18}/` and `reports/matrix-summary.md`.

## Related

- Ops runbook: [`docs/operations/spec091-upgrade-from-v0.22.0.md`](../../docs/operations/spec091-upgrade-from-v0.22.0.md)
- Migration engine: [`specs/091-simplify-data-layer/07-migration-engine.md`](../091-simplify-data-layer/07-migration-engine.md)
- RM acceptance (RM-AC-13): [`specs/091-simplify-data-layer/22-ingestion-migration-system-assessment.md`](../091-simplify-data-layer/22-ingestion-migration-system-assessment.md)
