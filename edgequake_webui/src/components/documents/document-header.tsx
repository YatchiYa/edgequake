/**
 * @module DocumentHeader
 * @description Header section for document management page.
 * Extracted from DocumentManager for SRP compliance (OODA-23).
 * 
 * WHY: Header JSX was inline in DocumentManager causing bloat.
 * This component displays:
 * - Page title with document count badge
 * - WebSocket connection status
 * - Pipeline status button and dialog
 * - Reprocess failed button
 * - Refresh and clear buttons
 * 
 * @implements FEAT0001 - Document management header
 */
'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { AlertTriangle, Clock, Loader2, RefreshCw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { Document } from '@/types';
import { ClearDocumentsDialog } from './clear-documents-dialog';
import { ConnectionBanner } from './connection-banner';
import { ConnectionStatus } from './connection-status';
import { PipelineStatusDialog } from './pipeline-status-dialog';
import { ReprocessFailedButton } from './reprocess-failed-button';

/**
 * Props for DocumentHeader component.
 */
export interface DocumentHeaderProps {
  /** Total document count */
  totalCount: number;
  /** Number of failed documents */
  failedCount: number;
  /** Show pipeline status shortcut in header */
  showPipelineIndicator: boolean;
  /** Ingestion alert mode for header shortcut styling */
  pipelineAlertMode?: 'working' | 'queued' | 'stuck' | 'mixed';
  /** Active working document count (for Working · N pill) */
  activeDocCount?: number;
  /** Queued / waiting document count (Working · W · Queued · Q) */
  waitingDocCount?: number;
  /** @deprecated Use pipelineAlertMode */
  pipelineWaitingOnly: boolean;
  /** Whether pipeline dialog is open */
  pipelineDialogOpen: boolean;
  /** Handler to set pipeline dialog state */
  onPipelineDialogChange: (open: boolean) => void;
  /** Handler to refresh documents */
  onRefresh: () => void;
  /** Tenant ID for pipeline dialog */
  tenantId?: string;
  /** Workspace ID for pipeline dialog */
  workspaceId?: string;
  /** Documents for pipeline waiting-state details */
  documents?: Document[];
}

/**
 * Document management page header with status and actions.
 */
export function DocumentHeader({
  totalCount,
  failedCount,
  showPipelineIndicator,
  pipelineAlertMode,
  activeDocCount,
  waitingDocCount,
  pipelineWaitingOnly,
  pipelineDialogOpen,
  onPipelineDialogChange,
  onRefresh,
  tenantId,
  workspaceId,
  documents,
}: DocumentHeaderProps) {
  const { t } = useTranslation();
  const alertMode = pipelineAlertMode ?? (pipelineWaitingOnly ? 'queued' : 'working');
  const working = activeDocCount ?? 0;
  const queued = waitingDocCount ?? 0;

  const pipelineButtonClass =
    alertMode === 'stuck'
      ? 'gap-1 text-rose-600 border-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40'
      : alertMode === 'queued' || (working === 0 && queued > 0)
        ? 'gap-1 text-amber-700 border-amber-300 hover:bg-amber-50 dark:text-amber-400 dark:hover:bg-amber-950/40'
        : 'gap-1 text-sky-700 border-sky-300 hover:bg-sky-50 dark:text-sky-300 dark:border-sky-800 dark:hover:bg-sky-950/40';

  // IS-AC-07 / LAW-IS3: Working count appears once; include Queued when both matter.
  const pipelineButtonLabel =
    alertMode === 'stuck'
      ? t('pipeline.stuckBadge', 'Needs attention')
      : working > 0 && queued > 0
        ? t('pipeline.workingAndQueued', 'Working · {{working}} · Queued · {{queued}}', {
            working,
            queued,
          })
        : working > 0
          ? t('pipeline.workingCount', 'Working · {{count}}', {
              count: working,
            })
          : queued > 0
            ? t('pipeline.queuedCount', 'Queued · {{count}}', { count: queued })
            : t('pipeline.busy', 'Working');

  return (
    <>
      {/* OODA-02: Connection status banner when disconnected */}
      <ConnectionBanner />
      
      {/* Header - Compact */}
      <header className="flex items-center justify-between gap-3 flex-wrap">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-semibold tracking-tight">{t('documents.title')}</h1>
            {/* OODA-39: Document count badge */}
            {totalCount > 0 && (
              <Badge variant="secondary" className="text-xs font-normal">
                {totalCount}
              </Badge>
            )}
            {/* OODA-30: WebSocket connection status indicator */}
            <ConnectionStatus compact={true} />
          </div>
          <p className="text-sm text-muted-foreground">
            {t('documents.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {/* Pipeline Status */}
          {showPipelineIndicator && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPipelineDialogChange(true)}
              className={pipelineButtonClass}
              data-testid="pipeline-header-button"
            >
              {alertMode === 'stuck' ? (
                <AlertTriangle className="h-4 w-4" />
              ) : alertMode === 'queued' ? (
                <Clock className="h-4 w-4" />
              ) : (
                <Loader2 className="h-4 w-4 animate-spin" />
              )}
              {pipelineButtonLabel}
            </Button>
          )}
          <PipelineStatusDialog
            open={pipelineDialogOpen}
            onOpenChange={onPipelineDialogChange}
            tenantId={tenantId}
            workspaceId={workspaceId}
            documents={documents}
          />
          
          {/* Reprocess Failed Button (GAP-UI-002) */}
          <ReprocessFailedButton
            failedCount={failedCount}
            onReprocessStarted={() => {
              onPipelineDialogChange(true);
            }}
          />
        
          <Button variant="outline" size="sm" onClick={onRefresh}>
            <RefreshCw className="h-4 w-4 mr-1" />
            {t('documents.refresh')}
          </Button>
          
          {/* Clear Documents Dialog (GAP-UI-009) */}
          <ClearDocumentsDialog
            documentCount={totalCount}
            onCleared={onRefresh}
          />
        </div>
      </header>
    </>
  );
}

export default DocumentHeader;
