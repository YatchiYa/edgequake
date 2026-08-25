# 06 — Edge cases

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC1 | `--drop-confirm` vs `--confirm-drop` | Alias list SSOT | E2E-137-02 |
| EC2 | Typo `--confirm-drp` | Non-zero + hint | E2E-137-03 |
| EC3 | `--confirm` alone | Not consent; unknown on apply path | E2E-137-03 family |
| EC4 | `EDGEQUAKE_MIGRATION_CONFIRM_DROP=1` without CLI flag | Still consent (unchanged) | existing C3 + 137-02 env optional |
| EC5 | Dirty sqlx row (`success=false`) | Re-run same version; no skip flag | documented |
| EC6 | Checksum drift on applied 125/131 | Loud refuse; allowlist | classifier + SPEC-111 |
| EC7 | Statement timeout on fat KV | Fail closed; index/cast already SPEC-111 | ops runbook |
| EC8 | Empty leftover tables | 125/126/131 no-op drop / 142 drops empties | E2E-137-02/07 |
| EC9 | Dual-legacy stamp stalls | Uncovered_fleet > 0; no auto-delete | SPEC-111 runbook |
| EC10 | Partitioned `tasks` + 149 | `ADD COLUMN` on parent | 149 SQL; 137-01 |
| EC11 | AGE graph present | Count unchanged after drops | E2E-137-07 |
| EC12 | `guard` while apply holds advisory lock | Read-only; may wait; no ledger write | E2E-137-08 |
| EC13 | 142 pending, `any_legacy_rows` true | Soft-exit includes 142 | existing pending_ok_to_serve |
| EC14 | Fresh install | Confirm not required | existing `cli_migrate_fresh_install_*` |
| EC15 | Shared `.env` with CONFIRM_DROP=1 | Ops: do not set casually | configuration.md already warns |

## AGE note

[AGE manual](https://age.apache.org/age-manual/master/intro/graphs.html): delete graphs with `drop_graph(..., true)`. Manual `DROP SCHEMA graph CASCADE` is unsupported and can corrupt the label cache.
