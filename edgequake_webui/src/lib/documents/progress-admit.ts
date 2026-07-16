/**
 * Progress Admit Lifecycle (upload + reprocess) — DRY / SRP facade.
 *
 * First principles:
 *   1. Paint before network (beginAdmit sync)
 *   2. Poll only seeded task_id (classifier + Queuing panel)
 *   3. Pin until server-honest or deferred unpin
 *   4. documentId-stable UI identity
 *
 * Callers (DIP): use-bulk-selection, use-document-mutations, use-file-upload,
 * document detail page — import from this module, not path-specific helpers.
 */

export {
  REPROCESS_INVALIDATE_DELAY_MS,
  REPROCESS_OPTIMISTIC_FIELDS,
  REPROCESS_PENDING_PREFIX,
  abortProvisionalReprocess,
  admitQueuingToastId,
  applyReprocessSuccessToCache,
  beginProvisionalReprocess,
  clearDeferredUnpinTimersForTests,
  clearReprocessPinsForTests,
  documentIdsWithQueuingSession,
  filterRunsExcludingQueuingSession,
  formatReprocessSkipReasons,
  isPollableReprocessProgressTrackId,
  isProvisionalReprocessTrackId,
  isReprocessBatchTrackId,
  isReprocessPinned,
  patchDocumentsReprocessOptimistic,
  pinDocumentShell,
  pinReprocessDocuments,
  protectPinnedDocumentsInQueryData,
  provisionalReprocessTrackId,
  resolveReprocessPanelTrackId,
  resolveReprocessProgressTrackId,
  restoreDocumentFromSnapshots,
  scheduleDeferredUnpin,
  scheduleDocumentsInvalidate,
  shouldShowReprocessQueuingPanel,
  unpinReprocessDocuments,
  updateReprocessPinTrackId,
} from "./reprocess-cache";

/** Sync admit alias — pin + optimistic processing + provisional keys. */
export { beginProvisionalReprocess as beginAdmit } from "./reprocess-cache";

/** Bind live task_id after API success (deferred unpin). */
export { applyReprocessSuccessToCache as bindLiveTask } from "./reprocess-cache";

/** Abort admit alias — unpin + restore snapshot. */
export { abortProvisionalReprocess as abortAdmit } from "./reprocess-cache";
