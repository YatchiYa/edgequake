# 01 — First Principles

## Axioms

1. A store that exists will be used.
2. Policy without physics fails — refuse + drop + assert.
3. Two SSOTs is a bug — era dual-read is temporary, census-gated.
4. ≤0.22 upgrade is a ladder, not a single migration.

## Laws L1–L6

See [README](README.md). Migration **142** asserts emptiness; it never silently deletes durable legacy rows (point operators to `--confirm-drop`).
