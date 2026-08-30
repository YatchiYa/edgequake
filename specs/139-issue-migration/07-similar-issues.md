# 07 — Similar issues

| Spec | Symptom | Lesson reused here |
|------|---------|---------------------|
| [SPEC-110](../110-migration-issue/) | 118 `ON CONFLICT` 21000 | LAW-M1; unfakable broken-SQL fixture |
| [SPEC-111](../111-issues/) | Advisor vs 131; stamp stalls | LAW-C3; do not use normalize as drop GREEN |
| [SPEC-098](../098-data-access-hardening/) | Edge upsert 21000 | Within-batch dedupe |
| [SPEC-137](../137-issue-migration-25-to-26/) | 0.25→0.26 consent | CLI already honest; this pack is engine copy |
| [SPEC-091](../091-simplify-data-layer/) | 117–122 one-shot + engine | Remainder descriptors for skipped parents |

SPEC-139 does **not** replace 110/111 SQL or 137 CLI. It unblocks the **data
movement** that those packs assume will finish before `--confirm-drop`.
