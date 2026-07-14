# SPEC-050 — UX Designer Lens

## Current User Flow: Delete (AS-IS)

```
User clicks "..." menu on a row
  │
  ▼
  ┌──────────────────────────────────┐
  │ DropdownMenuItem: Delete         │
  └──────────────────────────────────┘
  │
  ▼
  Optimistic mutation fires immediately
  (no confirmation, no impact preview)
  │
  ▼
  ┌──────────────────────────────────┐
  │ toast.loading "Deleting…"        │  ← user has no idea what will be deleted
  └──────────────────────────────────┘
  │
  ▼ (500ms – 2s later)
  ┌──────────────────────────────────┐
  │ toast.success "Document deleted" │  ← no stats shown
  │ OR                               │
  │ toast.error "Delete failed"      │
  └──────────────────────────────────┘
  │
  ▼
  Row disappears (or stays on error)
```

**Problems:**  
- No confirmation step  
- No impact preview  
- No progress indication  
- No detail in success  
- Error state leaves row in indeterminate state

## Target User Flow: Delete (TO-BE)

```
User clicks "..." menu → "Delete"
  │
  ▼
  ┌──────────────────────────────────────────────────────────┐
  │ DeleteConfirmDialog opens                                │
  │                                                          │
  │  Loading impact…  (spinner while GET /deletion-impact)  │
  │                                                          │
  │  Impact loaded:                                          │
  │  ┌────────────────────────────────────────────┐         │
  │  │ This will permanently remove:              │         │
  │  │   📄 1 document                            │         │
  │  │   🔗 14 chunks  ·  34 vectors              │         │
  │  │   🏷  8 entities (removed from graph)       │         │
  │  │   ↔  12 relationships (removed)            │         │
  │  │   🔄  3 entities (updated, other sources)  │         │
  │  └────────────────────────────────────────────┘         │
  │                                                          │
  │  [ Cancel ]  [ Delete permanently ]                      │
  └──────────────────────────────────────────────────────────┘
  │
  ▼ User confirms
  ┌──────────────────────────────────────────────────────────┐
  │ Row enters "Deleting" state (dimmed, spinner badge)      │
  │                                                          │
  │ DeleteProgressPanel appears (below row or in dialog):   │
  │  ✓ Cancelling in-flight task                            │
  │  ⟳ Removing 34 vector embeddings   ← active, animated  │
  │  · Removing graph entities/edges                         │
  │  · Removing KV records                                   │
  │  · Finalizing                                            │
  └──────────────────────────────────────────────────────────┘
  │
  ▼ (~500ms–2s)
  Row removed from list with fade-out animation
  ┌──────────────────────────────────────────────────────────┐
  │ toast.success "Deleted — 8 entities, 12 relationships    │
  │  removed from the knowledge graph"                       │
  └──────────────────────────────────────────────────────────┘
```

## Current User Flow: Reprocess (AS-IS)

```
User clicks "Reprocess" in actions menu
  │
  ▼
  ReprocessDialog opens (full/entities choice)
  │
  User confirms
  │
  ▼
  Dialog closes
  │
  ▼
  Row status briefly shows OLD status
  (~2-5 second gap — no "queued" state)
  │
  ▼
  WebSocket event arrives → ServerStageStepper appears
  showing "preprocessing" or similar
```

**Problem:** The 2-5 second gap looks like the action failed.

## Target User Flow: Reprocess (TO-BE)

```
ReprocessDialog opens
  │
  User confirms
  │
  ▼
  Mutation fires
  │
  ▼
  Row IMMEDIATELY shows "Queued" badge (optimistic update)
  ┌─────────────────────────────────────────────────────────┐
  │ [filename]  [Queued ●]  — Waiting for worker            │
  └─────────────────────────────────────────────────────────┘
  │
  ▼ (WS event arrives)
  Same SPEC-048 stepper as first ingestion:
  ┌─────────────────────────────────────────────────────────┐
  │ [filename]  [Processing ●]                              │
  │  □ uploading · □ converting · ● extracting  · □ merging │
  │  Extracting: chunk 3/14 · 2 entities · $0.002           │
  └─────────────────────────────────────────────────────────┘
```

## Bulk Delete Flow (TO-BE)

```
User clicks "Clear All Documents"
  │
  ▼
  ClearDocumentsDialog opens
  │
  User types "DELETE ALL" to confirm
  │
  ▼
  ┌──────────────────────────────────────────────────────────┐
  │  Deleting documents…                                     │
  │                                                          │
  │  ████████████░░░░░░░░  7 / 23 deleted                    │
  │                                                          │
  │  ✓ doc-001-research.pdf                                  │
  │  ✓ doc-002-manual.txt                                    │
  │  ⟳ doc-003-spec.md  (removing graph…)                    │
  │  · doc-004… (pending)                                    │
  │                                                          │
  │  ⚠ 2 documents skipped (currently processing)           │
  └──────────────────────────────────────────────────────────┘
```

## Error States

```
Delete fails (partial):
  ┌──────────────────────────────────────────────────────────┐
  │ [filename]  [Partial failure ⚠]                         │
  │  ✓ Vector embeddings removed                            │
  │  ✗ Graph cascade failed: connection timeout             │
  │  ✓ KV records removed                                   │
  │                                                          │
  │  [Retry graph cleanup]  [Force delete anyway]            │
  └──────────────────────────────────────────────────────────┘
```

## State Machine: Delete Operation

```
                  ┌─────────────┐
                  │    Idle     │
                  └──────┬──────┘
                         │ user clicks Delete
                         ▼
                  ┌─────────────┐
                  │  Impact     │◄─── GET /deletion-impact
                  │  Loading    │
                  └──────┬──────┘
                         │ loaded
                         ▼
                  ┌─────────────┐
                  │  Impact     │
                  │  Preview    │◄─── shown in dialog
                  └──────┬──────┘
                    │         │
              cancel│         │confirm
                    ▼         ▼
                  Idle   ┌─────────────┐
                         │  Deleting   │◄─── mutation pending
                         │  (phases)   │
                         └──────┬──────┘
                           │         │
                       success│       │error
                           ▼         ▼
                  ┌─────────────┐ ┌─────────────┐
                  │  Removed    │ │   Failed    │
                  │  (fade out) │ │  (stays,    │
                  └─────────────┘ │   error UI) │
                                  └─────────────┘
```
