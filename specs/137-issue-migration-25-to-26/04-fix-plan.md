# 04 — Fix plan

## Locked approach

| Step | Action | Why |
|------|--------|-----|
| 1 | Consent SSOT in `migrate_console` (`CONFIRM_DROP_FLAGS`) | LAW-137-1 |
| 2 | Alias `--drop-confirm`; canonical `--confirm-drop` in all hints | Ticket token |
| 3 | Reject unknown flags on apply path | LAW-137-2 |
| 4 | `classify_migrate_abort` + `print_drop_abort_hint` | LAW-137-4 |
| 5 | `migration_class_tag`: expandable `>= 106` except drops/142 | LAW-137-8 |
| 6 | Patch `upgrade-to-0.26.0.md` 091 ladder | LAW-137-5 |
| 7 | E2E-137-01..09 + `make spec137-migrate-025-026-proof` | Proof |
| 8 | CHANGELOG Unreleased | Honesty |

## Rejected alternatives

| Idea | Reject reason |
|------|----------------|
| Auto `--confirm-drop` on 0.26 | Destroys data without consent |
| Skip 125 if 149 pending | sqlx/order mix; data loss |
| Edit 125/126/131 SQL to be laxer | LAW-137-3 |
| Treat `--confirm` as drop | Too broad (`console --watch` family) |
| `DROP SCHEMA CASCADE` on AGE | Crashes / corrupts AGE catalog |

## SOLID mapping

- **S:** tokens/hints in `migrate_console`; apply mode in bootstrap; predicates in SQL.
- **O:** new expandables inherit class tags; new abort markers add a match arm.
- **D:** e2e drives the binary, not a second parser.

## Implementation notes

- Keep `print_wave_d_abort_hint` as a thin alias of `print_drop_abort_hint`.
- `print_failure_hint` must not always mention tasks locks.
- Do not set `EDGEQUAKE_MIGRATION_CONFIRM_DROP` in shared `.env` examples casually.

## Acceptance

- [x] Spec pack (this directory)
- [x] Alias + unknown-flag CLI
- [x] Abort classifier
- [x] Upgrade runbook
- [x] E2E proof target (`make spec137-migrate-025-026-proof`)
