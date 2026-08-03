# ISSUE — Resolver DRY (W0)

| Meta | Value |
|------|-------|
| Findings | F-102-02, F-102-03 |
| Laws | LAW-102-1, LAW-102-5 |
| Wave | W0 |
| Status | done |

## Problem

Multiple private palettes drift; defaults miss Rust entity types.

## Approach

Add `entity-type-colors.ts` with defaults, normalize, hex validate, resolve, merge/strip. Re-export from `label-utils`. Expand defaults for pipeline + manufacturing presets.

## DoD

- [x] Unit tests green  
- [x] No private TYPE_COLORS remain after W2  
