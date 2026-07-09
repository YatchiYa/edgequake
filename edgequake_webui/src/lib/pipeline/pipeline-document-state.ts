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
