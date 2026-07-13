# SPEC-050 — Product Owner Lens

## 5 WHYs: Why Is Delete Feedback Critical?

```
Why #1: Users are deleting documents but don't know what happened
  └─ Why? The delete action shows a brief toast — no impact preview
       └─ Why? Impact endpoint exists (GET /deletion-impact) but is never called by the UI
            └─ Why? The delete UI was built quickly without impact-first UX
                 └─ Why? The team prioritised ingestion richness over deletion parity
                      └─ ROOT CAUSE: Asymmetry in the treat-destructive-operations-as-first-class
                                     principle — ingest is first-class, delete is an afterthought.
```

```
Why #1: Users re-processing a failed document see "processing" and nothing else
  └─ Why? The ServerStageStepper only appears after the WS update arrives
       └─ Why? There is a ~2-5 second lag between reprocess mutation and first WS event
            └─ Why? The task gets queued, a worker picks it up, first event is emitted
                 └─ Why? No "queued" visual state is shown immediately on confirm
                      └─ ROOT CAUSE: Reprocess dialog closes and the row reverts to
                                     whatever the old status was until the WS event arrives.
```

## Business Value

| Problem                         | User Impact                                                | Business Risk                       |
| ------------------------------- | ---------------------------------------------------------- | ----------------------------------- |
| No impact preview before delete | Users accidentally remove entities shared across documents | Data loss, lost trust               |
| No staged delete progress       | Users don't know if "Deleting…" is working for large docs  | Frustration, duplicate clicks       |
| Bulk delete is opaque           | No per-document progress for hundreds of docs              | SRE can't triage stuck bulk deletes |
| Reprocess lag state             | Users re-click reprocess thinking it didn't work           | Double-processing, wasted LLM cost  |
| Re-process ≠ first ingest UX    | Users don't know reprocess ran the same pipeline           | Confusion, support tickets          |

## Jobs-to-Be-Done

1. **"When I delete a document, I want to see exactly what will be removed."**  
   → Impact preview: N entities, N relationships, N chunks, N embeddings.

2. **"When I delete a document, I want to see deletion progressing, not just a spinner."**  
   → Phase stepper: cancelling → removing embeddings → removing graph → removing KV → done.

3. **"When I reprocess a document, I want the same rich stage view I see on first upload."**  
   → Immediate "queued" state, then SPEC-048 stepper with same stage granularity.

4. **"When I clear all documents, I want to see how many have been deleted so far."**  
   → Bulk progress: X/N documents deleted, showing per-document as it goes.

## Acceptance Criteria (Product)

- [ ] **AC-050-01**: Confirm-delete dialog shows entity/relationship/chunk impact counts loaded before the user can confirm.
- [ ] **AC-050-02**: After confirming single delete, a phase stepper appears in-row or in a toast showing each deletion phase completing.
- [ ] **AC-050-03**: On reprocess confirm, the row immediately shows "Queued" state (no lag gap).
- [ ] **AC-050-04**: Reprocess progress uses the same SPEC-048 stepper as first-time ingestion.
- [ ] **AC-050-05**: Bulk-delete dialog shows a progress counter (X/N deleted) updating in real-time.
- [ ] **AC-050-06**: If deletion partially fails (e.g., graph cascade error), the row shows "Partial failure" with error detail, not empty.
- [ ] **AC-050-07**: All deletion and reprocess operations are covered by E2E tests.
