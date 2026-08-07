# 04 — Fix Plan (SPEC-110)

> Implementation wave after this pack is authored. Acceptance checklist at bottom.

## Locked approach

| Step | Action | Why |
|------|--------|-----|
| 1 | Patch `118_spec091_wsdoc_backfill.sql` with `DISTINCT ON (doc_id)` | Unblocks stuck@117 (LAW-M1/M2/M3) |
| 2 | Patch `121_spec091_injection_backfill.sql` with `DISTINCT ON (inj_id)` | Prevent latent twin (LAW-M1) |
| 3 | Update `edgequake/migrations/checksums.lock` for both files | CI migration-guard + NOTES |
| 4 | Add bootstrap checksum repair for known-broken → fixed SHA (118, 121) | Fleets that already applied old body (LAW-M3 / M078) |
| 5 | Wire repair into pre-sqlx migrate path; DEV_MODE-gated rewrite; loud refuse otherwise | SPEC-083 X-02 |
| 6 | E2E-110-01..07 + `make spec110-migrate-118-proof` | Prove + measure |
| 7 | CHANGELOG + cut **v0.24.2** | LAW-M4 |

## Rejected alternatives

| Idea | Reject reason |
|------|---------------|
| New migration `143_*` only | Never reached while 118 fails |
| `ON CONFLICT DO NOTHING` for 118 | Loses intentional NULL-scope repair via COALESCE update |
| Multi-row membership table | Out of scope; changes SPEC-091 SSOT |
| Document “delete duplicate wsdoc keys” as ops fix | Punts product bug; every multi-ws tenant hits it |
| Silent checksum rewrite in prod | Violates SPEC-083 X-02 |

## Step 1 — Migration 118 SQL (normative)

Replace the `EXECUTE format` body with partner-baseline semantics:

- Subquery extracts `(doc_id, ws_id)` with existing UUID regex + workspace `EXISTS` guards.
- Outer: `SELECT DISTINCT ON (doc_id) doc_id, ws_id, '', 'indexed' … ORDER BY doc_id, ws_id`.
- Keep `ON CONFLICT (id) DO UPDATE SET workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)`.
- Comment: explain why `DISTINCT ON` is required (membership index vs conflict key).

Broken SHA-384 (lockfile / shipped 0.24.1):

```text
331967467fdbeb58aeeb41ca92b6e3ec87ee84ace9286166275e14af9699a4cb862f1a92516043ee9c2489138a560629
```

Fixed SHA-384: recompute after edit via `scripts/check_migration_checksums.sh` / `sha384sum`.

## Step 2 — Migration 121 SQL (normative)

Same pattern: wrap SELECT in subquery aliased columns; `DISTINCT ON (inj_id) … ORDER BY inj_id, ws_id`; preserve existing `ON CONFLICT DO UPDATE` column list.

Broken SHA-384:

```text
da347384f34eb9db99d635f482293c7ce4cb678f3dc1a809e9b0308b95a8475471a9fa5b894667fb7a9d8207d8e5de7f
```

## Step 3 — checksums.lock

Replace the two lines for `118_spec091_wsdoc_backfill.sql` and `121_spec091_injection_backfill.sql` with new SHA-384 digests in the same PR as the SQL edits.

## Step 4 — Checksum repair modules

Mirror [`m078.rs`](../../edgequake/crates/edgequake-api/src/state/migration_bootstrap/reconcile/m078.rs):

```ascii
 repair_migration_118_checksum_if_needed(pool)
   if no _sqlx_migrations → Ok(false)
   if version 118 success checksum == BROKEN_0_24_1
      if !EDGEQUAKE_DEV_MODE → Err Protocol(runbook)
      else UPDATE checksum to FIXED_0_24_2
   similarly for 121
```

Constants:

- `M118_CHECKSUM_BROKEN_V0241` / `M118_CHECKSUM_FIXED_V0242`
- `M121_CHECKSUM_BROKEN_V0241` / `M121_CHECKSUM_FIXED_V0242`

Call sites: same pre-sqlx hook chain as M071/M078 inside migration bootstrap (migrate CLI **and** any path that verifies checksums before run). Prefer one `m118.rs` (+ thin 121 twin or shared helper) under `reconcile/` — DRY with existing `allow_checksum_repair()`.

## Step 5 — Operator paths (detail in [09](09-ops-runbook.md))

| Fleet state | Action |
|-------------|--------|
| `latest_applied ≤ 117` (partner PPD) | Pull `0.24.2` → `migrate --confirm-drop` (or staged SAFE SCHEMA first). No checksum repair needed for 118 (never recorded). |
| `118` success with **old** checksum | Pull `0.24.2` → one-shot `EDGEQUAKE_DEV_MODE=true` migrate/repair **or** manual `UPDATE _sqlx_migrations` → then continue pending. |
| Fresh install on 0.24.2 | Fixed 118 applies once; store new checksum. |

## Step 6 — Tests / Makefile

| Artifact | Role |
|----------|------|
| `e2e_spec110_wsdoc_on_conflict` (or SQL harness under storage/api tests) | E2E-110-01..04 |
| Source guard `#[test]` | E2E-110-05 greps `DISTINCT ON` in 118/121 |
| Unit on repair | E2E-110-06 |
| Docker proof script | E2E-110-07 → `measurements/` |
| `make spec110-migrate-118-proof` | Wraps above |

## Step 7 — Release

1. Land SQL + lock + repair + tests.
2. `CHANGELOG.md` Unreleased / `0.24.2` note: SPEC-110 migrate 118/121.
3. `make version-bump VERSION=0.24.2` when cutting.
4. Tag `v0.24.2` → GHCR per release-and-cd.
5. Partner: retarget compose/image to `0.24.2`.

## Acceptance checklist

- [ ] 118 SQL uses `DISTINCT ON (doc_id)` + `ORDER BY doc_id, ws_id`
- [ ] 121 SQL uses `DISTINCT ON (inj_id)` + `ORDER BY inj_id, ws_id`
- [ ] `checksums.lock` updated; `scripts/check_migration_checksums.sh` green
- [ ] Repair modules refuse without DEV_MODE; rewrite only known-broken → fixed
- [ ] E2E-110-01 fails on old SQL / passes on new (or documents fixture against patched only + source of old failure)
- [ ] E2E-110-02..07 green with artifacts in `measurements/`
- [ ] Ops runbook + partner reply accurate
- [ ] **v0.24.2** published (or local image proof recorded until tag)

## Partner SQL reference (implementation paste baseline)

See partner update in the issue thread / [00-issue-data.md](00-issue-data.md). Normative copies live in the migration files after the implementation wave — do not maintain a third divergent copy in this pack once landed; this section is the design SSOT until then.
