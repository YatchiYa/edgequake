# 05 — E2E / contract test matrix

> Gates for Waves A–E. Prefer extending [`e2e_spec090_multi_pool.rs`](../../edgequake/crates/edgequake-storage/tests/e2e_spec090_multi_pool.rs) over a parallel pool harness (DRY).

## Legend

| Tag | Meaning |
|-----|---------|
| C | Compile-time / source contract (`include_str!` or path assert) |
| U | Unit (no Postgres) |
| E | Postgres e2e (`DATABASE_URL`, `--features postgres`) |
| A | API / integration |

## Matrix

| ID | Wave | Tag | Assert |
|----|------|-----|--------|
| T-112-01 | A | C | Hygiene / connect helper source contains `application_name` |
| T-112-02 | A | C | `server.rs` (or shutdown path) contains pool/`bundle` `close` after drain |
| T-112-03 | A | E | After `PgPoolBundle::connect`, `SHOW application_name` on each role pool is `edgequake:<role>` |
| T-112-04 | A | E | Bundle sets finite idle/max lifetime (probe options or observe reap under short test env) |
| T-112-05 | A | E | After `bundle.close().await`, EdgeQuake backends for test user drop (or ≤ prior baseline) within timeout |
| T-112-06 | B | U | `total_max_connections` == sum of role max; clamp 1–128 still holds |
| T-112-07 | B | U | Budget helper: `instances × total` vs limit → `Ok` / `Warn` / `Fail` as configured |
| T-112-08 | B | E/A | Boot with absurd `INSTANCE_COUNT` + `BUDGET_MODE=fail` refuses start (exit / err) |
| T-112-09 | B | E | Idle process: `count(*)` for test app_name prefix ≤ configured total |
| T-112-10 | C | C | Connect SSOT contains `idle_in_transaction_session_timeout` |
| T-112-11 | C | E | `SHOW idle_in_transaction_session_timeout` non-zero on fresh checkout |
| T-112-12 | D | A | Metrics scrape includes per-role max (or documented gauge name) |
| T-112-13 | D | A | Ready/health JSON includes pool role stats when postgres feature on |
| T-112-14 | A | E | Existing multi-pool isolation still holds (ingest saturated ⇒ query `SELECT 1` ok) — **regression** |

## Suggested commands

```bash
# Storage multi-pool + new 112 asserts
DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
  cargo test -p edgequake-storage --features postgres --test e2e_spec090_multi_pool -- --nocapture

# New contract module (proposed)
cargo test -p edgequake-storage --lib contract_spec112_pool_hygiene
cargo test -p edgequake-api --test contract_spec112_shutdown_closes_pools

# Budget unit
cargo test -p edgequake-storage --lib pool_budget
```

## Edge-case coverage map

| EC (see 06) | Covered by |
|-------------|------------|
| EC-01 deploy double-count | T-112-07 with `instances=2` |
| EC-02 min_connections warm | T-112-09 |
| EC-03 SIGKILL | Ops only (document) |
| EC-04 PgBouncer txn mode | Docs / manual |
| EC-05 RESET ALL vs prepared | Existing SPEC-090 EC-06 + C hygiene |
| EC-06 empty application_name regression | T-112-03 |
| EC-07 migrate CLI extra pool | Ops budget formula includes CLI |

## Definition of done (code train)

- All T-112-01…14 for implemented waves green locally with Postgres
- Measurements transcript saved under `measurements/` (e.g. `e2e112-gates.txt`)
- [`BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md) “Bar for fixed” checklist updated
