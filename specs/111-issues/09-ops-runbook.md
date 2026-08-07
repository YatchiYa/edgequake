# 09 — Ops runbook (SPEC-111 Cluster A + residual harden + honesty closeout)

Target ship: **v0.24.2**. Confirm-drop remains consent-gated.

## Sequence

```text
1. Backup (pg_dump -Fc / snapshot)
2. Roll write-stop binary; apply expandable migrations
   edgequake migrate
   - includes 143 (legacy_vector_id columns)
3. If checksum drift on 125/131 (older bodies already applied):
   - **Local `make dev`:** passes `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` + DEV_MODE into migrate automatically.
   - One-shot (preferred scoped): `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=125,131 DATABASE_URL=… cargo run -- migrate`
   - Avoid DEV_MODE alone in prod (also disables auth). Policy: `10-migration-immutability.md`.
4. Finish engine jobs:
   - w3-chunk-embedding-backfill
   - iw2-fleet-embedding-backfill
   - iw2-fleet-provenance-stamp   ← required when typed rows exist without provenance
5. Verify:
   - EDGEQUAKE_MIGRATION_VERIFY_EQUALITY default on (copy path)
   - regenerate: EDGEQUAKE_MIGRATION_VERIFY_EQUALITY=0 (coverage-only)
6. dry-run / guard:
   - chunk drop (126): uncovered_chunk==0 + backend typed + verify_chunk
   - fleet drop (131): uncovered_fleet==0 (provenance-only) + chunk covered + verify_fleet
7. If uncovered_fleet>0 and console/advisor reports dual-legacy stalls:
   - inspect alias residue (two legacy keys → one typed row)
   - manually delete/merge the extra legacy vector (no auto-delete)
   - re-run provenance-stamp
8. edgequake migrate --confirm-drop
9. edgequake migrate   # applies deferred 142 emptiness assert
```

## Clear All / delete while ingesting

If logs show `Task cancelled — preserving Cancelled status` followed by
`Failed to persist task result … Task not found`, that was a **lifecycle race**
(purge deletes the row after signalling cancel). Fixed: worker persist,
progress heartbeats, admission document-id write, and cancel apply all tolerate
missing rows (debug / Cancelled unwind). Restart the backend after upgrade; no
DB repair needed.

Partner cutover notes: [`11-release-partner-notes.md`](11-release-partner-notes.md).  
Upgrade SSOT: [`docs/operations/upgrade-to-0.24.2.md`](../../docs/operations/upgrade-to-0.24.2.md).

## KG persist near-miss (`999/1000` / arrow in entity name)

If fail-closed mirror reports SPEC-098 misses like
`27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS`, upgrade to **v0.24.2+**
(last-`->` parse) and **reprocess** the document. Do not treat as missing spine /
re-run 139–140 solely for this class. See
[`docs/operations/spec098-entity-spine-ensure.md`](../../docs/operations/spec098-entity-spine-ensure.md).

## Provenance-stamp / iw2 verify noise

If boot logs `iw2-fleet-provenance-stamp … expected=N actual=0` with **no typed spine** for those fleet keys:

1. Wipe under typed authority now **purges residual `eq_*_vectors`** (write-stop is upsert/CREATE only).
2. Stamp verify is **stampable-only** (orphans without typed rows are iw2/wipe, not stamp).
3. iw2 **ensures entity/relationship spine** when metadata has extract signals, then writes provenance.

Reset failed jobs after upgrade: `UPDATE edgequake.edgequake_migration_job SET state='pending', … WHERE step_id LIKE 'iw2%' AND state='failed'`.

## LAW-C3 (do not skip)

Fleet drop readiness is **`legacy_vector_id` provenance**, not normalize-join alone.

- Advisor `uncovered_fleet_rows` ≡ migration 131 guard ≡ `verify_fleet` coverage `actual`.
- Display-name spines need iw2 write **or** provenance-stamp before fleet GREEN.
- Exact-name SQL fallback was removed (cross-workspace false-GREEN).
- Dual-legacy stalls: typed row already holds another `legacy_vector_id` — fail-closed until operator cleans residue.

## Env

| Variable | Role |
|----------|------|
| `EDGEQUAKE_MIGRATION_MODE` | `verify` / `automatic` / `off` |
| `EDGEQUAKE_MIGRATION_VERIFY_EQUALITY` | default on; `0` = coverage-only |
| `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` | **Preferred** scoped auth for checksum rewrite (`71,78,118,121,125,131`) |
| `EDGEQUAKE_DEV_MODE` | Broad allow (also auth); local friction / legacy |

## Console lights

- `vector-chunk drop-readiness (126)` ≠ `vector-fleet drop-readiness (131)`
- Separate `verify_chunk` / `verify_fleet`; RESIDUE column = uncovered counts
- Fleet RED may show `stalls=N` when dual-legacy collisions block stamp
