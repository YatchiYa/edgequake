# Soak verdict — 0.22.0-pg16

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg16` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg16` |
| started | 2026-07-31T10:55:36Z |
| finished | 2026-07-31T10:59:27Z |
| wall seconds | 231 |
| PASS / FAIL | 20 / 0 |
| Postgres | `16.14 (Debian 16.14-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 137 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `d98f2b250deb…` (666535 bytes) |
| dump path | `artifacts/spec93-migration-assessment/pg16/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
