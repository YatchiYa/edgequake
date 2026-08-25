# 03 — Root cause (code is law)

> Pre-fix = v0.26.0 shipped CLI. Post-fix = this pack.

## Track A — consent never given (primary pin)

Ticket used `--drop-confirm` three times. Pre-fix:

```rust
fn drop_confirmed(args: &[String]) -> bool {
    args.iter().any(|a| a == "--confirm-drop")
        || env EDGEQUAKE_MIGRATION_CONFIRM_DROP in {1,true,on,yes}
}
```

`dispatch_migrate`: any first token starting with `--` calls `run_migrate_cli`
with **no unknown-flag check**. `--drop-confirm` is therefore a no-op consent
token: expandable apply / soft-exit 0, DROP OLD still pending.

`migrate guard` is read-only. Retry cannot create consent.

Advisor `confirm_tag` printed `[requires --confirm]` — a **third** token.

```ascii
  operator types --drop-confirm
           │
           ▼
  drop_confirmed() == false
           │
           ▼
  ExpandableOnly / soft-exit
           │
           ▼
  "remaining migrations need confirm"
           │
           ▼
  operator runs guard (no writes)
           │
           ▼
  same command → same path
```

## Track B — consent given, SQL fail-closed

When `--confirm-drop` (or env) is set, `MigrationApplyMode::All` runs 125 then
126 then 131 then 142. Each drop file `RAISE EXCEPTION`s if coverage fails.

Pre-fix `print_wave_d_abort_hint` only matched Wave D. `print_failure_hint`
always mentioned `public.tasks` / `pg_locks` — wrong for KV/vector aborts.

`migrate guard` after a SQL abort does not apply engine jobs. Same confirm
command hits the same `RAISE`.

```ascii
  --confirm-drop
           │
           ▼
  125 guard ── ABORT ──► uncovered KV
           │
           ▼
  126 guard ── ABORT ──► uncovered chunk vectors
           │
           ▼
  131 guard ── ABORT ──► missing legacy_vector_id
           │
           ▼
  142        ── ABORT ──► leftover rows (drops not finished)
```

Engine jobs (when RED): `w3-chunk-embedding-backfill`,
`iw2-fleet-embedding-backfill`, `iw2-fleet-provenance-stamp`.
Checksum: `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` (never silent).

## What is not the root cause

| Hypothesis | Why rejected |
|------------|----------------|
| Migration 149 SQL | Additive `ADD COLUMN IF NOT EXISTS`; first migrate was OK |
| AGE DROP SCHEMA | 125/126/131 never touch graph namespaces |
| sqlx cannot apply 125 after 149 | sqlx 0.8 applies unapplied older versions ([PR #1030](https://github.com/launchbadge/sqlx/pull/1030)) |
| `guard` should drop | LAW-137-6 |

## Product honesty gap

[`upgrade-to-0.26.0.md`](../../docs/operations/upgrade-to-0.26.0.md) (pre-fix)
omitted leftover 091 confirm-drop. Operators on typed-default 0.25 followed a
greenfield 149 checklist.
