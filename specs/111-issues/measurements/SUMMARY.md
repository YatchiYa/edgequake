# SPEC-111 Cluster A — measurement / gate summary

Date: 2026-08-06  
Database: local `edgequake_test` scratch (checksum-repaired 125/131 before migrate)

## Gates run

| ID | Command | Result |
|----|---------|--------|
| coverage unit | `cargo test -p edgequake-storage --lib migration_engine::coverage` | PASS (3) |
| E2E-111-01 | `--test contract_spec111_residue_cast` | PASS (2) |
| E2E-111-04/05 | `--test e2e_spec111_iw2_normalize` + `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1` | PASS (3) |
| E2E-111-06/07 | `--test e2e_spec091_vector_retire` (pre-drop retirable) | PASS (5) |
| E2E-111-01 parity | `--test e2e_spec091_console … matches_125_guard` | PASS |
| wave_d cast | `--test e2e_spec091_wave_d … drop_guard_verifies_typed_ssot` | PASS |
| E2E-111-08 | `--test e2e_document_deletion_postgres … e2e_spec111_clear_all_list_empty_pg` | PASS |

## Ops notes

- `EDGEQUAKE_MIGRATION_VERIFY_EQUALITY` default **on**; set `0` for coverage-only verify after regenerate.
- Checksum repair for migrations **125** / **131**: `EDGEQUAKE_DEV_MODE=1` on API boot (never silent in prod).
- Scratch test DBs auto-repair known broken→fixed checksums before `sqlx migrate` (test harness only).
- `--confirm-drop` remains consent-gated; advisor GREEN ≠ auto-drop.

## #361

No code change. Request partner timings (docs/min, LLM provider, queue depth) before any concurrency work.
