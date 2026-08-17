'use client';

import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Progress } from '@/components/ui/progress';
import { invalidateKnowledgeGraph } from '@/lib/cache-manager';
import { deleteAllDocuments } from '@/lib/api/edgequake';
import { useBulkDeletionProgress } from '@/hooks/use-bulk-deletion-progress';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { AlertTriangle, CheckCircle2, Loader2, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

interface ClearDocumentsDialogProps {
  /**
   * Total number of documents that will be deleted.
   * Used for display purposes.
   */
  documentCount: number;
  /**
   * Callback when documents are cleared successfully.
   * @param deletedCount Number of documents that were deleted
   */
  onCleared?: (deletedCount: number) => void;
  /**
   * Whether to show the button (false to use only as a controlled dialog)
   */
  showTrigger?: boolean;
  /**
   * Controlled open state
   */
  open?: boolean;
  /**
   * Callback when open state changes
   */
  onOpenChange?: (open: boolean) => void;
  /**
   * SPEC-099: "menu" renders a destructive menu item (overflow); default is ghost button.
   */
  triggerVariant?: "button" | "menu";
}

const CONFIRMATION_TEXT = 'DELETE ALL';

/**
 * Dialog component for clearing all documents from the system.
 * Requires typing "DELETE ALL" to confirm the destructive action.
 * Connects to DELETE /api/v1/documents endpoint.
 */
export function ClearDocumentsDialog({
  documentCount,
  onCleared,
  showTrigger = true,
  open: controlledOpen,
  onOpenChange: controlledOnOpenChange,
  triggerVariant = "button",
}: ClearDocumentsDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [internalOpen, setInternalOpen] = useState(false);
  const [confirmation, setConfirmation] = useState('');

  // Use controlled or internal state
  const isOpen = controlledOpen !== undefined ? controlledOpen : internalOpen;
  const setOpen = controlledOnOpenChange || setInternalOpen;

  const isConfirmed = confirmation === CONFIRMATION_TEXT;

  // ISSUE-309: HTTP 202 admits wipe; final counts arrive on BulkDeletionCompleted
  // (correlated by wipe_track_id) or task-status poll fallback.
  const [awaitingWipeComplete, setAwaitingWipeComplete] = useState(false);
  const [plannedDeleteCount, setPlannedDeleteCount] = useState(0);
  const [wipeTrackId, setWipeTrackId] = useState<string | null>(null);

  const clearMutation = useMutation({
    mutationFn: deleteAllDocuments,
    onSuccess: (data) => {
      if (data.accepted) {
        setPlannedDeleteCount(data.deleted_count);
        setWipeTrackId(data.wipe_track_id ?? null);
        setAwaitingWipeComplete(true);
        toast.message(
          t('documents.clearAll.started', 'Deleting documents…'),
          {
            description: t(
              'documents.clearAll.startedDesc',
              'Bulk wipe accepted for {{count}} document(s). Waiting for completion…',
              { count: data.deleted_count },
            ),
          },
        );
        return;
      }
      toast.success(
        t('documents.clearAll.success', 'Documents cleared'),
        {
          description: t('documents.clearAll.successDesc', 'Successfully deleted {{count}} document(s) and their associated data.', { count: data.deleted_count }),
          duration: 5000,
        }
      );
      queryClient.invalidateQueries({ queryKey: ['documents'] });
      invalidateKnowledgeGraph(queryClient);
      setConfirmation('');
      setOpen(false);
      onCleared?.(data.deleted_count);
    },
    onError: (error) => {
      setAwaitingWipeComplete(false);
      toast.error(
        t('documents.clearAll.failed', 'Clear failed'),
        {
          description: error instanceof Error ? error.message : t('common.unknownError', 'An error occurred'),
          action: {
            label: t('common.retry', 'Retry'),
            onClick: () => clearMutation.mutate(),
          },
        }
      );
    },
  });

  // SPEC-050: Track bulk deletion progress via WebSocket + wipe_track_id poll.
  const bulkProgress = useBulkDeletionProgress(
    isOpen && (clearMutation.isPending || awaitingWipeComplete),
    wipeTrackId,
  );

  useEffect(() => {
    if (!awaitingWipeComplete || !bulkProgress.isComplete) return;
    if (bulkProgress.isFailed) {
      toast.error(t('documents.clearAll.failed', 'Clear failed'), {
        description:
          bulkProgress.errorMessage ||
          t('common.unknownError', 'An error occurred'),
      });
      setAwaitingWipeComplete(false);
      setWipeTrackId(null);
      return;
    }
    const count = bulkProgress.completed || plannedDeleteCount;
    toast.success(
      t('documents.clearAll.success', 'Documents cleared'),
      {
        description: t(
          'documents.clearAll.successDesc',
          'Successfully deleted {{count}} document(s) and their associated data.',
          { count },
        ),
        duration: 5000,
      },
    );
    queryClient.invalidateQueries({ queryKey: ['documents'] });
    invalidateKnowledgeGraph(queryClient);
    setConfirmation('');
    setAwaitingWipeComplete(false);
    setWipeTrackId(null);
    setOpen(false);
    onCleared?.(count);
  }, [
    awaitingWipeComplete,
    bulkProgress.isComplete,
    bulkProgress.isFailed,
    bulkProgress.errorMessage,
    bulkProgress.completed,
    plannedDeleteCount,
    queryClient,
    t,
    setOpen,
    onCleared,
  ]);

  const handleClear = () => {
    if (!isConfirmed) return;
    clearMutation.mutate();
  };

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      // Reset confirmation when closing
      setConfirmation('');
    }
    setOpen(newOpen);
  };

  // Don't show if no documents
  if (documentCount === 0) {
    return null;
  }

<<<<<<< HEAD
  const triggerButton = (
    <Button
      variant="ghost"
      size="sm"
      className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 gap-1.5"
    >
      <Trash2 className="h-3.5 w-3.5" />
      {t('documents.clearAll.button', 'Clear All')}
    </Button>
  );
=======
  const triggerButton =
    triggerVariant === "menu" ? (
      <button
        type="button"
        className="relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none text-destructive focus:bg-destructive/10 data-[disabled]:pointer-events-none data-[disabled]:opacity-50"
        data-testid="spec099-clear-all-menu-item"
        aria-label={t('documents.clearAll.button', 'Clear All')}
      >
        <Trash2 className="h-3.5 w-3.5" />
        {t('documents.clearAll.button', 'Clear All')}
      </button>
    ) : (
      <Button
        variant="ghost"
        size="sm"
        className="text-muted-foreground hover:text-destructive hover:bg-destructive/10 gap-1.5"
        data-testid="spec099-clear-all-button"
      >
        <Trash2 className="h-3.5 w-3.5" />
        {t('documents.clearAll.button', 'Clear All')}
      </Button>
    );
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  return (
    <AlertDialog open={isOpen} onOpenChange={handleOpenChange}>
      {showTrigger && (
        <AlertDialogTrigger asChild>
          {triggerButton}
        </AlertDialogTrigger>
      )}
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2 text-destructive">
            <AlertTriangle className="h-5 w-5" />
            {t('documents.clearAll.title', 'Delete All Documents')}
          </AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div className="text-muted-foreground text-sm space-y-3">
              <p>
                {t('documents.clearAll.warning', 'This action cannot be undone. This will permanently delete:')}
              </p>
              <ul className="list-disc list-inside space-y-1 text-sm">
                <li>{t('documents.clearAll.item1', '{{count}} document(s)', { count: documentCount })}</li>
                <li>{t('documents.clearAll.item2', 'All extracted entities and relationships')}</li>
                <li>{t('documents.clearAll.item3', 'All document chunks and embeddings')}</li>
              </ul>
              <div className="pt-2">
                <Label htmlFor="confirmation" className="text-sm font-medium">
                  {t('documents.clearAll.typeToConfirm', 'Type {{text}} to confirm:', { text: CONFIRMATION_TEXT })}
                </Label>
                <Input
                  id="confirmation"
                  value={confirmation}
                  onChange={(e) => setConfirmation(e.target.value)}
                  placeholder={CONFIRMATION_TEXT}
                  className="mt-2"
                  disabled={clearMutation.isPending || awaitingWipeComplete}
                  autoComplete="off"
                />
              </div>
            </div>
          </AlertDialogDescription>
        </AlertDialogHeader>

        {/* SPEC-050: Real-time bulk deletion progress (HTTP pending or async wipe) */}
        {(clearMutation.isPending || awaitingWipeComplete) && (
          <div className="space-y-2 py-1" data-testid="bulk-deletion-progress">
            <Progress
              value={bulkProgress.total > 0 ? (bulkProgress.completed / bulkProgress.total) * 100 : undefined}
              className="h-2"
            />
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>
                {bulkProgress.total > 0
                  ? t('documents.clearAll.progressCount', '{{done}} / {{total}} deleted', {
                      done: bulkProgress.completed,
                      total: bulkProgress.total,
                    })
                  : t('documents.clearAll.progressPreparing', 'Preparing…')}
              </span>
              {bulkProgress.isComplete && (
                <span className="flex items-center gap-1 text-emerald-600 dark:text-emerald-400">
                  <CheckCircle2 className="h-3 w-3" />
                  {t('documents.clearAll.progressDone', 'Done')}
                </span>
              )}
            </div>
          </div>
        )}
        <AlertDialogFooter>
          <AlertDialogCancel disabled={clearMutation.isPending}>
            {t('common.cancel', 'Cancel')}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={handleClear}
            disabled={!isConfirmed || clearMutation.isPending}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {clearMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                {t('documents.clearAll.deleting', 'Deleting...')}
              </>
            ) : (
              t('documents.clearAll.confirmButton', 'Delete All Documents')
            )}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
