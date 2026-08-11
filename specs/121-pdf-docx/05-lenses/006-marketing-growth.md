# Lens 006 — Marketing & Growth

## Stake

FAQ and landing/docs that say “DOCX Planned” create inbound expectations EdgeQuake will not meet in v1. Growth copy must match the engineering matrix or support load spikes.

## Messaging rules

| Do | Don't |
|----|-------|
| “Ingest Markdown, text, JSON, images, and PDFs” | “Upload any Office document” |
| “PDF conversion uses vision-capable models” | “PDF just works offline with no LLM” |
| “Word/Excel: export to PDF or Markdown” | “DOCX coming soon” without a dated SPEC |

## Funnel impact of #370

```ascii
  Partner trial (Docker)
       │
       ├─ uploads JSON demo  → delight
       ├─ uploads DOCX       → reject → feels broken if FAQ said Planned
       └─ uploads PDF        → env fail → churn unless runbook exists
```

Fixing matrix honesty is a **retention** feature, not only a bugfix.

## Cross-refs

- WHY: [../00-why.md](../00-why.md)
- FAQ drift: [../02-cross-ref-matrix.md](../02-cross-ref-matrix.md)
