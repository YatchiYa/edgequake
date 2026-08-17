# 12 — Release Lessons (SPEC-104)

## Why these escaped into 0.22 / 0.23

1. **Inspector was not on the schema SSOT path** — storage renamed graphs to `eq_*_graph`; monitor kept `"edgequake"` and `id`.
2. **Fail-open hid defects** — `.unwrap_or(true)` and early returns converted 42703 / missing KV into “healthy”.
3. **SPEC-091 cutover updated writers, not all readers/monitors** — INV-03 still KV-shaped after mig 125.
4. **No CI exercised hourly SQL against live AGE + workspaces PK**.
5. **Unique natural keys lacked idempotent writes** — retries amplified 23505.

## Hard gates before next tag

```ascii
 RELEASE GATE (data-layer monitors)
 ┌──────────────────────────────────────────────────────────┐
 │ 1. InspectorConfig graph == storage graph_name helper    │
 │ 2. No SQL text "workspaces WHERE id" in tree             │
 │ 3. INV-03 references public.chunks (not only KV)         │
 │ 4. Tenant create ON CONFLICT (slug) covered by E2E-104-04│
 │ 5. M038 GIN check or issue331 e2e still green            │
 │ 6. Prod-class errcodes 42703/42P01 from inspector = fail │
 └──────────────────────────────────────────────────────────┘
```

## Checklist (operator + engineer)

- [x] `rg 'workspaces WHERE id' edgequake/crates` → empty (post-104)
- [x] `rg 'graph_name: "edgequake"' edgequake/crates` → empty in Default
- [x] `cargo test -p edgequake-api --features postgres --test contract_spec104_datalayer`
- [x] `e2e_issue331` adjacency: GIN present via SQL spotcheck (`measurements/v23-sql-spotchecks.txt`); named filter empty on this workspace — see measurements README
- [x] Staging: `inspect` with zero 42P01/42703 in Postgres logs **since MARKER** (`measurements/v23-*`)
- [x] Tutorial `multi-tenant.md` shows `workspace_id`
- [x] Release notes: tenant duplicate slug → 200 existing / 409 identity clash ([`CHANGELOG.md`](../../CHANGELOG.md) Unreleased)
- [x] Deploy SPEC-104 binary only after chunk backfill / with post-091 DB (EC-16) — staging DB is post-091 (`chunk_text_ssot=relational`)

## Principle for the next release

**Every SSOT migration must update every reader and every monitor in the same change set, with an e2e that fails if the old name remains.** Monitors that cannot see the truth must not claim health.

See also: [13-fix-assessment.md](13-fix-assessment.md) ship matrix.
