# SPEC-051 — Implementation Plan

**SOLID + DRY principles applied throughout.**

---

## Work Packages

### WP-1: Document detail page — Reprocess button + dialog + progress panel (GAP-051-01, 03)

**Files touched:**
- `edgequake_webui/src/app/(dashboard)/documents/[id]/page.tsx`

**Changes:**
1. Import `ReprocessDialog`, `reprocessDocument`, `cancelTask`, `IngestionProgressPanel`
2. Add state: `reprocessDialogOpen`, `reprocessInProgressTrackId`
3. Add Reprocess button in header (for failed/cancelled/completed)
4. Add Cancel button in header (for processing)
5. Add inline `IngestionProgressPanel` strip below header when `reprocessInProgressTrackId` is set
6. Wire `staleTime` down to 3s when `status === 'pending' || status === 'processing'`

**SRP boundary:** `page.tsx` is the route entry point; it may embed a mini reprocess
coordination block because there is no `DocumentManager` wrapper on this page.

---

### WP-2: Bulk reprocess → IngestionProgressPanel tracking (GAP-051-02)

**Files touched:**
- `edgequake_webui/src/hooks/use-bulk-selection.ts`
- `edgequake_webui/src/app/(dashboard)/documents/document-manager.tsx`

**Changes:**
- `use-bulk-selection.ts`: Add `onReprocessTriggered?: (name: string, trackId: string) => void` to `UseBulkSelectionOptions`
- In `handleBulkReprocess`: call `onReprocessTriggered(docName, response.track_id)` after each success
- `document-manager.tsx`: Pass `onReprocessTriggered: addReprocessEntry` to `useBulkSelection`

---

### WP-3: Backend DRY — vision provider resolution (GAP-051-04)

**Files touched:**
- `edgequake/crates/edgequake-api/src/handlers/documents/recovery/reprocess.rs`

**Changes:**
- Extract `resolve_vision_settings_for_workspace(workspace_id, workspace_service, pdf_storage)` as a local async fn
- Use it in BOTH the KV path and the Postgres fallback path
- The Postgres path passes `pdf.workspace_id` as the workspace to resolve from

---

### WP-4: E2E test + screenshots (GAP-051 overall)

**Files created:**
- `edgequake_webui/e2e/spec051-reprocess.spec.ts`
- `specs/051-reprocess/e2e/screenshots/*.png`

---

## Definition of Done

- [ ] AC-01–AC-09 from analysis doc all green
- [ ] `cargo clippy` clean on touched crates
- [ ] `bun run typecheck` clean on touched files
- [ ] Screenshots present in `specs/051-reprocess/e2e/screenshots/`
- [ ] CHANGELOG updated
