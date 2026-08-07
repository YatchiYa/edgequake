# 01 — First principles (LAW-111)

## LAW-111-1 — Code is law

GitHub prose is a hypothesis. The SSOT is the tree:

- Advisor: `edgequake-storage/src/migration_engine/advisor/{types,rules,residue,mod}.rs`
- Backfill: `fleet_embedding_backfill.rs`, `verify.rs`
- Guards: `migrations/125_spec091_kv_drop.sql`, `126_spec091_vector_drop.sql`, `131_*` (fleet)

If docs / dry-run / job counters disagree with these, the **code** wins until patched.

## LAW-111-2 — Drop readiness = coverage, not emptiness

An irreversible DROP may delete or drop the legacy store. Therefore pre-drop readiness **must not** require the legacy store to already be empty.

Correct predicate (already used inside migration **126**):

```text
∀ legacy chunk row L: ∃ typed coverage C(L)
```

Wrong predicate (advisor `chunk_retirable` today):

```text
COUNT(legacy chunk rows) == 0
```

Emptiness is a **post-condition** of DROP, not a **pre-condition**.

## LAW-111-3 — Advisor ↔ SQL guard parity (extend LAW-C3)

SPEC-091 already asserts advisor residue SQL mirrors migration 125. Same law for vectors:

- Advisor `retirable` / `fleet_retirable` must enable **iff** the corresponding migration guard would pass.
- Contract tests that only check post-drop emptiness are insufficient (see broken parity claim in `e2e_spec091_vector_retire.rs`).

## LAW-111-4 — Progress counters measure outcomes, not scans

`processed_count += scanned` with `continue` on join miss is a **false GREEN**. Unresolved rows must increment a failure / skip-with-reason metric, and verify must compare **typed coverage vs legacy expected**.

## LAW-111-5 — Cast the constant, not the indexed column

Postgres cannot use a btree on `uuid` when the predicate is `(id)::text = $text` ([cast-vs-index](https://dba.stackexchange.com/questions/277981/understanding-index-w-cast), [pgMustard](https://www.pgmustard.com/blog/why-isnt-postgres-using-my-index)). Cast the extracted key to `uuid` (or use an expression index). Same file already does this correctly for chunks (`left(k.key, 36)::uuid`).

## LAW-111-6 — One normalize SSOT

Entity identity for joins uses `normalize_entity_name` (`edgequake_storage::entity_id`). iw2 must not invent a second equality. If graphs still diverge after normalize, report residual coverage — do not claim success.

## LAW-111-7 — Version honesty

Bugs reported on **v0.12.11** are not automatically live on **v0.24.1**. Classify: Fixed / Residual / Still present / Capacity. **Exception:** when the reporter corrects the version in-thread (as on #360 → #366), reclassify immediately.

## LAW-111-8 — Irreversible consent ≠ readiness lie

`--confirm-drop` is operator consent. Readiness must still be **truthful**. Prefer: physical SQL abort on unsafe drop + advisor GREEN only when safe — never advisor RED for a reason that can only become true after the drop.

## LAW-111-9 — List ⊆ Wipe (authoritative empty is terminal)

`GET /documents` must not read a secondary store after the primary membership plane has answered **empty**.

- Relational `documents` membership returning `Some([])` means the workspace has zero listable docs.
- Falling back to a global KV `-metadata` suffix scan on that empty set resurrects dual-write residue (Clear All ghosts — #366 / #360 on v0.24.1).
- Wipe may still **scan** secondary stores to **delete** residue; readers must not.

Corollary: every surface that can populate the list must either be purged by wipe or marked non-authoritative for reads.

## LAW-111-10 — Applied migration SQL is immutable history (LAW-MIG)

sqlx content-addresses applied files (`_sqlx_migrations.checksum`). Editing a shipped `NNN_*.sql` after field apply is history rewrite, not a schema fix.

- **Default:** new expandable migration (or engine job). Never patch applied bodies for field DBs.
- **Exception:** bookkeeping-only checksum rewrite via allowlisted `reconcile/mNNN.rs`, authorized by `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` (preferred) or `DEV_MODE` — and **the same path that runs migrate** must pass that auth (`make_dev` / LD-15).
- Full decision tree: [`10-migration-immutability.md`](10-migration-immutability.md).

## SOLID / DRY application

| Principle | Application |
|-----------|-------------|
| SRP | Coverage predicate lives in one helper used by advisor + (documented) mirror of SQL |
| OCP | New embedding families extend coverage helper, not new emptiness checks |
| LSP | `retirable()` means “migration 126 guard would pass”, not “table already empty” |
| ISP | Separate `coverage_ready` vs `post_drop_empty` signals in console; separate membership `authoritative` from wipe residue scan |
| DIP | Console prints posture; does not redefine predicates |
| DRY | Fix cast once in shared SQL fragments for advisor + 125; fix join once via normalize; one `WorkspaceMetadataKeyList` for list authority; one `allow_checksum_repair` + twin allowlists (Rust/Makefile) |
