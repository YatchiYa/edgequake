# 00 — Charter

## Mission

Prove that the EdgeQuake **migration system** can upgrade a realistic multi-tenant, multi-workspace corpus from published **v0.22.0** to HEAD on **PostgreSQL 16, 17, and 18** without data loss, isolation breach, or post-drop KV/vector regressions.

## In scope

1. Same-major stay: seed on `edgequake:0.22.0` + `edgequake-postgres:0.22.0-pg{16,17,18}`, then apply HEAD migrations 106–137 via `edgequake migrate --confirm-drop`.
2. Realistic corpus defaults: **5 tenants × 3 workspaces × 40 documents = 600 documents** (mock LLM/embeddings).
3. Operator path: dry-run (no ledger advance) → migrate without confirm (refuse / expandable-first) → `--confirm-drop` → HEAD API boot (LD-15 verify-only).
4. Post-upgrade gates: isolation, wipe sibling integrity, assets non-500, fence-on query non-500, ledger through 137, zero `eq_%_kv`.
5. Published reports under `specs/93-migration-assessment/reports/`.

## Out of scope

| Item | Why |
| --- | --- |
| Cross-major `pg_upgrade` (16→18) | Separate infra path (`scripts/migrate_postgres_major.sh`) |
| 100k+ ANN recall soak | Deferred in SPEC-091 RM4 |
| Production dump restore automation | Documented manual path in ops runbook |
| PR-blocking CI for 600-doc matrix | Formal proof is matrix report; optional nightly later |

## Inheritance

| Source | What we reuse |
| --- | --- |
| SPEC-091 Waves A–D / RM0–RM5 | Schema train 106–137, irreversible set 125/126/131 |
| `scripts/spec091_upgrade_soak.sh` | Harness (generalized for SPEC-93 profiles) |
| `docker-compose.spec091-soak.yml` | Disposable API + Postgres stack |
| `docs/operations/spec091-upgrade-from-v0.22.0.md` | Operator sequence SSOT |

## Success definition

**AC-M-08 GREEN:** all three majors (`pg16`, `pg17`, `pg18`) produce `verdict.md` with `status: GREEN` and `reports/matrix-summary.md` records overall PASS.
