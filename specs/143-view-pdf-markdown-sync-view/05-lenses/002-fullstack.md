# Lens — Full Stack Developer

## Scope

Frontend-primary. Backend already emits markers and chunk `page_*`. No API
shape change required for v1.

## Implementation units (DRY)

```ascii
  page-markers.ts          pure parse/inject
  usePageSyncController    state + lock
  useMarkdownPageObserver  MD IO
  pdf-viewer.tsx           continuous stack
  side-by-side-viewer.tsx  sync toggle chrome
  documents/[id]/page.tsx  URL ownership
```

## SOLID checklist

| Principle | Application |
|-----------|-------------|
| S | Controller ≠ renderer ≠ marker util |
| O | New sync without changing marker grammar |
| L | Controlled `currentPage` still works for deeplink |
| I | `onPageChange` optional for standalone PDF use |
| D | Panes depend on controller interface, not each other |

## Integration points

- Preserve SPEC-128 overlay fetch for active page.
- Preserve SPEC-033/142 `?chunk=&page=` inbound.
- Sanitize allowlist must keep `data-eq-page`.

## Test layers

1. Unit: `injectPageAnchors`, controller lock.
2. Component: PDF `onPageChange` on stack scroll.
3. E2E: Playwright contracts in [08-e2e-test-matrix.md](../08-e2e-test-matrix.md).

## Cross-refs

- Architecture: [04-target-architecture.md](../04-target-architecture.md)
- Plan: [07-implementation-plan.md](../07-implementation-plan.md)
