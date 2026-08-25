# 05 — E2E test matrix

Proof target: `make spec137-migrate-025-026-proof`

| ID | Gate | Where |
|----|------|--------|
| E2E-137-01 | Mid-cutover: expandable 148-shaped DB; `migrate` applies 149; 125/126/131 remain; exit 0 | `cli_migrate_console` |
| E2E-137-02 | `--drop-confirm` ≡ `--confirm-drop` on empty leftover (through 124) | `cli_migrate_console` |
| E2E-137-03 | `--confirm-drp` exits non-zero; stderr names `--confirm-drop` | `cli_migrate_console` (no DB) |
| E2E-137-04 | Wave D fixture: stderr names KV residue / backfill; not tasks locks as the class | `cli_migrate_console` |
| E2E-137-05 | IW2 fixture: stderr names provenance-stamp | `cli_migrate_console` |
| E2E-137-06 | Abort classifier matches 125/126/131/142/checksum strings; advisor≡SQL stays existing contracts | unit + `e2e_spec091_console` / `e2e_spec091_vector_retire` / `e2e_spec111_provenance_parity` |
| E2E-137-07 | After successful confirm-drop, 142 applied; `ag_catalog.ag_graph` count unchanged (skip if AGE absent) | `cli_migrate_console` |
| E2E-137-08 | `migrate guard` does not change `_sqlx_migrations` row count / max version | `cli_migrate_console` |
| E2E-137-09 | `make spec137-migrate-025-026-proof` runs the suite and tees measurements | `scripts/spec137_migrate_025_026_proof.sh` |

## Existing contracts (do not fork)

Reuse; do not duplicate SQL predicates:

- `contract_spec091_advisor_matches_125_guard`
- `contract_spec091_advisor_matches_126_guard`
- SPEC-111 provenance parity (131)

## Skip policy

If `DATABASE_URL` unset: skip DB-backed cases; **E2E-137-03 and unit classifier still run**.
