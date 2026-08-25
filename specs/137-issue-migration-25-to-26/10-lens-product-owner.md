# 10 — Lens: Product Owner

## Outcome

Operators finishing 0.25→0.26 must see **two products in one CLI**:

1. **Ship the binary** — SAFE SCHEMA 149. No data deleted. No confirm.
2. **Retire legacy stores** — optional, irreversible, backup-then-confirm.

Mixing them into “0.26 migrate failed” is a messaging failure. The 0.26
highlight list (PDF pack, manuscript, Langfuse sibling) is unrelated to DROP OLD.

## Honesty rules

- Do not imply `--confirm-drop` is required for 149.
- Do not imply leftover 125/126/131 means the fleet is stuck and cannot serve.
- Do not auto-drop. Rollback after drop is restore-only — say so every time.
- Consent token spelling is a product surface: accept the permutation operators
  actually type (`--drop-confirm`) and reject garbage flags instead of ignoring
  them.

## Success metrics

- Field upgrade follows [09](09-ops-runbook.md) without a second mystery flag.
- Guard RED is treated as “do not drop”, not “CLI is broken”.
- No customer/person identifiers in this pack ([raw-logs](raw-logs/)).
