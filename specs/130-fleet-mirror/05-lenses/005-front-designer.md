# Lens 005 — Front Designer

## Display vs column

```ascii
  Backend integrity (SPEC-130)     Front display
  ----------------------------     -------------
  relationships.id + embeddings    Documents status chip
  miss sample in API logs          Optional detail drawer / error string
```

No new visual component required. Preserve existing Failed / Completed chips and document detail error text.

## FE SSOT

- Status chip continues to follow document status APIs (KV + SQL merge as today).
- Do not invent a client-side “retry until mirror works” spinner policy.

## Visual non-goals

- Dashboard widgets for fleet FK miss rates (ops/metrics later).
- Redesign of side-by-side viewer for this bug.

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
