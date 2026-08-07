# 04 — Fix plan (DRY / SOLID)

Priority: **#363 → #364 → #362** (Cluster A) in parallel with **#366/#360** (List ⊆ Wipe). #361 measure-only until data.

## Phase A — Shared coverage primitives (DRY)

Add one module (suggested): `migration_engine/coverage.rs`

| API | Responsibility |
|-----|----------------|
| `legacy_chunk_uncovered_count(pool)` | Same predicate as migration 126 guard |
| `legacy_fleet_uncovered_count(pool)` | Same idea for entity/rel/report vs typed tables |
| `typed_vs_legacy_coverage_ratio(...)` | Console + job verify |

**Do not** invent a second SQL dialect for advisor vs migration. Prefer extracting shared SQL fragments or generating both from one template (LAW-111-3).

## Phase B — #363 iw2 join + honesty

1. **Join:** bind `normalize_entity_name(src/tgt)` — or SQL `normalize` expression matching `entity_id::normalize_entity_name` — when looking up `entities.name`. Prefer storing/comparing normalized names only (SSOT).
2. **Metrics:** track `unresolved_join`, `written`, `scanned` separately; terminal job **fails verify** if `written / expected < threshold` OR uncovered > 0.
3. **failed_count:** increment on unresolved (or dedicated column) — never report success when coverage ≪ 100%.
4. **Escape hatch (ops):** document “regenerate from spine” as supported recovery; verify path for regenerate must use **coverage** not byte equality (Phase C).

E2E: seed display-case entities + normalized legacy keys → assert written > 0 after normalize; assert job fails if names cannot match.

## Phase C — #364 advisor retirable

1. Change `chunk_retirable` / `fleet_retirable` to use **uncovered == 0** (+ backend flipped + verify policy), **not** `legacy_*_rows == 0`.
2. Keep `legacy_*_rows` as informational residue columns in console (rename header: `LEGACY` vs `UNCOVERED`).
3. Fix console copy: “un-migrated” → “uncovered” when that is the truth.
4. Verify policy split:
   - **Copy mode:** sampled numeric equality (current).
   - **Regenerate mode / coverage-only:** presence join passes; optional cosine threshold later.
5. Repair `contract_spec091_advisor_matches_126_guard` to assert **pre-drop** retirable ≡ guard pass.

## Phase D — #362 cast direction

1. In `residue.rs` (RESIDUE_SQL + GUARD_TOTAL_SQL): replace

   `d.id::text = substring(...)` → `d.id = substring(...)::uuid`  
   (same for `document_id`).

2. Apply **identical** change to `125_spec091_kv_drop.sql` (checksums.lock + checksum repair if already applied — follow SPEC-110 / M078 pattern if 125 already ran in field; if 125 pending, in-place edit + lockfile).

3. Contract: EXPLAIN or timing test on fixture ≥N keys proves Index Cond; existing `contract_spec091_advisor_matches_125_guard` still green.

## Phase E — #366 / #360 Clear All (LAW-111-9)

1. Membership listing returns `WorkspaceMetadataKeyList { authoritative }`.
2. All list readers: if `authoritative`, empty keys ⇒ empty list (no global KV suffix).
3. Wipe `ClearingRelational`: after typed set-delete, `plan_workspace_document_kv_deletion` + `kv.delete` (wipe may suffix-scan to **delete** residue).
4. E2E: wipe → list 0; plant raw `eq_*_kv` ghosts → list still 0.

## Phase F — #361

1. Ask reporter for: file count, sizes, provider, worker concurrency, wall times.
2. Benchmark against SPEC-090 baselines; only then change caps.

## Non-goals

- Auto `--confirm-drop` without consent.
- Deleting legacy rows during backfill (emptiness as readiness) — violates LAW-111-2.
- Silent checksum rewrite in prod without DEV_MODE (SPEC-083).

## Implementation order (checklist)

```
- [x] A: coverage helpers + unit tests (`migration_engine/coverage.rs`)
- [x] B: iw2 normalize + metrics + e2e (`e2e_spec111_iw2_normalize`)
- [x] C: retirable predicate + console + parity contract (coverage + provenance fleet)
- [x] D: residue + 125 cast + checksum discipline (m125/m131 + allowlist)
- [x] E: LAW-111-9 + wipe residual KV + clear-all e2e (#366)
- [ ] F: #361 measurement intake (non-goal for v0.24.2 code)
- [x] CHANGELOG Unreleased + partner comments updated
- [x] Residual LAW-C3: provenance stamp + E2E-111-10..17 + release-safety gates
- [x] Cancel/purge TaskNotFound soft-fail (worker/progress/admission/cancel)
```
