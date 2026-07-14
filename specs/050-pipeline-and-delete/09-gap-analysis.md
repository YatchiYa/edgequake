# SPEC-050 Gap Analysis — Post-Implementation Audit

**Date:** 2026-07-13  
**Evidence:** Screenshots from live application showing two critical UX gaps.

## Gap 1: Reprocess — Row Does Not Update

### Observed Behaviour
After clicking Reprocess on a "Completed" document:
- The `ActiveRunsPanel` banner correctly shows "Queued — waiting for a free worker slot"
- The document **row itself still shows "Completed"** — no visual change

### Root Cause (Code is Law)

```
reprocessMutation.onMutate()
  → Optimistic update: status:"pending", current_stage:"queued"  ✓

reprocessMutation.onSuccess()
  → queryClient.invalidateQueries({ queryKey: ["documents"] })   ← FIRES IMMEDIATELY
  → Server refetch: document DB still has status:"completed"     ← NOT YET UPDATED
  → Optimistic state OVERWRITTEN by server "completed"           ← BUG
```

**Secondary issue:** `isDocumentActivelyProcessing()` in `use-document-queries.ts` does
NOT include `status === "pending"`, so the 2-second polling interval never activates
for queued documents. The 30-second fallback poll eventually catches it.

**Tertiary issue:** After `invalidateQueries` overwrites the optimistic state, the
WS subscription loop in `useDocumentWebSocket` loses the new `track_id` (which came
from the reprocess response). Without the correct `track_id`, the WS subscription
does not subscribe to the new task.

### Fix

1. In `reprocessMutation.onSuccess` — update cache WITH the new `track_id`, add 2s
   delay before `invalidateQueries`.
2. In `use-document-queries.ts` — add `"pending"` to `isDocumentActivelyProcessing`.
3. In `useDocumentWebSocket` — also subscribe when `status === "pending"`.

---

## Gap 2: Bulk Toolbar Delete — No Confirmation, No Impact Preview

### Observed Behaviour
- User selects 1 document via checkbox
- Clicks "Delete" in the `BatchActionsBar`
- Immediately fires `deleteDocument()` without any confirmation
- Shows "Deleting 1 of 1..." toast at bottom-right
- No impact preview, no abort chance

### Root Cause (Code is Law)

```
BatchActionsBar.onDelete
  → DocumentToolbarSection.onBulkDelete
  → DocumentManager.handleBulkDelete
  → useBulkSelection.handleBulkDelete()
  → deleteDocument(id) DIRECTLY — NO DIALOG
```

`useBulkSelection.handleBulkDelete` calls `deleteDocument(id)` directly,
bypassing the `DeleteConfirmDialog` added in SPEC-050 initial implementation.

### Fix

Introduce `BulkDeleteConfirmDialog` (simple list + count + confirm button).  
`useBulkSelection` gains an `onDeleteRequested(ids: string[])` callback.  
`DocumentManager` wires this to show the dialog, then calls `handleDeleteDocument`
per ID on confirm.

---

## Gap 3: Preview Panel Delete — Bypasses Confirm Dialog

### Observed Behaviour
Clicking "Delete" in the right preview panel calls `onDelete(id)` which calls
`handleDeleteDocument(id)` → `deleteMutation.mutate(id)` **without** opening
`DeleteConfirmDialog`.

### Root Cause

`DocumentManager` wires `DocumentPreviewRightPanel.onDelete` directly to
`handleDeleteDocument`, which calls the mutation immediately.

### Fix

Add `deleteConfirmTarget` state to `DocumentManager`.  
`onDelete` from the preview panel → set `deleteConfirmTarget`, which opens
`DeleteConfirmDialog`. On confirm → call `handleDeleteDocument`.

---

## Gap 4: Reprocess — WS Track Subscription Lost

After optimistic update overwrite (Gap 1), the new `track_id` from the
reprocess response is never stored in the cache. Even if polling recovers, the
WS subscription never subscribes to the new task's channel.

### Fix

Merged into Gap 1 Fix 1: set `track_id: data.track_id` in `onSuccess` cache update.

---

## Summary Table

| Gap                              | Severity | Location                                               | Fix                                                   |
| -------------------------------- | -------- | ------------------------------------------------------ | ----------------------------------------------------- |
| Reprocess row not updating       | High     | `use-document-mutations.ts`, `use-document-queries.ts` | Cache update + delayed invalidation + pending polling |
| Bulk toolbar delete — no dialog  | High     | `use-bulk-selection.ts`, `DocumentManager`             | `BulkDeleteConfirmDialog` + callback                  |
| Preview panel delete — no dialog | Medium   | `DocumentManager`                                      | `deleteConfirmTarget` state                           |
| WS track lost after reprocess    | High     | `use-document-mutations.ts`                            | Set `track_id` in cache on success                    |
