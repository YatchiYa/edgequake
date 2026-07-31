# Matrix summary — SPEC-93 migration assessment

> Generated: 2026-07-31T11:10:24Z (started 2026-07-31T10:55:36Z)
> Source: `ghcr.io/raphaelmansuy/edgequake:0.22.0`
> Target: HEAD migrations through **137**
> Profile: `realism`

| PG profile | Verdict | Wall (s) | Postgres | Pre max mig | Post max mig | Docs seeded | Dump SHA (12) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| pg16 | **GREEN** | 232 | `16.14 (Debian 16.14-1.pgdg12+1)` | 105 | 137 | 600 | `d98f2b250deb` |
| pg17 | **GREEN** | 276 | `17.10 (Debian 17.10-1.pgdg12+1)` | 105 | 137 | 600 | `62df3f13e298` |
| pg18 | **GREEN** | 380 | `18.4 (Debian 18.4-1.pgdg12+1)` | 105 | 137 | 600 | `103997c4490e` |

**Overall:** **PASS**

## Notes

- Per-major artifacts: `reports/pg16/`, `reports/pg17/`, `reports/pg18/`
- Binary dumps: `artifacts/spec93-migration-assessment/<major>/pre-upgrade.dump`
- Protocol: [01-test-protocol.md](../01-test-protocol.md) · AC: [02-acceptance-criteria.md](../02-acceptance-criteria.md)
