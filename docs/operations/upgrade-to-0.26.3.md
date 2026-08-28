# Upgrade to EdgeQuake v0.26.3

> **From:** v0.26.1 / v0.26.2 · **To:** v0.26.3 · **CD:** GHCR (`edgequake`,
> `edgequake-frontend`, `edgequake-postgres`)

> **Honesty:** product `VERSION` is still **0.26.2**. This cut is **Unreleased**
> until `make version-bump VERSION=0.26.3` and `git tag v0.26.3`. Do **not**
> pull `ghcr.io/raphaelmansuy/edgequake:0.26.3` until that tag exists. Until
> then build/deploy **this branch**. Do **not** stay on **0.26.1** for a
> mid-cutover fleet.

Ops/product patch: SPEC-139 mid-cutover engine (iw2 21000, W3 coverage-sum
verify, KV remainder after 119-before-122). **No new migrations** — schema
train remains **149** from [upgrade-to-0.26.0.md](upgrade-to-0.26.0.md).

## Highlights

| Area | What changed |
|------|----------------|
| iw2 | Within-batch last-write-wins on arbiter keys; `ON CONFLICT DO UPDATE` COALESCE provenance kept |
| W3 | Per-table coverage SUM (≡ 126 UUID-shaped `-chunk-` ids); reclaim `verify_failed` |
| Remainder | `w2-dedup-remainder`, `wc-shell-remainder`, `w5-artifact-remainder` (engine jobs, not sqlx 150) |
| Engine | `run_engine` continues after one job `Err` |

This cut does **not** weaken DROP OLD SQL (125 / 126 / 131 / 142). Uncovered
KV/vector rows still abort. Operators still need `migrate guard` GREEN before
`--confirm-drop`. Remainder leftover orphans stay advisor-RED (copy-complete
job verify — no fail-loop).

## Sequence

```text
1. Backup (pg_dump -Fc / volume snapshot) — required if leftover 125/126/131
2. Deploy this branch (or v0.26.3 after the tag) — not 0.26.1
3. Set EDGEQUAKE_MIGRATION_MODE=automatic and start the API
4. Follow leftover 091 copy in upgrade-to-0.26.0.md (engine, not CLI copy)
5. edgequake migrate guard  → wait GREEN
6. Backup again, then: edgequake migrate --confirm-drop
7. edgequake migrate        → 142 emptiness assert
8. Verify /health version is 0.26.3 only after the tag
```

Compose / quickstart pin (after the tag):

```bash
EDGEQUAKE_VERSION=0.26.3 docker compose -f docker-compose.quickstart.yml up -d
```

Until tagged, pin the image you built from this branch.

## Verify

```bash
curl -s http://localhost:8080/health | jq -r '.version'   # 0.26.3 after tag
edgequake migrate status
edgequake migrate guard
```

Proof (dev): `make spec139-migrate-engine-proof`

## Out of scope in this cut

- New schema / migrate step (train stays **149**)
- Fresh Acc n=200 medical-mid run
- Auto `--confirm-drop`
- Editing applied 117–122 / 125–131 SQL bodies

Detail: [`specs/139-issue-migration/`](../../specs/139-issue-migration/) ·
field runbook: [`09-ops-runbook.md`](../../specs/139-issue-migration/09-ops-runbook.md).
