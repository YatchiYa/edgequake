# Database Migrations

EdgeQuake uses [SQLx](https://github.com/launchbadge/sqlx) embedded migrations.
**Schema is applied only by `edgequake migrate`** (SPEC-091 LD-15 / SPEC-150).
The API process never auto-applies numbered migrations on start.

**Operator guide (plain English):**
[`docs/operations/upgrading.md`](../../docs/operations/upgrading.md)

## Quick Reference

| Task | Command |
|------|---------|
| Fresh dev stack | `make dev` (runs `edgequake migrate` before backend start) |
| Preflight | `edgequake migrate check` |
| Apply schema | `edgequake migrate` / `edgequake migrate --confirm-drop` |
| Drain data jobs | `edgequake migrate drain` |
| Preview | `edgequake migrate dry-run` |
| Check immutability | `./scripts/check_migration_checksums.sh` |
| Epoch coverage | `./scripts/check_epoch_coverage.sh` |
| Manifest SSOT | `edgequake/migrations/manifest.toml` |

**Train:** HEAD ships through migration **186**
(`170_spec164_plain_graph.sql` expand, `171_spec164_drop_age.sql` confirm-drop,
`172`–`186` request-path indexes).
Apply 170 with `edgequake migrate`. Apply 171 only with `--confirm-drop` after
`PostgresAGEGraphStorage::run_spec164_cutover` marks `eq_graph_cutover` ready.
Migrations 172 through 186 do not raise the serving floor.
The released product pin stays **v0.34.0** until the version-bump ceremony;
`release_for_schema` maps 170–186 to v0.35.0.
Run `edgequake migrate` before expecting `/ready` 200 when upgrading from an older schema.

## How Migrations Work

1. **Numbered SQL files** — `NNN_description.sql` in `edgequake/migrations/`
2. **Explicit CLI apply** — `edgequake migrate` (admin pool + reconcile)
3. **Per-step progress** — `[ i/N ] version description … applied in Xs`
4. **Manifest** — phases, fossils, irreversible drops, compat window (SPEC-150)
5. **Immutability lock** — `checksums.lock` is append-only; never edit shipped bodies
6. **Support scripts** — `migrations/support/` ops-only SQL (also locked)
7. **Epoch proof** — every published `vX.Y.Z` maps to `scripts/spec150/epochs.toml`

### Serving vs migrate

| Process | Writes schema? | Behavior when behind |
|---------|----------------|----------------------|
| `edgequake migrate` | Yes | Applies expandables; gates drops behind `--confirm-drop` |
| API (`edgequake`) | No | `EDGEQUAKE_SCHEMA_GATE=fail` → exit 78; `wait` → lite `/live` + `/ready` 503 |

**Env vars (SPEC-150):** see [upgrading.md](../../docs/operations/upgrading.md) §7.

## Golden path (upgrade)

```bash
edgequake migrate check
edgequake migrate dry-run
edgequake migrate
edgequake migrate drain --timeout 3600   # optional data cutover
edgequake migrate --confirm-drop         # only when guard GREEN
curl -sf localhost:8080/ready
```

Full runbook: [`docs/operations/upgrading.md`](../../docs/operations/upgrading.md).  
Design: [`specs/150-reliable-migration-system/`](../../specs/150-reliable-migration-system/).  
Epoch proof: `make spec150-matrix-quick`.
