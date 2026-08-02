# Matrix summary — SPEC-93 migration assessment

> Generated: 2026-08-02T12:38:33Z (started 2026-08-02T12:25:05Z)
> Source: `ghcr.io/raphaelmansuy/edgequake:0.22.0`
> Target: HEAD migrations through **141**
> Profile: `realism`
> Isolation: foreign host ports unchanged (EdgeForce :8787/:55432, GPS, …)

| PG profile | Verdict | Wall (s) | Postgres | Pre max mig | Post max mig | Docs seeded | Dump SHA (12) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| pg16 | **GREEN** | 85 | `16.14 (Debian 16.14-1.pgdg12+1)` | 105 | 141 | 600 | `99706454b81a` |
| pg17 | **GREEN** | 397 | `17.10 (Debian 17.10-1.pgdg12+1)` | 105 | 141 | 600 | `581e12c159ad` |
| pg18 | **GREEN** | 322 | `18.4 (Debian 18.4-1.pgdg12+1)` | 105 | 141 | 600 | `657b8a826429` |

**Overall:** **PASS**

## Notes

- Per-major artifacts: `reports/pg16/`, `reports/pg17/`, `reports/pg18/`
- Binary dumps: `artifacts/spec93-migration-assessment/<major>/pre-upgrade.dump`
- Protocol: [01-test-protocol.md](../01-test-protocol.md) · AC: [02-acceptance-criteria.md](../02-acceptance-criteria.md)
