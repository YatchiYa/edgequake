# ISSUE — Selector UI (W3)

| Meta | Value |
|------|-------|
| Findings | F-102-04 |
| Laws | LAW-102-6, LAW-102-7 |
| Wave | W3 |
| Status | done |

## Problem

Admins cannot assign colors when configuring entity types.

## Approach

Optional `colors` / `onColorsChange` on `EntityTypeSelector`; swatch + native color + hex + reset. Wire create/reconfigure wizards + payloads.

## DoD

- [x] `data-testid`: `entity-type-color-swatch`, `entity-type-color-picker`, `entity-type-color-reset`  
- [x] Playwright selector-picker gate  
