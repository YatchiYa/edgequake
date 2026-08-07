# 10 — Migration immutability (LAW-MIG) — never repeat the checksum cliff

> Trigger: local `make_dev` failed with `Migration 125 checksum drift` after SPEC-111 edited an already-applied SQL body, while the migrate step did not authorize repair.

## First principles

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  sqlx law: applied migration bytes are content-addressed history         │
│  SHA-384(file) is stored in _sqlx_migrations at first successful apply   │
│  Changing those bytes later ≠ re-running SQL — it is history rewrite     │
└──────────────────────────────────────────────────────────────────────────┘
```

| Law | Statement |
|-----|-----------|
| **LAW-MIG-1** | **Never edit applied / shipped migration SQL to fix field DBs.** Add a **new** expandable migration (or engine job) that performs the fix. |
| **LAW-MIG-2** | **`checksums.lock` + CI** (`scripts/check_migration_checksums.sh`) are the pre-merge gate that file bytes match the lock — they do **not** prove field DBs already applied the old hash. |
| **LAW-MIG-3** | **Checksum rewrite is bookkeeping only**, allowlisted, fail-loud by default. Authorize with `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=<versions>` (preferred) or `EDGEQUAKE_DEV_MODE` (broad). Never silent in production. |
| **LAW-MIG-4** | **Migrate CLI and `make_dev` share the same repair authorization.** A repair that only works on server boot (or only when auth is off) will break LD-15 visible migrate. |
| **LAW-MIG-5** | **Advisor / mirror SQL ≠ permission to mutate DROP bodies.** Prefer shared fragments in Rust (`residue.rs`, `coverage.rs`) for LAW-C3. If a DROP SQL file must change *after* some fleets applied it, ship repair module + lockfile + allowlist entry — acknowledge it is an exception, not the default. |

## Why editing 125 hurt

1. v0.24.1 fleets already applied M125 (old SHA).
2. SPEC-111 changed the SQL body for cast-direction parity (correct for *pending* applies).
3. sqlx refused: history checksum ≠ file checksum.
4. Repair existed but required `EDGEQUAKE_DEV_MODE`; **`make_dev` migrate did not pass it** (backend did) → false “migrate broken” for every local DB.

Prevention is not “remember DEV_MODE”. Prevention is **do not create the drift**, and when an exception is unavoidable, **authorize repair on the same path that runs migrate**.

## Decision tree (before touching `migrations/NNN_*.sql`)

```text
Has any published image / partner DB applied NNN?
  NO  → edit in place + ./scripts/update_migration_checksums.sh + commit lock
  YES → STOP
         Prefer: new migration NNN+k with the fix (expandable / idempotent)
         Exception only if:
           - effect already irreversible/applied (e.g. table dropped), AND
           - body change is source/parity only, AND
           - you add reconcile/mNNN.rs broken→fixed + KNOWN_CHECKSUM_REPAIR_VERSIONS
             (Rust + Makefile) + update checksums.lock + ops note
```

## Authorization matrix

| Env | Local `make_dev` | Prod image | Controlled upgrade |
|-----|------------------|------------|--------------------|
| `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` | set to known list by Makefile | unset | set once to needed versions |
| `EDGEQUAKE_DEV_MODE` | true when auth off | unset / false | avoid (also disables auth) |

SSOT list: `KNOWN_CHECKSUM_REPAIR_VERSIONS` in

- `edgequake-api/.../checksum_repair.rs`
- root `Makefile` (`KNOWN_CHECKSUM_REPAIR_VERSIONS`)

Contract: `contract_spec111_checksum_repair_wiring`.

## Operator one-liner (prod)

```bash
EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=125,131 \
  DATABASE_URL=… edgequake migrate
# then unset the env — do not leave allowlist on serving processes
```

## Anti-patterns

- Edit applied SQL “because advisor needs the same string” without a new version or repair module.
- Rely on `EDGEQUAKE_DEV_MODE` alone for migrate (couples auth off to schema history).
- Update `checksums.lock` without a field upgrade story for already-applied hashes.
- Silent `UPDATE _sqlx_migrations` in application code outside allowlisted repair modules.
