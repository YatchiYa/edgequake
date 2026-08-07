# 05 — E2E test matrix

## Gates (must be green before claiming Cluster A + honesty closeout fixed)

| ID | Issue | Gate | How |
|----|-------|------|-----|
| E2E-111-01 | #362 | Advisor residue query uses Index Cond–friendly cast | Source contract: forbid `id::text = substring` / `document_id::text = substring` in `residue.rs` + `125_*.sql`; allow `= substring(...)::uuid` |
| E2E-111-02 | #362 | Parity advisor ↔ 125 | Existing `contract_spec091_advisor_matches_125_guard` |
| E2E-111-03 | #362 | Timing (optional soak) | Fixture ~50k KV metadata keys; residue < 5s under default statement_timeout |
| E2E-111-04 | #363 | Normalize join recovers display-name entities | Seed `entities.name='Acme Corp Ltd'`, legacy `entity:ACME_CORP_LTD` + rel key; after iw2 batch, typed row exists |
| E2E-111-05 | #363 | False GREEN forbidden | Force unresolvable names; job verify fails or `failed_count`/`unresolved` > 0; `processed_count` must not imply 100% written |
| E2E-111-06 | #364 | Pre-drop retirable ≡ 126 guard | After full chunk coverage + backend flip: `retirable()==true` **and** `legacy_chunk_rows > 0`; after 126: rows 0 / dropped |
| E2E-111-07 | #364 | Console language | dry-run must not say “un-migrated” when uncovered==0 |
| E2E-111-08 | #366/#360 | Wipe empties list + no KV resurrect | Seed ≥3 docs; wipe; list 0; plant raw `eq_*_kv` ghosts; list still 0 |
| E2E-111-09 | #361 | Baseline only | Record ingest wall time for N PDFs on known provider — no pass/fail until SLO defined |
| E2E-111-10 | #364 | No provenance → not fleet_retirable | `e2e_spec111_provenance_parity` |
| E2E-111-11 | #364 | Stamp → `fleet_retirable()==true` + `legacy_fleet_rows>0` | Sole-table isolation + typed backend |
| E2E-111-12 | mirror | INSERT stamps `legacy_vector_id` | provenance parity |
| E2E-111-13 | coverage | Missing workspace_id stays uncovered | provenance parity |
| E2E-111-14 | stall | Dual-legacy → stall count + advisor “stall” reason | provenance parity |
| E2E-111-15 | console | Chunk vs fleet lights + verify_fleet fields | source contract |
| E2E-111-16 | #364 | Dataful 131 DROP after GREEN | `raw_sql(MIGRATION_131)`; table gone; typed survives |
| E2E-111-17 | #364 | 131 ABORT without provenance | `SPEC-091 IW2 ABORT` / `legacy_vector_id` |

## Commands (local)

```bash
# Honesty closeout storage gates (postgres feature)
EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1 \
DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake \
  cargo test -p edgequake-storage --features postgres \
  --test e2e_spec111_provenance_parity \
  --test e2e_spec111_iw2_normalize \
  --test e2e_spec091_vector_retire \
  --test contract_spec111_residue_cast \
  -- --nocapture

# API wipe / delete (existing)
cargo test -p edgequake-api --features postgres \
  --test e2e_document_deletion_postgres \
  -- --nocapture

# WebUI clear-all demotion (UX, not completeness)
cd edgequake_webui && pnpm exec playwright test e2e/spec099-clear-all-demoted.spec.ts
```

## New tests (with fix PR)

| Test name | Crate |
|-----------|-------|
| `contract_spec111_residue_cast_direction` | edgequake-storage |
| `e2e_spec111_iw2_normalize_join` | edgequake-storage |
| `e2e_spec111_iw2_unresolved_fails_verify` | edgequake-storage |
| `contract_spec111_retirable_pre_drop_coverage` | edgequake-storage (extend retire e2e) |
| `e2e_spec111_clear_all_list_empty` | edgequake-api |
| `e2e_spec111_provenance_parity` (10–17) | edgequake-storage |

## Measurement artifacts

Store under `specs/111-issues/measurements/` when gates run (mirror SPEC-110).
