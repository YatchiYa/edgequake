# 01 — First principles (LAW-137)

> **Cross-refs**: [WHY](00-why.md) · [Issue data](00-issue-data.md) · [RCA](03-root-cause.md)

## Axioms

1. **Expand ≠ destroy.** SAFE SCHEMA (add columns/tables) is required to serve.
   DROP OLD deletes legacy stores. Consent gates destroy only.
2. **Consent is an exact token.** A misspelled or permuted flag is not consent
   unless the product lists it as an alias. Silent ignore is a lie.
3. **Advisor GREEN ≡ SQL would pass.** If they disagree, the product is wrong
   (LAW-C3 / LAW-111-3). Aborting when GREEN is a bug; dropping when RED is a
   bug.
4. **sqlx records success only after the version body commits.** A `RAISE`
   leaves the version unapplied; retry is the same statement.
5. **AGE graphs are not `eq_*` tables.** Apache AGE deletes graphs with
   `ag_catalog.drop_graph(name, true)` — not `DROP SCHEMA … CASCADE`.

## Causal diagram

```text
  0.25 serving (SAFE SCHEMA through 148)
           │
           │  deploy 0.26 binary  (embeds 149)
           ▼
  edgequake migrate
           │
           ├─ ExpandableOnly ──► apply 149 (and any other non-drop)
           │                     omit 125/126/131; defer 142 if rows
           ▼
  DROP OLD still pending  ← legal to serve
           │
           ├─ Track A: --drop-confirm ──► NOT in drop_confirmed()
           │              unknown --* ignored ──► same expandable path
           │
           └─ Track B: --confirm-drop ──► MIGRATOR.run All
                            │
                            ├─ 125 Wave D ABORT  (KV uncovered)
                            ├─ 126 W4 ABORT      (chunk vectors)
                            ├─ 131 IW2 ABORT     (no legacy_vector_id)
                            ├─ 142 leftover rows
                            └─ checksum refuse
```

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-137-1** | **Consent SSOT** — One canonical flag `--confirm-drop`, documented alias `--drop-confirm`, env `EDGEQUAKE_MIGRATION_CONFIRM_DROP`. All stdout/usage strings use the canonical name. |
| **LAW-137-2** | **Unknown apply flags fail closed** — `edgequake migrate --*` apply path accepts only consent flags. Anything else exits non-zero with a hint. Subcommands (`guard`, `console`, …) keep their own flags. |
| **LAW-137-3** | **Drop SQL is fail-closed safety** — Do not skip 125/126/131 because upgrade is “just 149”. Uncovered rows must abort. |
| **LAW-137-4** | **Abort class honesty** — Stderr names Wave D / W4 / IW2 / 142 / checksum / lock. Generic `pg_locks` only for lock errors. |
| **LAW-137-5** | **149 ≠ 091** — Additive 149 never requires confirm. Leftover 091 drops are a different ladder. |
| **LAW-137-6** | **guard is read-only** — `migrate guard` must not insert/update `_sqlx_migrations`. |
| **LAW-137-7** | **AGE stays** — Confirm-drop must not drop `ag_catalog` graphs. |
| **LAW-137-8** | **Preflight tags match law** — Versions 144–149 (and later expandables) are tagged SAFE SCHEMA; 142 remains ASSERT. |

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| **S** | `migrate_console` owns tokens, class tags, abort text. Bootstrap owns apply mode. SQL files own drop predicates. |
| **O** | New SAFE SCHEMA versions inherit expandable tagging (`>= 106` and not irreversible). |
| **L** | Every confirm alias must go through `is_confirm_drop_flag`. |
| **I** | No “force drop” flag. |
| **D** | Runbooks depend on CLI tokens + engine job names, not ad-hoc SQL. |
| **DRY** | `IRREVERSIBLE_DROP_VERSIONS` remains bootstrap SSOT; console constants 125/126/131 must match. |
