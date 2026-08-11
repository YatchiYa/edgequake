# 09 — Acceptance

| # | Criterion | Pass? |
|---|-----------|-------|
| A1 | SQL repro of 23505 documented before fix | **Pass** — `10-reproduction.md` |
| A2 | `upsert_batch` never returns Err for legacy unique collisions (entity/rel/report) | **Pass** — `fleet_legacy_absorb` + T1 |
| A3 | Exactly one typed owner per `(workspace_id, legacy_vector_id)` after dual write | **Pass** — contract + e2e |
| A4 | Stamp-once: non-null lid not overwritten | **Pass** |
| A5 | Multi-workspace same lid still allowed | **Pass** |
| A6 | EntityNameIndex oldest-wins + stable `ORDER BY created_at, id` | **Pass** |
| A7 | Storage contract + storage mirror e2e green | **Pass** — 6+2 |
| A7b | Merger concurrent e2e entity + relationship green | **Pass** — api T2b (2 tests) |
| A7c | Family FK metadata unit contract (absorb OCP) | **Pass** — `contract_spec120_family_typed_fk_metadata` |
| A13 | FP close decision: no unnecessary Product B/C closes | **Pass** — `13-close-decision.md` |
| A8 | GitHub #374 investigation comment posted with SPEC-120 link | **Pass** |
| A9 | No UI redesign; this error class cannot surface when absorb returns Ok | **Pass** (by construction; not UI-tested) |
| A10 | Alias completeness deferred to SPEC-083 (honest limit) | **Pass** |
| A11 | DRY absorb module (not triplicated family SQL) | **Pass** — `fleet_legacy_absorb.rs` |
| A12 | Honest assessment doc lists residual gaps | **Pass** — `11-honest-assessment.md` |

## Honest limits

- Absorb may leave an unstamped typed FK when a peer already owns the lid.
- Historical duplicate `entities` rows are not auto-merged (SPEC-083).
- Migration `fleet_provenance_stamp` dual-legacy fail-closed policy is unchanged.
- HTTP upload worker dual-doc race is **not** covered; merger-level e2e is the bound.
- Close #374 only after merge to main.
