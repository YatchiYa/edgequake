# Soak verdict — 0.22.0-pg18

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `realism` |
| postgres tag | `0.22.0-pg18` |
| source API | `edgequake:0.22.0` |
| compose project | `spec93soak-pg18` |
| started | 2026-08-02T12:33:10Z |
| finished | 2026-08-02T12:38:31Z |
| wall seconds | 321 |
| PASS / FAIL | 20 / 0 |
| Postgres | `18.4 (Debian 18.4-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 141 |
| tenants / workspaces / docs | 5 / 15 / 600 |
| dump SHA256 | `657b8a826429…` (674479 bytes) |
| dump path | `/Users/raphaelmansuy/Github/03-working/edgequake/artifacts/spec93-migration-assessment/pg18/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
