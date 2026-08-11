# Lens 006 — AI Engineer

## Pipeline contract

Injection content still flows through the same extract → embed → persist path. SPEC-118 does **not** change prompts, gleaning, chunk geometry, or extract budgets.

## Failure mode (pre-fix)

```ascii
  LLM extract may succeed
       │
       ▼
  relational chunk persist fails on UUID parse
       │
       ▼
  task retries → failed
  (typed embeddings never get a spine to attach to)
```

## Post-fix expectations

1. Chunks land under injection UUID → embedding writer can `load_for_document`.
2. Soft `Ok(0)` on unknown non-UUID remains for unrelated ids.
3. Do not disable relational authority for injections — that reintroduces Acc/product split.

## Edge cases for AI quality

| Case | Expectation |
|------|-------------|
| Short glossary text | Still extracts; ≥0 entities OK |
| Large injection near content limit | Persist must not fail on identity |
| Re-inject / version bump | New injection UUID → new document row |
