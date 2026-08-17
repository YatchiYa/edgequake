# Soak verdict — 0.22.0

| Field | Value |
| --- | --- |
| **status** | GREEN |
| profile | `smoke` |
| postgres tag | `0.22.0` |
| source API | `edgequake:0.22.0` |
| compose project | `spec091soak` |
| started | 2026-08-03T12:13:28Z |
| finished | 2026-08-03T12:15:18Z |
| wall seconds | 110 |
| PASS / FAIL | 19 / 0 |
| Postgres | `18.4 (Debian 18.4-1.pgdg12+1)` |
| pre migration max | 105 |
| post migration max | 142 |
| tenants / workspaces / docs | 3 / 6 / 6 |
| dump SHA256 | `566e94c8f0f7…` (427680 bytes) |
| dump path | `/Users/raphaelmansuy/Github/03-working/edgequake/artifacts/spec091-upgrade-soak/pre-upgrade.dump` |

## Acceptance

See [02-acceptance-criteria.md](../../02-acceptance-criteria.md). Matrix rollup: [matrix-summary.md](../matrix-summary.md).

## Logs

- `soak.log`, `migrate-*.log`, `head-api.log`, `seed.env` in this directory.
