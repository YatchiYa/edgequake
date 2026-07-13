# SPEC-050 — Full-Stack Developer Lens

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  Browser (Next.js)                                                                  │
│                                                                                     │
│  DocumentManager                                                                    │
│    │                                                                                │
│    ├── DocumentTableRow (+ isDeleting prop)                                         │
│    │    ├── EnhancedStatusBadge  ("Deleting ⟳" new state)                          │
│    │    └── ServerStageStepper  (existing — same for reprocess)                    │
│    │                                                                                │
│    ├── DocumentActionsMenu                                                          │
│    │    └── onClick Delete  ──► setDeleteTarget(doc)                               │
│    │                                                                                │
│    ├── DeleteConfirmDialog  ◄── NEW                                                 │
│    │    ├── DeletionImpactCard  ◄── NEW                                             │
│    │    │    └── GET /api/v1/documents/{id}/deletion-impact                         │
│    │    └── onConfirm → deleteMutation.mutate()                                     │
│    │                                                                                │
│    └── useDocumentMutations                                                         │
│         ├── deleteMutation (adds optimistic deleting state)                         │
│         └── reprocessMutation (adds optimistic queued state)                        │
└──────────────────────────────────────────────┬──────────────────────────────────────┘
                                               │ HTTP + WebSocket
┌──────────────────────────────────────────────▼──────────────────────────────────────┐
│  Axum API (Rust)                                                                    │
│                                                                                     │
│  DELETE /api/v1/documents/{id}                                                      │
│    └── delete_document()                                                            │
│         ├── resolve_kv_key_prefix()                                                 │
│         ├── [NEW] broadcast DeletionStarted event                                   │
│         ├── cancel in-flight task (if processing)                                   │
│         ├── [NEW] broadcast DeletionPhase::RemovingVectors                         │
│         ├── cascade_remove_document_sources() → graph + vectors                     │
│         ├── [NEW] broadcast DeletionPhase::RemovingKV                              │
│         ├── delete KV keys                                                          │
│         ├── [NEW] broadcast DeletionPhase::Finalizing                              │
│         ├── [NEW] broadcast DeletionCompleted with stats                            │
│         └── return DeleteDocumentResponse (unchanged shape)                         │
│                                                                                     │
│  GET /api/v1/documents/{id}/deletion-impact  (EXISTING — wire to UI)                │
│                                                                                     │
│  DELETE /api/v1/documents  (bulk)                                                   │
│    └── delete_all_documents()                                                       │
│         ├── [NEW] broadcast BulkDeletionStarted {total}                             │
│         ├── per-document loop:                                                      │
│         │    └── [NEW] broadcast BulkDeletionItemProgress {n, total, doc_id}        │
│         └── [NEW] broadcast BulkDeletionCompleted                                   │
│                                                                                     │
│  WS /ws/pipeline/progress  ◄── ProgressEvent new variants                          │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## New WebSocket Event Variants (Rust)

```rust
// edgequake-api/src/handlers/websocket_types.rs — add to ProgressEvent enum:

/// Single document deletion started.
DeletionStarted {
    document_id: String,
    track_id: String,
},

/// Deletion phase progress.
DeletionPhase {
    document_id: String,
    track_id: String,
    phase: DeletionPhaseKind,
    phase_label: String,
    items_processed: u32,
    items_total: u32,
},

/// Single document deletion completed.
DeletionCompleted {
    document_id: String,
    track_id: String,
    chunks_deleted: usize,
    entities_removed: usize,
    relationships_removed: usize,
    embeddings_deleted: usize,
    partial_failure: bool,
    error: Option<String>,
},

/// Bulk deletion started.
BulkDeletionStarted { total: usize },

/// Per-document progress in a bulk deletion.
BulkDeletionItemProgress {
    document_id: String,
    completed: usize,
    total: usize,
    entities_removed: usize,
    relationships_removed: usize,
},

/// Bulk deletion finished.
BulkDeletionCompleted {
    deleted_count: usize,
    skipped_count: usize,
    total_entities_removed: usize,
    total_relationships_removed: usize,
},
```

```rust
// New enum for deletion phases:
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeletionPhaseKind {
    CancellingTask,
    RemovingVectors,
    RemovingGraph,
    RemovingKv,
    Finalizing,
}

impl DeletionPhaseKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::CancellingTask => "Cancelling in-flight task",
            Self::RemovingVectors => "Removing vector embeddings",
            Self::RemovingGraph => "Removing graph entities & edges",
            Self::RemovingKv => "Removing document records",
            Self::Finalizing => "Finalizing",
        }
    }
}
```

## API Contracts

### GET /api/v1/documents/{id}/deletion-impact (existing)

```json
{
  "document_id": "abc123",
  "chunks_to_delete": 14,
  "entities_to_remove": 8,
  "entities_to_update": 3,
  "relationships_to_remove": 12,
  "relationships_to_update": 5,
  "preview_only": true
}
```

### DELETE /api/v1/documents/{id} (enhanced response)

Add `track_id` and `partial_failure` to `DeleteDocumentResponse`:

```json
{
  "document_id": "abc123",
  "deleted": true,
  "chunks_deleted": 14,
  "entities_affected": 8,
  "relationships_affected": 12,
  "embeddings_deleted": 34,
  "partial_failure": false,
  "partial_failure_reason": null
}
```

## Frontend Data Flow

```
DeleteConfirmDialog opens for doc.id
  │
  ├─ useQuery(['deletion-impact', doc.id]) → GET /deletion-impact
  │   └─ show DeletionImpactCard while loading / loaded
  │
  └─ onConfirm()
       ├─ setDeletingIds(prev => new Set([...prev, doc.id]))  [optimistic]
       ├─ deleteMutation.mutate(doc.id)
       │    onSuccess → setDeletingIds(prev => { prev.delete(doc.id); ... })
       │    onError   → setDeletingIds(prev => { prev.delete(doc.id); ... })
       └─ toast shows stats from response
```

## Component Props

```typescript
// DeleteConfirmDialog
interface DeleteConfirmDialogProps {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  document: Pick<Document, 'id' | 'title' | 'file_name'> | null;
  onConfirm: (docId: string) => void;
}

// DeletionImpactCard
interface DeletionImpactCardProps {
  impact: DeletionImpact | null;
  isLoading: boolean;
  error?: Error | null;
}

// DeletionImpact (TypeScript mirror of API response)
interface DeletionImpact {
  document_id: string;
  chunks_to_delete: number;
  entities_to_remove: number;
  entities_to_update: number;
  relationships_to_remove: number;
  relationships_to_update: number;
  preview_only: boolean;
}
```

## Hooks

```typescript
// NEW: use-deletion-impact.ts
export function useDeletionImpact(documentId: string | null) {
  return useQuery({
    queryKey: ['deletion-impact', documentId],
    queryFn: () => getDeletionImpact(documentId!),
    enabled: !!documentId,
    staleTime: 30_000,  // 30s — impact doesn't change rapidly
    retry: 1,
  });
}
```

## DRY Principles Applied

1. **`DeletionImpactCard`** is a pure display component — no fetching. The hook handles fetching, the dialog composes them.
2. **Phase stepper display** reuses the same color token function from `ServerStageStepper` — no duplication.
3. **`useDocumentMutations`** owns ALL mutation logic (already exists) — new optimistic state is added there, not in each component.
4. **Bulk progress list** reuses `DeletionImpactCard` per-item (compact variant).

## SOLID Principles Applied

| Principle                 | Application                                                                                           |
| ------------------------- | ----------------------------------------------------------------------------------------------------- |
| **S**ingle Responsibility | `DeleteConfirmDialog` only shows impact + gets confirm. Mutation is in the hook.                      |
| **O**pen/Closed           | `ProgressEvent` enum is extended with new variants — no existing handlers need changes.               |
| **L**iskov Substitution   | `DeleteProgressPanel` has same `onComplete?` / `onFailed?` interface as `IngestionProgressPanel`.     |
| **I**nterface Segregation | `DeletionImpactCard` only depends on `DeletionImpact` — not on the full `Document` type.              |
| **D**ependency Inversion  | Delete dialog depends on `useDeletionImpact` abstraction, not on `getDeletionImpact` API fn directly. |
