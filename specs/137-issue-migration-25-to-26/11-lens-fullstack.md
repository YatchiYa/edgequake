# 11 — Lens: Full Stack Developer

## Surfaces

| Surface | Change |
|---------|--------|
| `dispatch_migrate` | Apply path: only consent flags; unknown `--*` bail |
| `drop_confirmed` | `CONFIRM_DROP_FLAGS` SSOT |
| Usage / first principles / soft-exit | Canonical `--confirm-drop`; mention alias once in usage |
| `confirm_tag` | `[requires --confirm-drop, IRREVERSIBLE]` not `--confirm` |
| `migration_class_tag` | 144–149 SAFE SCHEMA |
| `print_failure_hint` / `print_drop_abort_hint` | Classified |

## Dispatch (first principles)

```text
  migrate                 → apply (no extra flags)
  migrate --confirm-drop  → apply + consent
  migrate --drop-confirm  → apply + consent (alias)
  migrate --confirm-drp   → ERROR (unknown)
  migrate guard [--family]→ read-only
  migrate console --watch → subcommand flags, not apply
```

## DRY

Do not parse consent in `main.rs` and again in the advisor. One
`is_confirm_drop_flag`.

## Tests

Binary integration: `edgequake/tests/cli_migrate_console.rs`.
Unit: `classify_migrate_abort` / flag helpers in `migrate_console.rs`.
