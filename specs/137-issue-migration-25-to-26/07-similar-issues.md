# 07 — Similar issues

| Spec | Symptom | Lesson reused here |
|------|---------|---------------------|
| [SPEC-110](../110-migration-issue/) | 118 `ON CONFLICT` 21000 on `--confirm-drop` | Pack template; do not skip versions |
| [SPEC-111](../111-issues/) | Advisor RED vs SQL coverage; 131 provenance | LAW-C3; stamp jobs; checksum repair |
| [SPEC-105](../105-fix-legacy/) | 142 aborts if leftover rows | Defer 142 while `any_legacy_rows` |
| [SPEC-091 C3](../091-simplify-data-layer/15-migration-console-cli.md) | Confirm gate | Expandable-first; consent for destroy |
| [SPEC-041](../041-fix-migration/) | Checksum repair pattern | Allowlist, never silent |

SPEC-137 does **not** replace 110/111 SQL. It fixes the **0.25→0.26 operator
path** (token, hints, runbook) that those packs assumed was already obvious.
