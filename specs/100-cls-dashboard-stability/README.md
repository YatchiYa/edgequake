# SPEC-100 — Dashboard CLS / Layout Stability

> **Product pin**: EdgeQuake v0.22.0+  
> **Status**: In progress  
> **Inherits**: [SPEC-099](../099-ux-ui-improvement/) Documents CLS playbook (F-099-17)  
> **Scope**: All `(dashboard)` routes — reserved geometry, soft-refresh, Playwright CLS gates

## Start here

1. [00-why.md](00-why.md)
2. [00-first-principles.md](00-first-principles.md)
3. [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
4. [03-implementation-roadmap.md](03-implementation-roadmap.md)
5. [04-e2e-test-matrix.md](04-e2e-test-matrix.md)
6. [05-edge-cases.md](05-edge-cases.md)

## Playbook

1. Never `return null` for chrome that later becomes a tall card — skeleton / reserved slot.
2. Soft refetch: `placeholderData` or `isInitialLoading = isLoading && !data`.
3. Page shells: `h-full min-h-0 overflow-clip`.
4. Every surface has a Playwright CLS gate under `e2e/spec100-*`.

## Shared primitives

| Artifact | Role |
|----------|------|
| `src/lib/layout/cls-stability.ts` | `shouldReserveSlot`, session hints, `isInitialLoading` |
| `src/components/shared/reserved-slot.tsx` | minHeight floor + skeleton/children |
| Documents wrappers | `documents-layout-stability.ts` delegates to shared lib |
