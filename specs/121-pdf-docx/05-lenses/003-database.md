# Lens 003 — Database Expert

## Stake

Upload format choice mostly hits **blob/object storage + job queues**, not graph schema. Still, wrong expectations (Office as first-class rows) would force new tables and retention policies. PDF already has dedicated PDF rows + convert→document linkage.

## As-is data plane

```ascii
  Text/JSON/MD  → documents row + ingest job
  Image         → documents row (+ VLM-derived text)
  PDF           → pdfs row → convert → markdown artifact → documents ingest
  DOCX/XLSX     → no durable row (reject before persist)
```

## Invariants to preserve

| ID | Invariant |
|----|-----------|
| DB-121-1 | Rejected formats leave no orphan `documents` / `pdfs` rows |
| DB-121-2 | PDF convert failure may leave PDF row in Failed; must be queryable/retryable |
| DB-121-3 | Successful PDF convert yields durable markdown before KG Insert |
| DB-121-4 | No new Office-specific tables in v1 |
| DB-121-5 | Workspace scoping on PDF mint remains fail-closed (`Workspace ID required`) |

## Future Office (non-goal)

If DOCX ships later: store original bytes optionally + **Markdown as the ingest authority** (same as PDF barrier). Avoid dual KG sources for one upload.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Office study: [../12-office-future-study.md](../12-office-future-study.md)
