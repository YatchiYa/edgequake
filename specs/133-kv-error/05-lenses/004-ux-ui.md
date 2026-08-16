# Lens 004 — UX / UI designer

## Current experience

```ascii
  Documents detail
    status pill: Failed (destructive)
    banner: long SPEC-091 / SPEC-098 technical dump
    CTA: Reprocess
```

Operators cannot tell “spine missing” from “legacy key parse” from the banner alone.

## Desired experience (this ship + light copy)

| State | UX intent |
|-------|-----------|
| True spine miss (`0/N`) | Keep hard fail; message already names PostgresEntitySink |
| Near-miss arrow class | Prefer hint: “relationship key could not be matched — reprocess after upgrade” once fix ships |
| During processing | No change — progress phases stay honest |

## Microcopy (optional WP)

When miss samples contain `->` inside names, append:

> Near-complete mirror miss with arrows in entity names — usually a key-parse class, not a missing database spine.

Do **not** invent a new status enum value for this.

## Accessibility

- Keep Failed as text + icon (not color-only).
- Banner remains selectable/copyable for support tickets.

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front lens: [005-front-designer.md](005-front-designer.md)
