# 06 — UX / UI Spec

## Visual scope

Infrastructure upgrade. No new brand surfaces. Preserve dashboard layout,
spacing, and existing loading skeletons.

## States

| State | UI |
|-------|-----|
| Bump only (Phases B) | Identical to pre-upgrade product UI |
| Auth redirect | Existing `/login?redirect=` |
| Swagger | Canonical `/swagger-ui/` with assets |
| SSE query | Streaming tokens (no buffered flash) |
| Phase C allowlisted nav | Existing skeleton/shell then content |

## Instant Navigations UX (Phase C)

```ascii
  User clicks Documents
       │
       ▼
  Shell paints immediately (layout + skeleton)
       │
       ▼
  List data streams / hydrates
```

Routes outside allowlist keep current navigation timing (acceptable).

## Accessibility

- Do not remove focus management on login redirect.
- Loading shells must not trap focus incorrectly (reuse existing patterns).

## Cross-refs

- Front designer: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
