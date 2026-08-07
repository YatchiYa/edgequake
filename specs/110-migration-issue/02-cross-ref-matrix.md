# 02 — Cross-Reference Matrix (SPEC-110)

| Symptom | Law | Smoking gun | Fix | E2E | Status |
|---------|-----|-------------|-----|-----|--------|
| Partner migrate fails 118 / 21000 | M1, M2 | `SELECT DISTINCT` + `ON CONFLICT (id) DO UPDATE` in 118 | `DISTINCT ON (doc_id)` | E2E-110-01..03 | **Spec'd** |
| Same class on injection backfill | M1, M2 | 121 workspace-prefixed key + `ON CONFLICT (id)` | `DISTINCT ON (inj_id)` | E2E-110-04 | **Spec'd** |
| Repo fix unused on GHCR 0.24.1 | M4 | `sqlx::migrate!` embed | Cut **v0.24.2** | E2E-110-07 | **Spec'd** |
| Already-applied old 118 + new binary | M3 | `_sqlx_migrations.checksum` ≠ new SHA | M078-style repair | E2E-110-06 | **Spec'd** |
| Append-only 143 proposed | M3 | sqlx applies in order; stuck at 118 | Reject; edit 118 | — | **Locked** |
| Multi-ws membership “lost” | M5 | `documents` PK + single `workspace_id` | Deterministic collapse | E2E-110-02 | **By design** |

## Call graph (migrate)

```ascii
 edgequake migrate --confirm-drop
   → migration_bootstrap (admin pool)
        → preflight (applied/pending)
        → [SPEC-110] checksum repair m118/m121 if needed
        → sqlx Migrator.run
             → 118_spec091_wsdoc_backfill.sql   ← ★ fails on 0.24.1
             → 119 … 142
```

## External / internal anchors

| Anchor | Path / URL | Role |
|--------|------------|------|
| Broken SQL | `edgequake/migrations/118_spec091_wsdoc_backfill.sql` | F1 |
| Latent twin | `edgequake/migrations/121_spec091_injection_backfill.sql` | F3 |
| Embed site | `edgequake-api/.../migration_bootstrap/mod.rs` `sqlx::migrate!` | M4 |
| Checksum lock | `edgequake/migrations/checksums.lock` | M3 |
| Repair precedent | `migration_bootstrap/reconcile/m078.rs` | M3 |
| Immutability notes | `edgequake/migrations/NOTES.md` | Exception via LAW-M3 |
| wsdoc grammar | `edgequake-storage/.../kv_key_schema.rs` | Membership index |
| Upgrade ops | `docs/operations/spec091-upgrade-from-v0.22.0.md` | Partner path |
| Release | `docs/operations/release-and-cd.md` | 0.24.2 cut |
| Postgres law | https://www.postgresql.org/docs/current/sql-insert.html | M1 |
| U126 | https://pganalyze.com/docs/log-insights/app-errors/U126 | M1 |
