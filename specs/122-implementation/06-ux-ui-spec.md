# 06 — UX / UI Spec

## Principles

1. **Admit ≠ ready** (LAW-122-1): copy must never say “uploaded and searchable” on HTTP 202 alone.
2. **Queue physics visible:** show count Processing / Pending / Completed; optional depth from queue-metrics.
3. **One job per surface:** Documents list = status truth; dropzone = transfer progress only.
4. **Honest bulk:** “Uploading 3 of N” (transfer) vs “Processing 1 of N — local mode processes one at a time” (ingest).
5. **Cancel reachable** while Pending/Processing (existing cancel APIs).

## Surfaces

| Surface | Shows | Must not |
|---------|-------|----------|
| Dropzone transfer bar | Bytes / file admit success|fail | Imply KG done |
| Document row status | Pending → Processing → Completed/Failed | Collapse convert vs extract |
| Bulk summary toast | “N admitted; processing queued” | “N documents ready” |
| Detail / side-by-side | PDF convert progress vs ingest | Hide Failed convert as upload fail |
| Ops (advanced) | Link/hint to queue-metrics | Require ops for basic honesty |

## Copy taxonomy

| State | User-facing | Notes |
|-------|-------------|-------|
| Admit OK | “Queued for processing” | 202 |
| Local serial | “Processing one document at a time (local LLM)” | When profile=local |
| Docker/cloud parallel | “Processing up to K documents in parallel” | K = tenant cap |
| PDF convert | “Converting PDF…” | Pre-searchable |
| Extract/embed | “Extracting knowledge…” / “Indexing…” | |
| Complete | “Ready” / Completed | Searchable |
| Failed convert | “PDF conversion failed” | Not unsupported (SPEC-121) |
| Failed extract | “Processing failed” + short reason | |

## Progressive availability (target)

```ascii
  Admitted ──► Converting(PDF?) ──► Extracting ──► Ready
     │                                  │
     └──── cancel ──────────────────────┘
```

v1 Phase A: clarify states + bulk summary. Optional later: partial chunk search (out of scope unless product funds).

## A11y

- Status text not color-only.
- Live region for bulk “N of M completed”.
- Keyboard: cancel control focusable.

## Cross-refs

- UX lens: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md)
- Front lens: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
