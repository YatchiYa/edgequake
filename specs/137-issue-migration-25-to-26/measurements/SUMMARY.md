# SPEC-137 measurements summary

Status: **proof green** (`make spec137-migrate-025-026-proof`, 2026-08-25).

| Artifact | Result |
|----------|--------|
| `e2e137-unit.txt` | 4/4 first_principles tests ok |
| `e2e137-cli.txt` | unknown flag + `--confirm` refuse + E2E-137-01/02/04/05/07/08 ok |
| `e2e137-contracts.txt` | 125/126 advisor≡SQL ok; SPEC-111 provenance 8/8 ok |
| `e2e137-source-guard.txt` | CONFIRM_DROP_FLAGS, unknown-flag dispatch, upgrade leftover 091 section |

E2E-137-03 covered by `cli_migrate_unknown_apply_flag_rejected`.
E2E-137-06 covered by existing LAW-C3 contracts (not forked).
E2E-137-09 is this proof target.
