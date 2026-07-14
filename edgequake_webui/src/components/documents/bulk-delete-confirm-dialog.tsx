/**
 * @module BulkDeleteConfirmDialog
 * @description Confirmation dialog for bulk document deletion from the toolbar.
 *
 * WHY: The toolbar "Delete" button (BatchActionsBar) previously called
 * deleteDocument() directly with no confirmation or impact preview, creating
 * a very different — and worse — UX than single-document deletion.
 *
 * This dialog provides parity: shows which documents will be deleted,
 * warns about knowledge graph impact, and requires explicit confirmation.
 *
 * @implements SPEC-050: Bulk toolbar delete parity (Gap 2 fix).
 * @implements AC-050-01: Impact preview before delete (bulk variant).
 * @enforces BR0303: Document deletion cascades to related entities.
 */
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
} from '@/components/ui/alert-dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import type { Document } from '@/types';
import { AlertTriangle, FileText, Loader2, Trash2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface BulkDeleteConfirmDialogProps {
  /** Whether the dialog is open. */
  open: boolean;
  /** Called when the dialog should close. */
  onOpenChange: (open: boolean) => void;
  /** The documents selected for deletion. */
  documents: Document[];
  /** Called when the user confirms the bulk delete. */
  onConfirm: () => void;
  /** True while deletion is in progress. */
  isDeleting?: boolean;
}

/**
 * Confirmation dialog for deleting a set of selected documents.
 *
 * Shows up to 5 document names, then "and N more…" for larger sets.
 * Warns about knowledge graph cascade impact.
 */
export function BulkDeleteConfirmDialog({
  open,
  onOpenChange,
  documents,
  onConfirm,
  isDeleting = false,
}: BulkDeleteConfirmDialogProps) {
  const { t } = useTranslation();

  const count = documents.length;
  const PREVIEW_LIMIT = 5;
  const preview = documents.slice(0, PREVIEW_LIMIT);
  const overflow = count - PREVIEW_LIMIT;

  const handleConfirm = (e: React.MouseEvent) => {
    e.preventDefault();
    onConfirm();
    onOpenChange(false);
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent data-testid="bulk-delete-confirm-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2 text-destructive">
            <Trash2 className="h-5 w-5" />
            {t('documents.bulkDeleteDialog.title', 'Delete {{count}} document(s)?', { count })}
          </AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">
                {t(
                  'documents.bulkDeleteDialog.description',
                  'This action is permanent and cannot be undone. All extracted entities and relationships will be removed from the knowledge graph.',
                )}
              </p>

              {/* Document list preview */}
              <div className="rounded-md border bg-muted/30 p-3 space-y-1">
                <div className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
                  {t('documents.bulkDeleteDialog.documentsLabel', 'Documents to delete')}
                </div>
                <ScrollArea className={count > 5 ? 'h-32 min-w-0 w-full' : 'min-w-0 w-full'}>
                  {preview.map((doc) => (
                    <div
                      key={doc.id}
                      className="grid grid-cols-[auto_minmax(0,1fr)] items-center gap-2 py-1 text-sm"
                      data-testid={`bulk-delete-item-${doc.id}`}
                    >
                      <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      <span className="truncate">
                        {doc.file_name || doc.title || doc.id.slice(0, 12)}
                      </span>
                    </div>
                  ))}
                  {overflow > 0 && (
                    <div className="text-xs text-muted-foreground pt-1">
                      {t('documents.bulkDeleteDialog.andMore', '…and {{n}} more', { n: overflow })}
                    </div>
                  )}
                </ScrollArea>
              </div>

              {/* Graph impact warning */}
              <div className="flex items-start gap-2 rounded-md border border-rose-200 bg-rose-50/60 dark:border-rose-900 dark:bg-rose-950/20 px-3 py-2 text-xs text-rose-700 dark:text-rose-400">
                <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                <span>
                  {t(
                    'documents.bulkDeleteDialog.graphWarning',
                    'Entities and relationships unique to these documents will be removed from your knowledge graph.',
                  )}
                </span>
              </div>
            </div>
          </AlertDialogDescription>
        </AlertDialogHeader>

        <AlertDialogFooter>
          <AlertDialogCancel
            disabled={isDeleting}
            data-testid="bulk-delete-confirm-cancel"
          >
            {t('common.cancel', 'Cancel')}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={handleConfirm}
            disabled={isDeleting || count === 0}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            data-testid="bulk-delete-confirm-submit"
          >
            {isDeleting ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                {t('documents.bulkDeleteDialog.deleting', 'Deleting…')}
              </>
            ) : (
              <>
                <Trash2 className="h-4 w-4 mr-2" />
                {t('documents.bulkDeleteDialog.confirm', 'Delete {{count}} document(s)', { count })}
              </>
            )}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
