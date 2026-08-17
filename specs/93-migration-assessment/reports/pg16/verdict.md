# Soak verdict — 0.22.0-pg16

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg16` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg16` |
| started | 2026-08-02T12:25:06Z |
| finished | 2026-08-02T12:26:30Z |
| wall seconds | 84 |
| PASS / FAIL | 20 / 0 |
| Postgres | `16.14 (Debian 16.14-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 141 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `99706454b81a…` (653531 bytes) |
| dump path | `/Users/raphaelmansuy/Github/03-working/edgequake/artifacts/spec93-migration-assessment/pg16/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
