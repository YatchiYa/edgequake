# 11 — Partner release notes (v0.24.2 candidate)

**Audience:** partners upgrading from **v0.24.1**.  
**Honesty:** Cluster A (#362–364) + Clear All (#366/#360) are fixed in this pin. Field cutover is **ship-with-runbook**, not click-and-forget.

## What this release fixes

| Issue | Partner symptom | Fix in this pin |
|-------|-----------------|-----------------|
| #364 | Dry-run RED while SQL guard would pass (emptiness paradox) | Readiness = **coverage** (`uncovered_* == 0`); fleet = **`legacy_vector_id` provenance** |
| #363 | iw2 false GREEN / silent join miss | Normalize join + honest `failed_count` / verify |
| #362 | KV residue advisor timeout | Cast `substring…::uuid` (advisor + migration **125**) |
| #366 / #360 | Clear All leaves ghost docs | LAW-111-9: authoritative empty list + wipe purges residual KV |
| (ops noise) | `ERROR persist_task_result … Task not found` after Clear All | Expected race; soft-failed (debug) |

Also in the same Unreleased vehicle: **SPEC-110** (migrate 118/121 ON CONFLICT) and **SPEC-109** (reasoning effort). Call those out separately in CHANGELOG if you only care about migrate/Clear All.

## What this release does **not** fix

- **#361** bulk upload “too slow” — capacity / LLM-bound; **out of scope** (measure only; no concurrency code in this ship).
- Auto `--confirm-drop` — still consent-gated.
- Auto-delete of **dual-legacy** alias residue — operator must clean manually when console shows `stalls=N`.
- Full task delete-saga redesign (SPEC-120) — purge still deletes rows; worker tolerates missing rows.

## Mandatory upgrade sequence

Follow [`09-ops-runbook.md`](09-ops-runbook.md). Short form:

1. **Backup** (`pg_dump -Fc` / snapshot).
2. Deploy this binary; run expandable migrate (includes **143** `legacy_vector_id` columns).
3. If checksum drift on **125/131** (or 118/121 from SPEC-110):  
   `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=71,78,118,121,125,131`  
   Prefer scoped allowlist. Avoid leaving `EDGEQUAKE_DEV_MODE=1` on in prod (also disables auth).
4. Finish engine jobs: `w3-chunk-embedding-backfill`, `iw2-fleet-embedding-backfill`, **`iw2-fleet-provenance-stamp`**.
5. Dry-run until GREEN (chunk **126** + fleet **131** provenance). If `stalls=N`, clean dual-legacy residue manually, re-stamp.
6. `edgequake migrate --confirm-drop` (consent).
7. `edgequake migrate` again for deferred **142** emptiness assert.

Without steps 3–5, dry-run may stay RED or stamp verify may fail (`expected=N actual=0` class) — that is **protective**, not a regression to force past.

## Clear All on old pins

If you Clear All’d on **v0.24.1**, upgrade first, then Clear All again (or wipe) so residual KV ghosts are purged under LAW-111-9.

## Proof artifacts (engineering)

- [`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md)
- [`measurements/e2e111-release-safety-gates.txt`](measurements/e2e111-release-safety-gates.txt) — fmt/clippy + honesty + clear-all + checksum wiring

## One-line partner status

> **v0.24.2 candidate:** Cluster A + Clear All fixed; upgrade with the SPEC-111 runbook (stamp + allowlisted checksum repair + confirm-drop). #361 not in this ship. Dual-legacy stalls need manual cleanup.
