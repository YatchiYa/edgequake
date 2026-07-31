# Soak verdict — 0.22.0-pg17

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg17` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg17` |
| started | 2026-07-31T10:59:28Z |
| finished | 2026-07-31T11:04:03Z |
| wall seconds | 275 |
| PASS / FAIL | 20 / 0 |
| Postgres | `17.10 (Debian 17.10-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 137 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `62df3f13e298…` (669588 bytes) |
| dump path | `artifacts/spec93-migration-assessment/pg17/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
