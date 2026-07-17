# 005 — UX Expert Lens

**Spec:** SPEC-057  
**Key question:** Can a user predict, observe, and interrupt ingestion without confusion?

---

## Scope

Mental models for upload → progress → cancel → fail → reprocess. Cross-ref SPEC-048 / SPEC-050 for progress parity. Out of scope: visual polish tokens (see UI lens).

---

## Primary user journeys

```text
  Happy path
  Upload ──► stages advance ──► Completed/Indexed ──► query

  Cancel path (target)
  Upload ──► Cancel ──► "Stopping…" ──► Cancelled ──► optional Reprocess

  Restart path (default product policy)
  Crash ──► doc not silently running ──► "Interrupted — Reprocess"
           (auto-resume OFF unless ops enables it)

  Fail path
  Fail ──► failure_class + recommended_action ──► guided next step
```

---

## Findings

### Strengths

- Canonical cancel API exists; documents manager wires `cancelMutation` to `track_id`.  
- Pipeline stages / cost / ETA work exists (SPEC-048 lineage).  
- Reprocess is the intended recovery verb when auto-resume is off.

### Risks

| UX risk | Evidence | User perception |
| ------- | -------- | --------------- |
| Cancel delay | Cooperative cancel waits on HTTP RTT | “Cancel does nothing” |
| Failed ≠ Cancelled | PDF status → Failed on cancel | “I cancelled but it failed” |
| Dual cancel APIs | `cancelTask` vs `cancelPdfProcessing` | Inconsistent Stopping… states |
| Restart amnesia | Channel drop + auto-resume off | “Where did my upload go?” |
| Parked fairness | No user-visible “waiting for tenant slot” | “Stuck at 0%” |
| Fake upload progress (historical) | SPEC-038 noted fixed 40% | Distrust of all progress bars |

---

## UX principles (enforce)

1. **Immediate acknowledgement** — Cancel click → Stopping… within 100ms (optimistic UI), then poll/WS until terminal.  
2. **One verb** — Cancel always means terminal Cancelled, never Failed.  
3. **Name the wait** — Distinguish *converting*, *extracting*, *waiting for capacity* (park).  
4. **Recovery is obvious** — Interrupted/cancelled/failed each show one primary CTA.  
5. **Don’t surprise with auto-spend** — Auto-resume must be an ops toggle, not silent UI behavior.

---

## Recommendations → REQ

| Recommendation                                       | REQ            |
| ------------------------------------------------------| ----------------|
| Unify cancel UX on `POST /tasks/{track_id}/cancel`   | REQ-057-05     |
| Map all terminals through one status story           | REQ-057-03, 04 |
| Surface park/waiting-for-capacity in progress dialog | REQ-057-09, 12 |
| Interrupted-after-restart empty state → Reprocess    | REQ-057-01, 05 |
| failure_class copy in fail panel                     | REQ-057-13     |

**Out of scope:** Full redesign of documents table; i18n expansion beyond cancel/fail strings.

Next: [006-ui-expert-lens.md](./006-ui-expert-lens.md)
