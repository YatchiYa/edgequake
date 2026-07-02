# Database Migrations

EdgeQuake uses [SQLx](https://github.com/launchbadge/sqlx) embedded migrations. On server start with PostgreSQL, pending migrations in `edgequake/migrations/` are applied automatically.

## Quick Reference

| Task | Command |
|------|---------|
| Fresh dev stack | `make dev` (migrations run on backend start) |
| Check migration immutability | `./scripts/check_migration_checksums.sh` |
| Resource safety proofs | `make resource-proof` |
| Migration 038 (prod ops) | `edgequake/scripts/migrations/apply_038.sh --help` |

## How Migrations Work

1. **Numbered SQL files** — `NNN_description.sql` in `edgequake/migrations/`
2. **Bootstrap auto-apply** — `migration_bootstrap::run_postgres_migrations()` on API start (progression logs + post-hooks)
3. **Immutability lock** — `edgequake/migrations/checksums.lock` prevents editing deployed migrations
4. **Support scripts** — `edgequake/migrations/support/` holds ops-only SQL (not picked up by sqlx)

### Bootstrap behavior (first principles)

| Principle | Implementation |
|-----------|----------------|
| Idempotent | sqlx + `CREATE INDEX IF NOT EXISTS`; safe restart |
| Observable | Structured `edgequake.migration` logs per step/graph |
| Workload-safe | Graphs ≥500k vertices: defer blocking index build; ops use `--concurrent` |
| Non-fatal | Missing AGE → skip 038 verify; server still starts |
| Verifiable | `/health` → `schema.source_ids_indexes` reports readiness |

**Env vars:**

| Variable | Default | Purpose |
|----------|---------|---------|
| `EDGEQUAKE_MIGRATION_LARGE_GRAPH_THRESHOLD` | `500000` | Defer inline index repair above this vertex count |

**Log target:** filter with `RUST_LOG=edgequake.migration=info` for migration-only progression output.

### Rules

- **Never edit** a migration that has been deployed to production. Add a new numbered file instead.
- **Append checksum** when adding a migration: `./scripts/update_migration_checksums.sh`
- **Auxiliary SQL** (preflight, rollback, CONCURRENTLY) lives under `migrations/support/` — not in the sqlx scan path.

## Migration 038 — source_ids Indexes (SPEC-006)

Improves bounded document delete, lineage, and relationship lookups on large AGE graphs.

| File | Role |
|------|------|
| `038_add_source_ids_gin_indexes.sql` | sqlx version marker (no blocking DDL) |
| `support/038/apply.sql` | Size-aware index SSOT (bootstrap + ops) |
| `support/038/preflight.sql` | Read-only preflight |
| `support/038/concurrent.sql` | Zero-downtime for large graphs |
| `support/038/rollback.sql` | Drop indexes only |
| `support/038/verify.sql` | Post-apply verification |

**Full guide:** [migrations/038-source-ids-indexes.md](migrations/038-source-ids-indexes.md)

```bash
export DATABASE_URL="postgres://edgequake:edgequake@localhost/edgequake"
./edgequake/scripts/migrations/apply_038.sh --dry-run
./edgequake/scripts/migrations/apply_038.sh --apply --yes
./edgequake/scripts/migrations/apply_038.sh --verify
```

## Migration 078 — Child Workspace Stats Indexes (SPEC-040 / #262)

Repairs AGE child-table indexes for workspace-scoped graph stats (fixes 15s timeout / nested-loop plans on large graphs).

| File | Role |
|------|------|
| `078_age_child_workspace_stats.sql` | sqlx migration — auto-applied on `make dev` / backend start |
| `support/078/concurrent.sql` | Ops-only CONCURRENTLY build for graphs >100k nodes |
| `specs/040-edgequake-issues/e2e/measure_graph_stats_perf.sh` | Performance proof |

**Full guide:** [migrations/078-age-child-workspace-stats.md](migrations/078-age-child-workspace-stats.md)

```bash
# Verify auto-deploy (local)
psql "$DATABASE_URL" -c "SELECT version, description FROM _sqlx_migrations WHERE version = 78;"

# Measure performance post-M078
./specs/040-edgequake-issues/e2e/measure_graph_stats_perf.sh

# Production large graph (manual, outside transaction)
psql "$DATABASE_URL" -f edgequake/migrations/support/078/concurrent.sql
```

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `migration N was previously applied but has been modified` | Restore canonical SQL or create new migration; never edit deployed files |
| Backend fails on migrate | Check `DATABASE_URL`, PostgreSQL version, AGE extension |
| Slow stats / graph timeout on large workspace | Apply migration 078 (auto on upgrade); verify with `measure_graph_stats_perf.sh`; use `support/078/concurrent.sql` if >100k nodes |
| OOM on list/delete (exit 137) | See [SPEC-006](../../specifications/006-ensure-perf/010-brutal-assessment.md); run `make resource-proof` |

## Related Docs

- [Runbook](runbook.md) — production operations
- [Getting Started](getting-started.md) — local setup
- [SPEC-006 specification](../../specifications/006-ensure-perf/000-index.md)
