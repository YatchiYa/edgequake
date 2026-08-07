# Brutal honesty — SPEC-111 (post release-safety audit)

Date: 2026-08-07

## Verdict

**Drop-safety and test fidelity are aligned on the worktree.** Fleet LAW-C3 (provenance-only) holds for advisor uncovered ≡ 131 ≡ verify_fleet coverage `actual`. E2E asserts what the plan claims (`fleet_retirable`, dataful 131 DROP/ABORT). Clear All LAW-111-9 holds. Cancel/purge `TaskNotFound` ERROR race is soft-failed (scoped).

**Production readiness: Ship with runbook — not blind upgrade.** See [`09-ops-runbook.md`](../09-ops-runbook.md) and partner notes [`11-release-partner-notes.md`](../11-release-partner-notes.md).

| Issue | Status |
|-------|--------|
| #364 retirable = coverage | Fixed + provenance-only fleet |
| #363 iw2 normalize + failed_count | Fixed |
| #362 cast direction | Fixed |
| #366/#360 clear-all ghosts | **Fixed** — LAW-111-9 + residual KV purge |
| Cancel/purge persist ERROR | **Fixed** — missing-row soft-fail (worker/progress/admission/cancel) |
| #361 | Comment only (non-goal) |

## Honesty closeout deltas

1. iw2 verify: dead “normalize remainder” removed; pre-143 entity/rel coverage fail-closed (`actual=0`).
2. `VectorPosture.verify_chunk` / `verify_fleet` split; retirable predicates use the matching verify.
3. Dual-legacy stalls: `provenance_stall_rows` + advisor/console hint + stamp verify `stalls=N`.
4. E2E-111-11 asserts `fleet_retirable()==true` (sole-table isolation).
5. E2E-111-16 dataful 131 DROP; E2E-111-17 ABORT without provenance.
6. Lifecycle purge vs worker persist: `TaskNotFound` is debug, not ERROR.

## Scorecard (this closeout)

| Dimension | Score | Note |
|-----------|-------|------|
| Data-loss safety on drop | **High** | Provenance-only both sides + ABORT e2e |
| Advisor honesty vs SQL | **High** | Split verify; stall surfaced |
| Ops reliability | **Medium-High** | Stamp still mandatory; stalls need manual residue cleanup; checksum repair allowlist for 125/131 |
| Test honesty vs plan | **High** | 11/16/17 + clear-all + release-safety gates |
| Code maintainability | **Good** | Dead path removed; stall scan is O(uncovered) |
| Git / release packaging | **Conditional** | Stage complete for SPEC-111 core; v0.24.2 also bundles SPEC-109/110 |
| Production readiness | **Ship with runbook** | Follow `09-ops-runbook.md` |

## Gates

| Artifact | Contents |
|----------|----------|
| [`e2e111-release-safety-gates.txt`](e2e111-release-safety-gates.txt) | **Current** — fmt/clippy + honesty closeout + clear-all + checksum wiring + purge units |
| [`e2e111-honesty-closeout-gates.txt`](e2e111-honesty-closeout-gates.txt) | Prior honesty closeout (residue/retire/iw2/provenance) |
| [`e2e111-final-gates.txt`](e2e111-final-gates.txt) | Earlier Cluster A + clear-all snapshot |

## Remaining release blockers (ops, not code defects)

1. Field DBs that applied old **125/131** bodies need `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR` (prefer scoped; avoid DEV_MODE alone in prod).
2. **`iw2-fleet-provenance-stamp`** mandatory when typed rows lack `legacy_vector_id`.
3. Dual-legacy stalls: **manual** residue cleanup — no auto-delete.
4. Consent-gated `--confirm-drop` then deferred **142**.
5. Soft-fail on missing task rows can hide wrong `track_id` at debug — `update_task` stays strict elsewhere.

## Remaining non-goals (intentional)

- Auto `--confirm-drop`
- plpgsql Unicode normalize
- #361 concurrency
- Large-fleet EXPLAIN soak artifact
- Auto-delete of dual-legacy alias residue
- Full SPEC-120 mark-and-supersede (delete saga)
