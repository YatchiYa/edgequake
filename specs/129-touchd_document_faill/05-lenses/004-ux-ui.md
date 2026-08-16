# Lens 004 — UX / UI

## User-visible bug

During checkpoint-resume re-embed:

| Signal | Buggy | Fixed |
|--------|-------|-------|
| Progress chip / KV | `re_embedding` | unchanged |
| Documents list SQL column | stuck `failed` | `processing` |
| Operator logs | WARN spam | quiet on happy path |

## UX contract

- Prefer **fresh processing** over stale **failed** while work continues.
- Do not invent a new list badge solely for `re_embedding` in SQL; chips already carry stage honesty from KV / WS.
- Terminal success still prefer `indexed` (C-23).

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front lens: [005-front-designer.md](005-front-designer.md)
