# Lens — Front Designer

## Scope

No brand refresh. Preserve existing dashboard chrome, typography, and density.
SPEC-144 is infrastructure; visual work is limited to ensuring loading shells
match current skeleton patterns.

## Shell guidance (Phase C only)

```ascii
  Allowlisted route transition
    ┌─────────────────────────────┐
    │  Existing layout chrome     │
    │  ┌───────────────────────┐  │
    │  │  skeleton / loading   │  │  ← reuse documents/loading.tsx language
    │  └───────────────────────┘  │
    └─────────────────────────────┘
         then stream content
```

- Reuse existing muted skeleton colors from dashboard — no new purple/glow.
- Do not introduce cards for loading states if the route already uses flat lists.
- Instant Insights (devtool) is engineer-facing; do not surface in product UI.

## Non-goals

- New marketing hero or landing redesign.
- Icon pack changes.
- Dark-mode theme overhaul.

## Cross-refs

- UX: [004-ux.md](004-ux.md)
- UX-UI spec: [06-ux-ui-spec.md](../06-ux-ui-spec.md)
