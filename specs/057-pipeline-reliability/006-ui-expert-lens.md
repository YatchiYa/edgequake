# 006 — UI Expert Lens

**Spec:** SPEC-057  
**Key question:** Do visual states match backend truth for task / doc / PDF?

---

## Scope

Component states, badge matrix, ASCII wireframes. Identity: `track_id` (SPEC-056). Out of scope: new design system.

---

## Status badge matrix (target)

| Backend | Badge label | Color intent | Primary action |
| ------- | ----------- | ------------ | -------------- |
| pending | Queued | neutral | Cancel |
| processing (+ stage) | {Stage}… | info | Cancel |
| waiting capacity | Waiting for capacity | warning | Cancel |
| cancellation_requested / Stopping | Stopping… | warning | — (disabled) |
| cancelled | Cancelled | muted | Reprocess |
| failed + failure_class | Failed | danger | recommended_action CTA |
| completed / indexed | Ready | success | Open / Query |
| partial_failure | Partial | warning | Reprocess / Inspect |

**Law today:** Doc KV supports `cancelled`; PDF enum does not → badge may disagree on PDF rows.

---

## ASCII — pipeline status dialog states

```text
  ┌──────────────────────────────────────────────┐
  │  Ingestion · doc.pdf              [×]        │
  ├──────────────────────────────────────────────┤
  │  ● Uploading        done                     │
  │  ● Converting       12/40 pages              │
  │  ○ Extracting                                │
  │  ○ Embedding                                 │
  │  ○ Merging                                   │
  ├──────────────────────────────────────────────┤
  │  ETA ~8 min · Cost est. $0.12                │
  │                                              │
  │              [ Cancel ]                      │
  └──────────────────────────────────────────────┘

  After Cancel click:
  ┌──────────────────────────────────────────────┐
  │  ● Converting       Stopping…                │
  │              [ Cancel ]  (disabled)          │
  └──────────────────────────────────────────────┘

  Terminal Cancelled:
  ┌──────────────────────────────────────────────┐
  │  Status: Cancelled                           │
  │         [ Reprocess ]  [ Dismiss ]           │
  └──────────────────────────────────────────────┘
```

---

## Component call sites (code is law)

| Surface | File | Cancel wiring |
| ------- | ---- | ------------- |
| Documents manager | `document-manager.tsx` | `onCancel` → `cancelMutation.mutate(trackId)` |
| Mutations hook | `use-document-mutations.ts` | `cancelTask(trackId)` |
| API | `pipeline.ts` | `POST /tasks/${taskId}/cancel` |
| PDF progress | `use-pdf-progress.ts` | `cancelPdfProcessing(pdf_id)` — alias path |
| Detail page | `documents/[id]/page.tsx` | `cancelTask(document.track_id)` |
| Pipeline card | `pipeline-stages-card.tsx` | cancel mutation + `cancellation_requested` |

---

## UI defects to fix (no code in this pass)

1. **Single badge mapper** — one function maps API doc status + task status + PDF status → badge props (DRY).  
2. **Stopping…** — bind to `cancellation_requested` OR local optimistic flag until terminal.  
3. **Don’t count cancelled as failed** — `document-manager.tsx` currently may aggregate `failed + cancelled` for some counts; separate chips.  
4. **track_id required** — disable Cancel when track_id missing (SPEC-056).

---

## Recommendations → REQ

| UI change | REQ |
| --------- | --- |
| Badge SSOT + Cancelled PDF | REQ-057-03, 04 |
| Stopping… contract | REQ-057-05 |
| Separate cancelled counts | REQ-057-05 |
| Capacity-wait state | REQ-057-12 |

**Out of scope:** Dark-mode token audit; graph page chrome.

Next: [007-fullstack-expert-lens.md](./007-fullstack-expert-lens.md)
