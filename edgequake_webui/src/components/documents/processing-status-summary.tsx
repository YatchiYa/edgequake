/**
 * ProcessingStatusSummary - Shows pipeline processing status
 *
 * @fileoverview Extracted from DocumentManager (OODA-11)
 * WHY: SRP - Processing status display is distinct responsibility
 *
 * @module edgequake_webui/components/documents/processing-status-summary
 */
'use client';

import {
  orphanQueuedTaskCount,
  summarizePipelineDocuments,
} from '@/lib/pipeline/pipeline-document-state';
import type { Document, PipelineStatus } from '@/types';
import { CheckCircle, Clock, Loader2 } from 'lucide-react';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

export interface ProcessingStatusSummaryProps {
  /** Pipeline status from API query */
  pipelineStatus: PipelineStatus;
  /** Documents to show processing details for */
  documents: Document[];
  /** Callback when user clicks to see details */
  onOpenDetails: () => void;
}

/**
 * ProcessingStatusSummary - Compact processing status display
 *
 * Distinguishes active worker stages from waiting/queued documents so the
 * banner does not say "Processing" when the pipeline worker is idle.
 */
export function ProcessingStatusSummary({
  pipelineStatus,
  documents,
  onOpenDetails,
}: ProcessingStatusSummaryProps) {
  const { t } = useTranslation();

  const { activeCount, waitingCount, activeDocs, waitingDocs } = useMemo(
    () => summarizePipelineDocuments(documents),
    [documents],
  );

  const extraQueuedTasks = orphanQueuedTaskCount(
    pipelineStatus.queued_tasks,
    waitingCount,
    activeCount,
  );
  const totalWaiting = waitingCount + extraQueuedTasks;

  const shouldShow =
    activeCount > 0 || totalWaiting > 0 || pipelineStatus.queued_tasks > 0;
  if (!shouldShow) return null;

  const waitingOnly = activeCount === 0 && totalWaiting > 0;
  const headline =
    activeCount > 0 && totalWaiting > 0
      ? t(
          'pipeline.processingAndWaiting',
          '{{processing}} processing · {{waiting}} waiting',
          { processing: activeCount, waiting: totalWaiting },
        )
      : activeCount > 0
        ? t('pipeline.processing', 'Processing {{count}} document(s)', {
            count: activeCount,
          })
        : t('pipeline.waitingToStart', '{{count}} document(s) waiting to start', {
            count: totalWaiting,
          });

  const containerClass = waitingOnly
    ? 'flex flex-col gap-2 px-3 py-2 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 rounded-lg cursor-pointer hover:bg-amber-100 dark:hover:bg-amber-950/50 transition-colors'
    : 'flex flex-col gap-2 px-3 py-2 bg-blue-50 dark:bg-blue-950/30 border border-blue-200 dark:border-blue-800 rounded-lg cursor-pointer hover:bg-blue-100 dark:hover:bg-blue-950/50 transition-colors';

  const titleClass = waitingOnly
    ? 'text-sm font-medium text-amber-800 dark:text-amber-300'
    : 'text-sm font-medium text-blue-700 dark:text-blue-300';

  const detailClass = waitingOnly
    ? 'text-xs text-amber-700 dark:text-amber-400 truncate'
    : 'text-xs text-blue-600 dark:text-blue-400 truncate';

  const sideClass = waitingOnly
    ? 'flex items-center gap-3 text-xs text-amber-700 dark:text-amber-400'
    : 'flex items-center gap-3 text-xs text-blue-600 dark:text-blue-400';

  return (
    <div
      className={containerClass}
      onClick={onOpenDetails}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && onOpenDetails()}
    >
      <div className="flex items-center gap-4">
        {waitingOnly ? (
          <Clock className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
        ) : (
          <Loader2 className="h-4 w-4 text-blue-600 dark:text-blue-400 animate-spin shrink-0" />
        )}
        <div className="flex-1 min-w-0">
          <p className={titleClass}>{headline}</p>
          {activeDocs.length > 0 && (
            <div className="mt-1 space-y-0.5">
              {activeDocs.slice(0, 2).map((doc) => (
                <p key={doc.id} className={detailClass}>
                  {doc.title || doc.file_name || 'Document'}:{' '}
                  {doc.stage_message || doc.current_stage || 'Processing...'}
                </p>
              ))}
            </div>
          )}
          {waitingDocs.length > 0 && (
            <div className="mt-1 space-y-0.5">
              {waitingDocs.slice(0, 2).map((doc) => (
                <p key={doc.id} className={detailClass}>
                  {doc.title || doc.file_name || 'Document'}:{' '}
                  {doc.stage_message ||
                    t(
                      'pipeline.waitingForSlot',
                      'Waiting for a processing slot',
                    )}
                </p>
              ))}
            </div>
          )}
        </div>
        <div className={sideClass}>
          {totalWaiting > 0 && activeCount > 0 && (
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {totalWaiting} waiting
            </span>
          )}
          {pipelineStatus.completed_tasks > 0 && (
            <span className="flex items-center gap-1">
              <CheckCircle className="h-3 w-3 text-green-600" />
              {pipelineStatus.completed_tasks} done
            </span>
          )}
          <span className={waitingOnly ? 'text-amber-600' : 'text-blue-500'}>
            Click for details →
          </span>
        </div>
      </div>
    </div>
  );
}

export default ProcessingStatusSummary;
