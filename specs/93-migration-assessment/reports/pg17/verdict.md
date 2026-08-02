# Soak verdict — 0.22.0-pg17

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg17` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg17` |
| started | 2026-08-02T12:26:32Z |
| finished | 2026-08-02T12:33:08Z |
| wall seconds | 396 |
| PASS / FAIL | 20 / 0 |
| Postgres | `17.10 (Debian 17.10-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 141 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `581e12c159ad…` (676943 bytes) |
| dump path | `/Users/raphaelmansuy/Github/03-working/edgequake/artifacts/spec93-migration-assessment/pg17/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
