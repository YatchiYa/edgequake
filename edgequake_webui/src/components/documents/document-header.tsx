/**
 * @module DocumentHeader
 * @description Header section for document management page.
 * Extracted from DocumentManager for SRP compliance (OODA-23).
 *
 * SPEC-099 LAW-099-5: Clear All is demoted to overflow (not peer of Refresh).
 *
 * @implements FEAT0001 - Document management header
 */
'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
<<<<<<< HEAD
import { AlertTriangle, Clock, Loader2, RefreshCw } from 'lucide-react';
=======
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { AlertTriangle, Clock, Loader2, MoreHorizontal, RefreshCw, Trash2 } from 'lucide-react';
import { useState, type ReactNode } from 'react';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
  /** Optional overflow honesty label (e.g. "17 of 240") */
  countLabel?: string;
  /** Number of failed documents */
  failedCount: number;
  /** Show pipeline status shortcut in header */
  showPipelineIndicator: boolean;
<<<<<<< HEAD
=======
  /**
   * SPEC-099 CLS: keep Working pill geometry reserved while feedback zone is
   * expected (skeleton) so the header does not grow when live work paints.
   */
  reservePipelineSlot?: boolean;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  /** Ingestion alert mode for header shortcut styling */
  pipelineAlertMode?: 'working' | 'queued' | 'stuck' | 'mixed';
  /** Active working document count (for Working · N pill) */
  activeDocCount?: number;
<<<<<<< HEAD
=======
  /** Queued / waiting document count (Working · W · Queued · Q) */
  waitingDocCount?: number;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
<<<<<<< HEAD
=======
  /** Optional columns control slot (e.g. show Cost) */
  columnsMenu?: ReactNode;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

/**
 * Document management page header with status and actions.
 */
export function DocumentHeader({
  totalCount,
  countLabel,
  failedCount,
  showPipelineIndicator,
<<<<<<< HEAD
  pipelineAlertMode,
  activeDocCount,
=======
  reservePipelineSlot = false,
  pipelineAlertMode,
  activeDocCount,
  waitingDocCount,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  pipelineWaitingOnly,
  pipelineDialogOpen,
  onPipelineDialogChange,
  onRefresh,
  tenantId,
  workspaceId,
  documents,
<<<<<<< HEAD
}: DocumentHeaderProps) {
  const { t } = useTranslation();
  const alertMode = pipelineAlertMode ?? (pipelineWaitingOnly ? 'queued' : 'working');
=======
  columnsMenu,
}: DocumentHeaderProps) {
  const { t } = useTranslation();
  const [clearAllOpen, setClearAllOpen] = useState(false);
  const alertMode = pipelineAlertMode ?? (pipelineWaitingOnly ? 'queued' : 'working');
  const working = activeDocCount ?? 0;
  const queued = waitingDocCount ?? 0;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  const pipelineButtonClass =
    alertMode === 'stuck'
      ? 'gap-1 text-rose-600 border-rose-300 hover:bg-rose-50 dark:hover:bg-rose-950/40'
<<<<<<< HEAD
      : alertMode === 'queued'
=======
      : alertMode === 'queued' || (working === 0 && queued > 0)
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        ? 'gap-1 text-amber-700 border-amber-300 hover:bg-amber-50 dark:text-amber-400 dark:hover:bg-amber-950/40'
        : 'gap-1 text-sky-700 border-sky-300 hover:bg-sky-50 dark:text-sky-300 dark:border-sky-800 dark:hover:bg-sky-950/40';

  const pipelineButtonLabel =
    alertMode === 'stuck'
      ? t('pipeline.stuckBadge', 'Needs attention')
<<<<<<< HEAD
      : alertMode === 'queued'
        ? t('pipeline.queuedBadge', 'Queued')
        : activeDocCount && activeDocCount > 0
          ? t('pipeline.workingCount', 'Working · {{count}}', {
              count: activeDocCount,
            })
          : t('pipeline.busy', 'Working');
=======
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

  const badgeText = countLabel ?? (totalCount > 0 ? String(totalCount) : null);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  return (
    <>
      <ConnectionBanner />
      
      <header className="flex items-center justify-between gap-3 flex-wrap">
        <div className="space-y-0.5">
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-semibold tracking-tight">{t('documents.title')}</h1>
            {badgeText && (
              <Badge
                variant="secondary"
                className="min-w-[2.25rem] justify-center text-xs font-normal tabular-nums"
                data-testid="spec099-documents-count"
              >
                {badgeText}
              </Badge>
            )}
            <ConnectionStatus compact={true} />
          </div>
          <p className="text-sm text-muted-foreground">
            {t('documents.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
<<<<<<< HEAD
          {/* Pipeline Status */}
          {showPipelineIndicator && (
=======
          {(showPipelineIndicator || reservePipelineSlot) && (
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPipelineDialogChange(true)}
<<<<<<< HEAD
              className={pipelineButtonClass}
=======
              className={`${pipelineButtonClass} min-w-[7.5rem] ${
                showPipelineIndicator
                  ? ''
                  : 'invisible pointer-events-none'
              }`}
              tabIndex={showPipelineIndicator ? 0 : -1}
              aria-hidden={!showPipelineIndicator}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
          
          <ReprocessFailedButton
            failedCount={failedCount}
            onReprocessStarted={() => {
              onPipelineDialogChange(true);
            }}
          />
        
          <Button
            variant="outline"
            size="sm"
            onClick={onRefresh}
            data-testid="documents-refresh-button"
          >
            <RefreshCw className="h-4 w-4 mr-1" />
            {t('documents.refresh')}
          </Button>

          {/* SPEC-099: Clear All demoted to overflow — not peer of Refresh */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="sm"
                aria-label={t('documents.moreActions', 'More actions')}
                data-testid="spec099-documents-overflow"
              >
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56">
              {columnsMenu}
              {columnsMenu ? <DropdownMenuSeparator /> : null}
              <DropdownMenuLabel className="text-xs text-muted-foreground font-normal">
                {t('documents.dangerZone', 'Danger zone')}
              </DropdownMenuLabel>
              <button
                type="button"
                className="relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none text-destructive focus:bg-destructive/10"
                data-testid="spec099-clear-all-menu-item"
                aria-label={t('documents.clearAll.button', 'Clear All')}
                onClick={() => setClearAllOpen(true)}
              >
                <Trash2 className="h-3.5 w-3.5" />
                {t('documents.clearAll.button', 'Clear All')}
              </button>
            </DropdownMenuContent>
          </DropdownMenu>
          <ClearDocumentsDialog
            documentCount={totalCount}
            onCleared={onRefresh}
            showTrigger={false}
            open={clearAllOpen}
            onOpenChange={setClearAllOpen}
          />
        </div>
      </header>
    </>
  );
}

export default DocumentHeader;
