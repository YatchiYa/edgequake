# SPEC-050 — Implementation Plan

## Checklist

- [x] Spec documentation created (all lenses)
- [x] Backend: Add `DeletionPhaseKind` enum to websocket_types.rs
- [x] Backend: Add `DeletionStarted`, `DeletionPhase`, `DeletionCompleted` to `ProgressEvent`
- [x] Backend: Add `BulkDeletionStarted`, `BulkDeletionItemProgress`, `BulkDeletionCompleted` to `ProgressEvent`
- [x] Backend: Add `embeddings_deleted`, `partial_failure`, `partial_failure_reason` to `DeleteDocumentResponse`
- [x] Backend: Broadcast deletion events in `delete_document()` (single.rs)
- [x] Backend: Broadcast bulk events in `delete_all_documents()` (bulk.rs) with per-doc progress
- [x] Frontend: Create `use-deletion-impact.ts` hook
- [x] Frontend: Create `DeletionImpactCard` component
- [x] Frontend: Create `DeleteConfirmDialog` component
- [x] Frontend: Update `DocumentActionsMenu` → use `DeleteConfirmDialog`
- [x] Frontend: Update `useDocumentMutations` → optimistic `queued` on reprocess
- [x] Frontend: Update `DocumentManager` → track `deletingDocumentIds` set + `handleDeleteDocument`
- [x] Frontend: Update `DocumentTableRow` → accept `isDeleting` prop
- [x] Frontend: Update `DocumentTableSection` → accept `deletingDocumentIds` prop
- [x] Frontend: Update `ClearDocumentsDialog` → add per-document progress list via `useBulkDeletionProgress`
- [x] Frontend: Add `getDeletionImpact` API function + `DeletionImpact` type in `documents.ts`
- [x] Frontend: Add WS message types for deletion events to `ingestion.ts`
- [x] Frontend: Update `progress-websocket.ts` to handle new deletion message types
- [x] Frontend: Export new hooks and components from barrel files
- [x] E2E: Write delete-confirm-dialog.spec.ts
- [x] E2E: Write reprocess-parity.spec.ts
- [x] E2E: Write bulk-delete-progress.spec.ts
- [x] Validate: cargo check passes (0 errors)
- [x] Validate: 951 backend tests pass (3 pre-existing failures unrelated to SPEC-050)
- [x] Validate: 0 TypeScript source errors (only pre-existing e2e test errors)

## Edge Cases

| Edge case                                       | Handling                                                          |
| ----------------------------------------------- | ----------------------------------------------------------------- |
| Impact fetch fails (network error)              | Show "Impact unavailable" banner, still allow delete              |
| Impact fetch times out (> 5s)                   | Cancel fetch, show "Impact unavailable", still allow delete       |
| Document is 'processing' during delete          | Cancel task first (already done), show phase in delete panel      |
| Delete called on doc already deleted            | 404 → toast.error "Document not found"                            |
| Bulk delete with 0 documents                    | Show "No documents to delete" immediately                         |
| Bulk delete with all docs 'processing'          | Show "All documents are currently processing. Wait and retry."    |
| WS disconnected during delete                   | Delete still completes server-side; toast appears on success      |
| Reprocess while another reprocess is in-flight  | Block second reprocess via `isInflight` check (already in dialog) |
| Delete while deletion is in-flight              | Deduplicate: `deletingDocumentIds` prevents double-confirm        |
| Very large document (1000 chunks, 500 entities) | Same flow — phases just take longer, progress shown               |
| Partial failure (graph error, KV success)       | `partial_failure: true` in response → show warning toast          |

## File Change Map

```
edgequake/crates/edgequake-api/src/
  handlers/
    websocket_types.rs          ← ADD DeletionPhaseKind, new ProgressEvent variants
    documents_types/mutation.rs ← ADD embeddings_deleted, partial_failure fields
    documents/delete/single.rs  ← ADD broadcast calls throughout
    documents/delete/bulk.rs    ← ADD bulk progress broadcast calls

edgequake_webui/src/
  lib/api/edgequake/documents.ts    ← ADD getDeletionImpact(), DeletionImpact type
  hooks/
    use-deletion-impact.ts          ← NEW
    use-document-mutations.ts       ← UPDATE optimistic states
  components/documents/
    deletion-impact-card.tsx        ← NEW
    delete-confirm-dialog.tsx       ← NEW
    document-actions-menu.tsx       ← UPDATE to use DeleteConfirmDialog
    clear-documents-dialog.tsx      ← UPDATE with progress list
    document-table-row.tsx          ← UPDATE isDeleting prop
    enhanced-status-badge.tsx       ← UPDATE deleting/queued states
  stores/ or components/
    [DocumentManager uses local state for deletingDocumentIds]
```

## Gap Analysis (Post-Implementation Iteration 2)

All 4 gaps from `09-gap-analysis.md` are now fixed:

- [x] Gap 1: Reprocess row status — `onSuccess` updates cache with `track_id` + 2s delayed invalidation + `isDocumentActivelyProcessing` includes "pending"
- [x] Gap 2: Bulk toolbar delete — `BulkDeleteConfirmDialog` via `useBulkSelection.onDeleteRequested` callback
- [x] Gap 3: Preview panel delete — `deleteConfirmTarget` state in `DocumentManager` routes through `DeleteConfirmDialog`
- [x] Gap 4: WS track lost — `onSuccess` sets `track_id: data.track_id` in cache for WS subscription

## Shared Entity Edge Cases (SPEC-050/EC-*)

All 10 edge cases documented in `10-shared-entity-edge-cases.md` are handled correctly.

Proof: `cargo test -p edgequake-api --test resource_safety_proof` → 19/19 PASS
