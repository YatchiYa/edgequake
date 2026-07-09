/**
 * Presentational ingestion activity banner (DRY surface for documents page).
 *
 * @implements SPEC-045 — honest ingestion UX (working / queued / stuck)
 */
'use client';

import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
  buildIngestionAlertHeadline,
  ingestionAlertContainerClass,
  ingestionAlertDetailClass,
  ingestionAlertTitleClass,
} from '@/lib/pipeline/ingestion-alert-presenter';
import { resolveBannerStageProgress } from '@/lib/pipeline/graph-merge-progress';
import { translateIngestionDetail } from '@/lib/pipeline/ingestion-user-messages';
import {
  resolvePipelineUiState,
  summarizePipelineDocuments,
} from '@/lib/pipeline/pipeline-document-state';
import type { Document, PipelineStatus } from '@/types';
import { cn } from '@/lib/utils';
import { AlertTriangle, Clock, Loader2 } from 'lucide-react';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

export interface IngestionAlertBannerProps {
  pipelineStatus: PipelineStatus;
  documents: Document[];
  onOpenDetails: () => void;
  onReprocessStuck?: (documents: Document[]) => void;
  isReprocessing?: boolean;
}

export function IngestionAlertBanner({
  pipelineStatus,
  documents,
  onOpenDetails,
  onReprocessStuck,
  isReprocessing = false,
}: IngestionAlertBannerProps) {
  const { t } = useTranslation();

  const summary = useMemo(
    () => summarizePipelineDocuments(documents),
    [documents],
  );

  const pipelineUi = useMemo(
    () => resolvePipelineUiState(documents, pipelineStatus),
    [documents, pipelineStatus],
  );

  const stageProgress = useMemo(
    () =>
      pipelineUi.isActivelyProcessing
        ? resolveBannerStageProgress(summary.activeDocs)
        : undefined,
    [pipelineUi.isActivelyProcessing, summary.activeDocs],
  );

  if (!pipelineUi.showPipelineIndicator) {
    return null;
  }

  const headline = buildIngestionAlertHeadline(pipelineUi, t);
  const messageContext =
    pipelineUi.alertMode === 'stuck'
      ? 'stuck'
      : pipelineUi.alertMode === 'queued'
        ? 'queued'
        : 'active';

  const detailDocs =
    pipelineUi.alertMode === 'stuck'
      ? pipelineUi.stuckDocs
      : pipelineUi.alertMode === 'queued'
        ? summary.waitingDocs
        : [...summary.activeDocs, ...summary.waitingDocs];

  const hintText =
    pipelineUi.alertMode === 'stuck'
      ? t(
          'pipeline.stuckHint',
          'No worker is processing these documents. Reprocess to create a new task.',
        )
      : pipelineUi.alertMode === 'queued'
        ? t(
            'pipeline.waitingHint',
            'Workers are idle but documents are queued. Processing will resume automatically.',
          )
        : null;

  return (
    <div
      data-testid="ingestion-status-banner"
      data-ingestion-mode={headline.dataTestId}
      className={cn(
        'flex flex-col gap-2 px-3 py-2 border rounded-lg cursor-pointer transition-colors overflow-hidden',
        ingestionAlertContainerClass(headline.variant),
      )}
      onClick={onOpenDetails}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && onOpenDetails()}
    >
      <div className="flex items-start gap-3 min-w-0">
        {headline.showAlert ? (
          <AlertTriangle className="h-4 w-4 text-rose-600 dark:text-rose-400 shrink-0 mt-0.5" />
        ) : headline.showClock ? (
          <Clock className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0 mt-0.5" />
        ) : (
          <Loader2 className="h-4 w-4 text-blue-600 dark:text-blue-400 animate-spin shrink-0 mt-0.5" />
        )}
        <div className="flex-1 min-w-0 space-y-1">
          <div className="flex items-start justify-between gap-2 min-w-0">
            <p
              data-testid={headline.dataTestId}
              className={cn(
                'text-sm font-medium min-w-0',
                ingestionAlertTitleClass(headline.variant),
              )}
            >
              {headline.text}
            </p>
            <div className="flex items-center gap-2 shrink-0">
              {pipelineUi.isStuck && onReprocessStuck && (
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  data-testid="ingestion-banner-reprocess"
                  className="h-7 text-xs border-rose-300 text-rose-700 hover:bg-rose-100 dark:border-rose-700 dark:text-rose-300"
                  disabled={isReprocessing}
                  onClick={(event) => {
                    event.stopPropagation();
                    onReprocessStuck(pipelineUi.stuckDocs);
                  }}
                >
                  {isReprocessing ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    t('pipeline.reprocessStuck', 'Reprocess')
                  )}
                </Button>
              )}
              <span
                className={cn(
                  'text-xs whitespace-nowrap',
                  ingestionAlertDetailClass(headline.variant),
                )}
              >
                {t('pipeline.clickForDetails', 'Details →')}
              </span>
            </div>
          </div>

          {hintText && (
            <p className={cn('text-xs', ingestionAlertDetailClass(headline.variant))}>
              {hintText}
            </p>
          )}

          {detailDocs.length > 0 && (
            <div className="space-y-0.5 min-w-0">
              {detailDocs.slice(0, 2).map((doc) => (
                <p
                  key={doc.id}
                  className={cn(
                    'text-xs line-clamp-2 break-words',
                    ingestionAlertDetailClass(headline.variant),
                  )}
                >
                  {translateIngestionDetail(doc, messageContext)}
                </p>
              ))}
            </div>
          )}

          {typeof stageProgress === 'number' && stageProgress > 0 && (
            <div
              className="space-y-1 pt-0.5"
              data-testid="ingestion-banner-progress"
            >
              <div className="flex items-center justify-between gap-2 text-[11px]">
                <span className={ingestionAlertDetailClass(headline.variant)}>
                  {t('pipeline.graphMergeProgress', 'Graph save progress')}
                </span>
                <span
                  className={cn(
                    'tabular-nums font-medium',
                    ingestionAlertDetailClass(headline.variant),
                  )}
                >
                  {Math.round(stageProgress * 100)}%
                </span>
              </div>
              <Progress
                value={stageProgress * 100}
                className="h-1.5 bg-black/5 dark:bg-white/10"
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default IngestionAlertBanner;
