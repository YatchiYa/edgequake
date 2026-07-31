# Soak verdict — 0.22.0-pg18

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg18` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg18` |
| started | 2026-07-31T11:04:04Z |
| finished | 2026-07-31T11:10:23Z |
| wall seconds | 379 |
| PASS / FAIL | 20 / 0 |
| Postgres | `18.4 (Debian 18.4-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 137 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `103997c4490e…` (677115 bytes) |
| dump path | `artifacts/spec93-migration-assessment/pg18/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
