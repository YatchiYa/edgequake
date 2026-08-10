# ISSUE — E2E gates

**Findings:** F-114-09 (+ all via matrix)  
**Wave:** W6  
**Laws:** LAW-114-8

## Goal

Every F-114-* maps to a green gate per [04-e2e-test-matrix.md](../04-e2e-test-matrix.md).

## Work

1. Rust API test file `e2e_spec114_relation_types.rs` (or extend spec013).  
2. Pipeline unit tests for `enforce_relation_type`.  
3. FE unit tests for presets + preview helpers.  
4. Playwright `e2e/spec114-kg-schema.spec.ts`.  
5. Non-regress: spec096, spec101, spec102.

## Acceptance

Document commands in README Verification; optional `measurements/` capture after first green run.
