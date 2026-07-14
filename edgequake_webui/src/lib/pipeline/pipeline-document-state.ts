/**
 * Classify document pipeline states for consistent banner + dialog messaging.
 *
 * WHY: "pending" means waiting (post-recovery or queued), not active LLM work.
 * Workers may be idle while documents still need processing.
 */

import {
  getDocumentDisplayStatus,
  isProcessingStatus,
  type DocumentStatus,
} from '@/components/documents/status-badge';
import type { Document } from '@/types';
import type { IngestionAlertMode } from './ingestion-alert-presenter';

export const WAITING_STATUSES: DocumentStatus[] = ['pending', 'queued'];

export function isWaitingStatus(status: DocumentStatus): boolean {
  return WAITING_STATUSES.includes(status);
}

/** Active pipeline stage — worker should be doing real work. */
export function isActiveProcessingStatus(status: DocumentStatus): boolean {
  return isProcessingStatus(status) && !isWaitingStatus(status);
}

export interface PipelineDocumentSummary {
  activeCount: number;
  waitingCount: number;
  queuedCount: number;
  activeDocs: Document[];
  waitingDocs: Document[];
}

export function summarizePipelineDocuments(
  documents: Document[] | undefined,
): PipelineDocumentSummary {
  const activeDocs: Document[] = [];
  const waitingDocs: Document[] = [];
  let queuedCount = 0;

  for (const doc of documents ?? []) {
    const status = getDocumentDisplayStatus(doc);
    if (isActiveProcessingStatus(status)) {
      activeDocs.push(doc);
    } else if (isWaitingStatus(status)) {
      waitingDocs.push(doc);
      if (status === 'queued') {
        queuedCount += 1;
      }
    }
  }

  return {
    activeCount: activeDocs.length,
    waitingCount: waitingDocs.length,
    queuedCount,
    activeDocs,
    waitingDocs,
  };
}

/** Tasks pending in queue not already represented by waiting or active documents. */
export function orphanQueuedTaskCount(
  pipelineQueuedTasks: number,
  waitingDocCount: number,
  activeDocCount = 0,
): number {
  return Math.max(0, pipelineQueuedTasks - waitingDocCount - activeDocCount);
}

/** Task counters from pipeline status APIs (basic or enhanced). */
export interface PipelineTaskStats {
  is_busy?: boolean;
  queued_tasks?: number;
  running_tasks?: number;
  pending_tasks?: number;
  processing_tasks?: number;
}

/** True when workers or the task queue can still pick up waiting documents. */
export function hasQueueCoverage(
  pipeline: PipelineTaskStats | undefined,
  pendingTaskCount: number,
  processingTaskCount: number,
): boolean {
  return (
    pendingTaskCount > 0 ||
    processingTaskCount > 0 ||
    Boolean(pipeline?.is_busy)
  );
}

/** Waiting documents with no worker/task scheduled (document ↔ task desync).
 *
 * SPEC-048: Do NOT treat a fresh upload as stuck. New docs often appear as
 * pending/queued for a few seconds before the task row is visible — that is
 * normal Queued, not "Needs attention".
 *
 * Stuck requires: no queue coverage AND (aged past grace OR recovery signal).
 */
export const STUCK_GRACE_MS = 60_000;

const RECOVERY_STUCK_RE =
  /auto-recovered|no worker|needs?\s+reprocess|orphaned|server restart/i;

export function isRecoveryStuckSignal(doc: Document): boolean {
  const msg = `${doc.stage_message || ""} ${doc.error_message || ""}`;
  return RECOVERY_STUCK_RE.test(msg);
}

/** How long the document has been waiting (ms). Unknown timestamps → aged. */
export function documentWaitAgeMs(doc: Document, now = Date.now()): number {
  const ts = doc.updated_at || doc.created_at;
  if (!ts) return Number.POSITIVE_INFINITY;
  const parsed = Date.parse(ts);
  if (Number.isNaN(parsed)) return Number.POSITIVE_INFINITY;
  return Math.max(0, now - parsed);
}

export function detectStuckDocuments(
  summary: PipelineDocumentSummary,
  hasCoverage: boolean,
  now = Date.now(),
): Document[] {
  if (summary.waitingCount === 0 || hasCoverage) {
    return [];
  }
  return summary.waitingDocs.filter((doc) => {
    // Explicit recovery / orphan copy → stuck even inside grace (server said so)
    if (isRecoveryStuckSignal(doc)) {
      return true;
    }
    // Fresh upload / just-queued: keep amber Queued, never red Needs attention
    if (documentWaitAgeMs(doc, now) < STUCK_GRACE_MS) {
      return false;
    }
    // Still has an active track — pipeline admitted the work; prefer Queued
    if (doc.track_id) {
      return false;
    }
    return true;
  });
}

/** Unified banner + dialog pipeline UI state (document truth + task queue). */
export interface PipelineUiState {
  activeDocCount: number;
  waitingDocCount: number;
  processingTaskCount: number;
  pendingTaskCount: number;
  isActivelyProcessing: boolean;
  /** @deprecated Prefer alertMode / isQueuedOnly — true when queued, not stuck */
  isWaitingOnly: boolean;
  isQueuedOnly: boolean;
  isStuck: boolean;
  stuckDocCount: number;
  stuckDocs: Document[];
  alertMode: IngestionAlertMode;
  showPipelineIndicator: boolean;
}

function resolveAlertMode(
  summary: PipelineDocumentSummary,
  isActivelyProcessing: boolean,
  isStuck: boolean,
  isQueuedOnly: boolean,
): IngestionAlertMode {
  if (isActivelyProcessing && summary.waitingCount > 0) {
    return 'mixed';
  }
  if (isActivelyProcessing) {
    return 'working';
  }
  if (isStuck) {
    return 'stuck';
  }
  if (isQueuedOnly) {
    return 'queued';
  }
  // No active / stuck / queued signal — caller must hide the indicator.
  return 'queued';
}

/**
 * Single source for pipeline header, banner, and dialog modes.
 * Documents in pending/queued win over idle task statistics.
 */
export function resolvePipelineUiState(
  documents: Document[] | undefined,
  pipeline?: PipelineTaskStats,
): PipelineUiState {
  const summary = summarizePipelineDocuments(documents);
  const pendingTaskCount =
    pipeline?.pending_tasks ?? pipeline?.queued_tasks ?? 0;
  const processingTaskCount =
    pipeline?.processing_tasks ?? pipeline?.running_tasks ?? 0;

  const orphanQueued = orphanQueuedTaskCount(
    pendingTaskCount,
    summary.waitingCount,
    summary.activeCount,
  );
  const waitingDocCount = summary.waitingCount + orphanQueued;

  const queueCoverage = hasQueueCoverage(
    pipeline,
    pendingTaskCount,
    processingTaskCount,
  );

  // First principle: "Processing N document(s)" requires document evidence when
  // the list is loaded and fully terminal. Stale `processing_tasks` / `is_busy`
  // must NOT keep the working banner after every doc is ingested.
  // Empty list still trusts task counters (docs may lag the queue briefly).
  const docsFullyIdle =
    (documents?.length ?? 0) > 0 &&
    summary.activeCount === 0 &&
    summary.waitingCount === 0;
  const isActivelyProcessing =
    summary.activeCount > 0 ||
    (processingTaskCount > 0 && !docsFullyIdle);

  const stuckDocs = detectStuckDocuments(summary, queueCoverage);
  const isStuck = !isActivelyProcessing && stuckDocs.length > 0;
  const isQueuedOnly =
    !isActivelyProcessing && waitingDocCount > 0 && !isStuck;

  const alertMode = resolveAlertMode(
    summary,
    isActivelyProcessing,
    isStuck,
    isQueuedOnly,
  );

  // Prefer document count; fall back to running tasks when list lags the queue.
  // When fully idle, never surface a stale task count as "Processing N".
  const displayActiveCount = isActivelyProcessing
    ? summary.activeCount > 0
      ? summary.activeCount
      : processingTaskCount
    : 0;

  return {
    activeDocCount: displayActiveCount,
    waitingDocCount,
    processingTaskCount,
    pendingTaskCount,
    isActivelyProcessing,
    isWaitingOnly: isQueuedOnly,
    isQueuedOnly,
    isStuck,
    stuckDocCount: stuckDocs.length,
    stuckDocs,
    alertMode,
    showPipelineIndicator: isActivelyProcessing || isStuck || isQueuedOnly,
  };
}
