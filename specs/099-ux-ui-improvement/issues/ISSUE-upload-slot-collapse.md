# ISSUE — Upload slot collapse + toast demotion

| Field | Value |
|-------|-------|
| ID | ISSUE-upload-slot-collapse |
| Findings | F-099-04, F-099-03 |
| Laws | LAW-099-4, LAW-099-6 |
| Wave | W3 |
| Status | Open |
| Inherits | SPEC-048 quiet dropzone |

## Problem

`document-dropzone.tsx` `quiet` mode densifies padding/copy but still reserves a full toolbar band. Concurrently `use-file-upload.ts` fires `toast.loading("Uploading N file(s)...")` while Active runs already list the same files.

## Why it hurts UX

Busy viewport loses inventory height; toast is a third status SSOT. SPEC-030 F-DOC-07 noted upload placement; SPEC-048 quiet was incomplete.

## Approach

```ascii
IDLE / empty:     [======== expanded dropzone ========]
BUSY (zone live): [⬆ Add files]  (collapsed drag-target, data-collapsed=true)
```

1. When `FeedbackZone.hasLiveWork`, set collapse (stronger than quiet).  
2. Retain click-to-upload + drag-over expand/highlight.  
3. If zone lists session file ids, skip or dismiss the loading toast (toast XOR zone).  
4. Keep `data-testid="document-dropzone"`; add `data-collapsed`.  
5. Fix audit selectors (F-099-13) in W8.

## DoD

- [ ] `spec099-upload-collapse` green  
- [ ] `spec099-toast-demotion` green  
- [ ] `spec048` quiet/collapse coexistence documented  
- [ ] `spec350-bulk-upload-webui` green  
- [ ] EC-099-12 keyboard path works  

## Non-goals

Modal-only upload; removing drag-and-drop.
