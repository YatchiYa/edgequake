# ISSUE — E2E gates (W4)

| Meta | Value |
|------|-------|
| Findings | F-102-06 (+ all) |
| Laws | LAW-102-8 |
| Wave | W4 |
| Status | done |

## Problem

No automated proof that custom colors reach legend/graph and respect edge cases.

## Approach

`e2e/spec102-entity-type-colors.spec.ts` with mocked workspace/graph where QC helpers allow; cover picker, recolor, community, reset, invalid hex.

## DoD

- [x] All `spec102-*` gates listed in `04-e2e-test-matrix.md` green  
