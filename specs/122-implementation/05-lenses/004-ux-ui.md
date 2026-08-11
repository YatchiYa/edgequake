# Lens 004 — UX / UI Designer

## Stake

Users cannot see queue physics; “upload finished” feels like “RAG ready,” then minutes of Processing feel like a bug.

## Principles

1. Separate **transfer** progress from **processing** progress.
2. Prefer plain language over metrics jargon in default UI; link advanced.
3. Local mode: one clear sentence about serial processing.
4. Failures: convert vs extract vs network distinguishable (align SPEC-121).
5. Bulk summary: N admitted / K processing / M ready.

## Flows

```ascii
  Select N files
       │
       ▼
  Transfer chips (≤3 active)
       │
       ▼
  Toast: “N queued for processing”
       │
       ▼
  Table rows animate Pending→Processing→Ready
       │
       └─ Optional banner: queue depth / local serial hint
```

## Non-goals

- Dashboard of 12 KPI cards on first viewport
- Fake determinate % without stage knowledge

## Cross-refs

- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front: [005-front-designer.md](005-front-designer.md)
