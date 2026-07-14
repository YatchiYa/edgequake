# SPEC-051 — Code-is-Law Gap Analysis

**Principle:** The code is the ground truth. Every gap is anchored to a specific symbol.

---

## 1. Reprocess access surface inventory

| Surface                                         | Location                                                | Has Reprocess  |      Has Mode Dialog       |     Fires onReprocessTriggered     | IngestionProgressPanel |
| ----------------------------------------------- | ------------------------------------------------------- | :------------: | :------------------------: | :--------------------------------: | :--------------------: |
| Documents list — table row `onReprocess`        | `document-manager.tsx:L462`                             |       ✅        |    ✅ `ReprocessDialog`     |     ✅ via `reprocessMutation`      |           ✅            |
| Documents list — table row `onRetry`            | `document-manager.tsx:L455`                             |     ✅ fast     | ❌ no dialog, entities only |     ✅ via `reprocessMutation`      |           ✅            |
| Documents list — preview panel `onReprocess`    | `document-manager.tsx:L501`                             |       ✅        |    ✅ `ReprocessDialog`     |     ✅ via `reprocessMutation`      |           ✅            |
| Documents list — bulk toolbar `onBulkReprocess` | `use-bulk-selection.ts:L275`                            |       ✅        |  ✅ `BulkReprocessDialog`   | **❌** direct `reprocessDocument()` |     **❌ missing**      |
| Documents list — stuck CTA `onReprocessStuck`   | `document-manager.tsx:L431`                             |  ✅ full only   |        ❌ no dialog         |     ✅ via `reprocessMutation`      |           ✅            |
| Document detail page `/documents/[id]`          | `app/(dashboard)/documents/[id]/page.tsx`               | **❌ MISSING**  |           **❌**            |               **❌**                |         **❌**          |
| `ResetDocumentStatusButton`                     | `components/documents/reset-document-status-button.tsx` | ✅ (retry+full) |      **❌ no dialog**       |               **❌**                |         **❌**          |

---

## 2. Gaps (Code anchors)

### GAP-051-01 — No reprocess action on document detail page  
**Severity: CRITICAL**

**Evidence:**
```tsx
// app/(dashboard)/documents/[id]/page.tsx:L370-L374
{isCancelled && (
  <div className="px-3 py-2 bg-muted/50 border-t">
    <p className="text-xs text-muted-foreground">
      {t('documents.cancelled.message',
         'Processing was cancelled. You can reprocess this document from the documents list.')}
    </p>
  </div>
)}
```
User is explicitly told to go back to the list. No Reprocess button exists on the detail page.

**Law:**
- `isFailed`, `isCancelled`, `status === 'completed'` are all computed but only used to show badges
- The header action buttons are: `DocumentDownloadMenu`, graph view button — no reprocess
- `StopCircle` is imported but never used in a cancel button

**Impact:** User on detail page of a failed document must navigate away and find the document in the list to reprocess. Broken flow.

---

### GAP-051-02 — Bulk reprocess bypasses IngestionProgressPanel tracking  
**Severity: HIGH**

**Evidence:**
```ts
// hooks/use-bulk-selection.ts:L278-L300 (handleBulkReprocess)
const response = await reprocessDocument(id, true, mode);
// ← onReprocessTriggered is NEVER called here
// ← IngestionProgressPanel never appears for bulk-reprocessed docs
```

Compare with the single-doc path:
```ts
// hooks/use-document-mutations.ts:L320-L324 (reprocessMutation.onSuccess)
if (onReprocessTriggered) {
  const displayName = name ?? documentId.slice(0, 8);
  onReprocessTriggered(displayName, data.track_id);  // ← fires IngestionProgressPanel
}
```

`useBulkSelection` calls `reprocessDocument()` directly, bypassing `useDocumentMutations`.

**Law:**
- `useBulkSelection` does not accept an `onReprocessTriggered` callback
- `DocumentManager.L479`: `onBulkReprocess={() => setBulkReprocessOpen(true)}` → `handleBulkReprocess(choice.mode)` 
- No `addReprocessEntry(name, trackId)` call anywhere in the bulk path

---

### GAP-051-03 — No live progress panel on document detail page during reprocess  
**Severity: HIGH**

**Evidence:**
```tsx
// app/(dashboard)/documents/[id]/page.tsx:L360
const { data: document, ... } = useQuery({
  queryKey: ['document', documentId, selectedWorkspaceId],
  staleTime: 30 * 1000,  // ← 30s stale time: no updates for 30 seconds!
});
// ← No IngestionProgressPanel when status === 'pending'/'processing'
// ← No WebSocket subscription on the detail page
// ← No cancel button when processing
```

Compare with documents list:
```tsx
// hooks/use-document-websocket.ts — used ONLY in document-manager.tsx
// Not wired to the detail page at all.
```

**Law:** `useDocumentWebSocket` is only called from `document-manager.tsx` (L232). The detail page has no real-time subscription after a reprocess.

---

### GAP-051-04 — Backend DRY violation: dual vision provider resolution  
**Severity: MEDIUM**

**Evidence:**

*KV path* (uses workspace settings ✅):
```rust
// edgequake/crates/edgequake-api/src/handlers/documents/recovery/reprocess.rs:L300-L320
let (vision_provider, vision_model, pdf_parser_backend) =
  if let Ok(ws_uuid) = uuid::Uuid::parse_str(&workspace_id) {
    if let Ok(Some(ws)) = state.workspace_service.get_workspace(ws_uuid).await {
      let vp = ws.vision_llm_provider...;
      let vm = ws.vision_llm_model...;
      (vp, vm, ws.resolved_pdf_parser_backend())
    } ...
```

*Postgres fallback path* (uses defaults ❌):
```rust
// edgequake/crates/edgequake-api/src/handlers/documents/recovery/reprocess.rs:L560-L563
let vision_opts = crate::handlers::pdf_upload::types::PdfUploadOptions::default();
let vision_provider = vision_opts.resolved_vision_provider();
let vision_model: Option<String> = Some(vision_opts.vision_model());
```

**Law:** Two code paths, two provider resolution strategies. Postgres path ignores workspace vision model.  
SSOT: `workspace_service.get_workspace()` — not used in the Postgres fallback.

---

### GAP-051-05 — `ResetDocumentStatusButton` hardcodes mode, bypasses dialog  
**Severity: MEDIUM**

**Evidence:**
```tsx
// components/documents/reset-document-status-button.tsx:L107-L118
const reprocessMutation = useMutation({
  mutationFn: () => {
    if (!document.id) throw new Error('No document id available for reprocessing');
    return reprocessDocument(document.id, true, 'full');  // ← hardcoded 'full'
  },
  // ← no onReprocessTriggered callback
  // ← no IngestionProgressPanel shown
```

**Primary path** (retryTask) also has no `IngestionProgressPanel`:
```tsx
// reset-document-status-button.tsx:L72-L84
const retryMutation = useMutation({
  mutationFn: async () => {
    if (document.track_id) {
      await retryTask(document.track_id);
      return { success: true };
    }
    throw new Error('No track_id available for reprocessing');
  },
  // ← no onReprocessTriggered callback
  // ← no progress panel shown
```

---

## 3. Architecture comparison: upload vs reprocess

```
UPLOAD (complete path):
  dropzone.onDrop()
    → useFileUpload.handleFilesUpload()
    → uploadFile() → POST /documents
    → backend assigns track_id
    → addUploadingFile({ trackId, ... })
    → UploadProgressList/ActiveRunsPanel shows IngestionProgressPanel
    → WebSocket updates document.track_id in real-time
    → pruneTerminalUploads() cleans up on completion

REPROCESS single doc (current good path):
  ReprocessDialog.onConfirm()
    → reprocessMutation.mutate({ id, mode, name })
    → POST /documents/reprocess
    → onSuccess: addReprocessEntry(name, track_id)
    → IngestionProgressPanel shown ✅
    → WebSocket updates via new track_id ✅
    → pruneTerminalReprocessEntries() cleans up ✅

REPROCESS bulk (BROKEN path):
  BulkReprocessDialog.onConfirm()
    → handleBulkReprocess(mode)
    → for each id: reprocessDocument(id, true, mode)
    → response.track_id DISCARDED ← GAP-051-02
    → NO IngestionProgressPanel ← GAP-051-02

REPROCESS from detail page (MISSING):
  No button exists ← GAP-051-01
  After reprocess (if triggered from list), no progress ← GAP-051-03
```

---

## 4. Acceptance criteria for SPEC-051

| ID    | Criterion                                                                                  |
| ----- | ------------------------------------------------------------------------------------------ |
| AC-01 | Document detail page shows Reprocess button for status ∈ {failed, cancelled, completed}    |
| AC-02 | Clicking Reprocess on detail page opens ReprocessDialog with mode selection                |
| AC-03 | After confirming reprocess on detail page, IngestionProgressPanel appears inline           |
| AC-04 | Cancel button appears on detail page when document is processing (status === 'processing') |
| AC-05 | Bulk reprocess: IngestionProgressPanel appears for each reprocessed document               |
| AC-06 | Backend: Postgres fallback path resolves vision provider from workspace settings           |
| AC-07 | Detail page polling interval activates when status is 'pending' (≤ 2s poll)                |
| AC-08 | E2E test covers: trigger reprocess from detail page → progress panel → completion          |
| AC-09 | E2E screenshots saved to specs/051-reprocess/e2e/screenshots/                              |
