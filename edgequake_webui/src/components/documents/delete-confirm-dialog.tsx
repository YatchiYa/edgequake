/**
 * @module DeleteConfirmDialog
 * @description Confirmation dialog for document deletion with pre-delete impact preview.
 *
 * WHY: Users must understand the cascade impact of deleting a document before
 * confirming. This dialog:
 *   1. Fetches impact data as soon as it opens (before the user can confirm).
 *   2. Displays entity/relationship/chunk counts in DeletionImpactCard.
 *   3. Requires explicit user confirmation before calling the delete mutation.
 *
 * This replaces the previous pattern of deleting immediately from DocumentActionsMenu
 * with no confirmation or impact preview.
 *
 * @implements SPEC-050: Impact preview before delete (AC-050-01, AC-050-02).
 * @implements BR0303: Document deletion cascades to related entities.
 * @enforces First Principle: A user must know what will happen before a destructive action.
 */
'use client';

import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { useDeletionImpact } from '@/hooks/use-deletion-impact';
import type { Document } from '@/types';
import { AlertTriangle, Loader2, Trash2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { DeletionImpactCard } from './deletion-impact-card';

interface DeleteConfirmDialogProps {
  /** Controls whether the dialog is open. */
  open: boolean;
  /** Called when the dialog should close (cancel or after action). */
  onOpenChange: (open: boolean) => void;
  /**
   * The document to delete.
   * Pass null to show a loading/empty state.
   */
  document: Pick<Document, 'id' | 'title' | 'file_name'> | null;
  /**
   * Called when the user confirms the delete.
   * The parent is responsible for calling the actual delete mutation.
   */
  onConfirm: (documentId: string) => void;
  /**
   * True while the parent's delete mutation is in progress.
   * Disables the confirm button and shows a spinner.
   */
  isDeleting?: boolean;
}

/**
 * Confirmation dialog that shows deletion impact before allowing the user to confirm.
 *
 * The dialog fetches impact data as soon as `open` becomes true and `document.id`
 * is set.  If the impact fetch fails, a warning banner is shown but the user
 * can still proceed — impact analysis is advisory, not a blocker.
 */
export function DeleteConfirmDialog({
  open,
  onOpenChange,
  document: target,
  onConfirm,
  isDeleting = false,
}: DeleteConfirmDialogProps) {
  const { t } = useTranslation();

  // Fetch impact only when the dialog is open and we have a document.
  // WHY: useQuery with `enabled: !!target?.id && open` avoids fetching for
  // closed dialogs and reuses the cached result for 30s (staleTime in hook).
  const { impact, isLoading: isImpactLoading, error: impactError } =
    useDeletionImpact(open && target?.id ? target.id : null);

  const docName =
    target?.file_name || target?.title || target?.id?.slice(0, 8) || '';

  const handleConfirm = () => {
    if (!target?.id) return;
    onConfirm(target.id);
    onOpenChange(false);
  };

  const handleCancel = () => {
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-md"
        data-testid="delete-confirm-dialog"
      >
        <DialogHeader>
          <DialogTitle
            className="flex items-center gap-2"
            data-testid="delete-confirm-dialog-title"
          >
            <Trash2 className="h-5 w-5 text-rose-500" />
            {t('documents.deleteDialog.title', 'Delete "{{name}}"?', {
              name: docName,
            })}
          </DialogTitle>
          <DialogDescription>
            {t(
              'documents.deleteDialog.description',
              'This action is permanent and cannot be undone. All extracted data for this document will be removed.',
            )}
          </DialogDescription>
        </DialogHeader>

        {/* Impact analysis — loaded when dialog opens */}
        <DeletionImpactCard
          impact={impact}
          isLoading={isImpactLoading}
          error={impactError}
        />

        {/* Warning banner for documents that may affect the shared knowledge graph */}
        {impact &&
          (impact.entities_to_remove > 0 ||
            impact.relationships_to_remove > 0) && (
            <div className="flex items-start gap-2 rounded-md border border-rose-200 bg-rose-50/60 dark:border-rose-900 dark:bg-rose-950/20 px-3 py-2 text-xs text-rose-700 dark:text-rose-400">
              <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
              <span>
                {t(
                  'documents.deleteDialog.graphWarning',
                  'Deleting this document will also remove entities and relationships from your knowledge graph.',
                )}
              </span>
            </div>
          )}

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={isDeleting}
            data-testid="delete-confirm-cancel"
          >
            {t('common.cancel', 'Cancel')}
          </Button>
          <Button
            variant="destructive"
            onClick={handleConfirm}
            disabled={isDeleting || !target?.id}
            data-testid="delete-confirm-submit"
          >
            {isDeleting ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                {t('documents.deleteDialog.deleting', 'Deleting…')}
              </>
            ) : (
              <>
                <Trash2 className="h-4 w-4 mr-2" />
                {t('documents.deleteDialog.confirm', 'Delete permanently')}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
