/**
 * @module DocumentManager
 * @description Document ingestion and management interface.
 * Supports file upload, progress tracking, status monitoring, and batch operations.
 * 
 * @implements UC0001 - User uploads documents for ingestion
 * @implements UC0007 - User monitors document processing progress
 * @implements UC0008 - User reprocesses failed documents
 * @implements UC0009 - User deletes documents from knowledge graph
 * @implements FEAT0001 - Document ingestion with entity extraction
 * @implements FEAT0003 - Batch document processing
 * @implements FEAT0004 - Processing status tracking per document
 * @implements FEAT0602 - Real-time progress indicators
 * 
 * @enforces BR0302 - Failed documents can be reprocessed
 * @enforces BR0303 - Document deletion cascades to related entities
 * @enforces BR0305 - Cost tracking per document ingestion
 * 
 * @see {@link docs/use_cases.md} UC0001, UC0007-UC0009
 * @see {@link docs/features.md} FEAT0001, FEAT0003
 */
'use client';

import { useSelectedWorkspace, useTenantStore } from '@/stores/use-tenant-store';
import type { Document } from '@/types';

import { useRouter } from 'next/navigation';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { nextDocumentSortState } from '@/lib/documents/document-sort';
import { buildIngestionRunViews } from '@/lib/pipeline/ingestion-run-view';
import { resolvePipelineUiState } from '@/lib/pipeline/pipeline-document-state';

import { useBulkSelection } from '@/hooks/use-bulk-selection';
import { useDocumentDropzone } from '@/hooks/use-document-dropzone';
import { useDocumentFiltering } from '@/hooks/use-document-filtering';
import { useDocumentHandlers } from '@/hooks/use-document-handlers';
import { useDocumentKeyboard } from '@/hooks/use-document-keyboard';
import { useDocumentMutations } from '@/hooks/use-document-mutations';
import { useDocumentPreferences } from '@/hooks/use-document-preferences';
import { useDocumentQueries } from '@/hooks/use-document-queries';
import { useDocumentTitle } from '@/hooks/use-document-title';
import { useDocumentWebSocket } from '@/hooks/use-document-websocket';
import { useFileUpload } from '@/hooks/use-file-upload';
import { useReprocessTracking } from '@/hooks/use-reprocess-tracking';
import { useStuckDetection } from '@/hooks/use-stuck-detection';
import type { PdfParserResolutionContext } from '@/lib/pdf/large-pdf-admission';
import {
    filterLargePdfFiles,
    type LargePdfAdmissionPreview,
    type PdfParserChoice,
} from '@/lib/pdf/large-pdf-admission';
import { BulkDeleteConfirmDialog } from './bulk-delete-confirm-dialog';
import { BulkReprocessDialog, type BulkReprocessChoice } from './bulk-reprocess-dialog';
import { DeleteConfirmDialog } from './delete-confirm-dialog';
import { DocumentErrorAlert } from './document-error-alert';
import { DocumentHeader } from './document-header';
import { DocumentPreviewRightPanel } from './document-preview-right-panel';
import { DocumentTableSection } from './document-table-section';
import { DocumentToolbarSection } from './document-toolbar-section';
import { DuplicateUploadDialog } from './duplicate-upload-dialog';
import { IngestionProgressPanel } from './ingestion-progress-panel';
import { LargePdfAdmissionDialog } from './large-pdf-admission-dialog';
import { ReprocessDialog, type ReprocessChoice } from './reprocess-dialog';

export function DocumentManager() {
  const { t } = useTranslation();
  const router = useRouter();

  // Get tenant context for query key
  const { selectedTenantId, selectedWorkspaceId } = useTenantStore();
  const selectedWorkspace = useSelectedWorkspace();

  // Selected document for preview panel
  const [selectedDocument, setSelectedDocument] = useState<Document | null>(null);
  const [previewPanelOpen, setPreviewPanelOpen] = useState(false);

  // Reprocess choice dialog state.
  // WHY: Reprocessing a completed PDF must let the user choose between a full
  // PDF -> markdown re-conversion (slower, spends vision tokens) and a fast
  // entity-only re-extraction (reuses cached markdown). The dialog collects the
  // intent before calling reprocessMutation with the chosen mode.
  const [reprocessTarget, setReprocessTarget] = useState<Document | null>(null);

  // Bulk reprocess choice dialog state.
  // WHY: The toolbar Reprocess button acts on every selected document at once.
  // We show one choice dialog (full vs entities) whose mode applies to the
  // whole batch, instead of prompting per document.
  const [bulkReprocessOpen, setBulkReprocessOpen] = useState(false);

  // SPEC-050 GAP-FIX: Delete confirm dialog state.
  // WHY: Both single (preview panel) and bulk (toolbar) delete routes must
  // open a confirmation dialog before deleting. This single state drives both.
  const [deleteConfirmTarget, setDeleteConfirmTarget] = useState<Document | null>(null);
  const [bulkDeleteTargets, setBulkDeleteTargets] = useState<Document[]>([]);
  const [bulkDeleteDialogOpen, setBulkDeleteDialogOpen] = useState(false);

  // SPEC-002: Document viewer dialog state for PDF/Markdown side-by-side view
  const [viewerDialogOpen, setViewerDialogOpen] = useState(false);
  const [viewerPdfId, setViewerPdfId] = useState<string | null>(null);

  // Search state
  const [searchQuery, setSearchQuery] = useState('');
  const [pdfParserBackend, setPdfParserBackend] = useState<'default' | 'vision' | 'edgeparse'>('default');
  const [largePdfAdmissionOpen, setLargePdfAdmissionOpen] = useState(false);
  const [largePdfPreviews, setLargePdfPreviews] = useState<LargePdfAdmissionPreview[]>([]);
  const [pendingAdmissionFiles, setPendingAdmissionFiles] = useState<File[]>([]);

  const pdfParserResolutionContext = useMemo<PdfParserResolutionContext>(
    () => ({
      uploadChoice: pdfParserBackend,
      workspaceBackend: selectedWorkspace?.pdf_parser_backend,
    }),
    [pdfParserBackend, selectedWorkspace?.pdf_parser_backend],
  );

  // VS-03: No pagination state — virtual scrolling handles windowing client-side.
  // We fetch all documents at once (up to VIRTUAL_PAGE_SIZE) and let the
  // virtualizer render only visible rows. This eliminates pagination UI entirely.

  // OODA-17: Filter/sort preferences with localStorage persistence
  const {
    statusFilter, setStatusFilter,
    sortField, setSortField,
    sortDirection, setSortDirection,
  } = useDocumentPreferences();

  const handleColumnSort = useCallback(
    (field: typeof sortField) => {
      const next = nextDocumentSortState(sortField, sortDirection, field);
      setSortField(next.field);
      setSortDirection(next.direction);
    },
    [sortField, sortDirection, setSortField, setSortDirection],
  );

  // Pipeline status dialog state
  const [pipelineDialogOpen, setPipelineDialogOpen] = useState(false);

  // OODA-13: Upload state extracted to useFileUpload hook
  const {
    uploadingFiles,
    isUploading,
    handleFilesUpload,
    removeUploadingFile,
    handleUploadComplete,
    handleUploadFailed,
    pruneTerminalUploads,
    pendingDuplicates,
    resolvePendingDuplicates,
  } = useFileUpload({
    tenantId: selectedTenantId,
    workspaceId: selectedWorkspaceId,
    onUploadStart: () => setStatusFilter('all'),
    pdfParserBackend:
      pdfParserBackend === 'default' ? undefined : pdfParserBackend,
  });

  const handleFilesAccepted = useCallback(
    async (files: File[]) => {
      const largePreviews = await filterLargePdfFiles(files, pdfParserResolutionContext);
      if (largePreviews.length > 0) {
        setLargePdfPreviews(largePreviews);
        setPendingAdmissionFiles(files);
        setLargePdfAdmissionOpen(true);
        return;
      }
      await handleFilesUpload(files);
    },
    [handleFilesUpload, pdfParserResolutionContext],
  );

  const handleAdmissionConfirm = useCallback(
    async (parserChoice: PdfParserChoice, files: File[]) => {
      setLargePdfAdmissionOpen(false);
      setLargePdfPreviews([]);
      setPendingAdmissionFiles([]);
      const parserOverride =
        parserChoice === 'default'
          ? undefined
          : parserChoice;
      if (parserChoice !== 'default') {
        setPdfParserBackend(parserChoice);
      }
      await handleFilesUpload(files, {
        pdfParserBackend: parserOverride,
      });
    },
    [handleFilesUpload],
  );

  const handleAdmissionCancel = useCallback(() => {
    setLargePdfAdmissionOpen(false);
    setPendingAdmissionFiles([]);
    setLargePdfPreviews([]);
  }, []);

  // SPEC-050-REPROCESS: Track reprocess operations to show IngestionProgressPanel
  // — identical feedback to a fresh upload (stage list, cost, ETA, cancel).
  // WHY SRP: This hook owns only state; the rendering and the mutation callback
  // are wired below, keeping each concern in the right layer.
  const {
    reprocessEntries,
    addReprocessEntry,
    removeReprocessEntry,
    pruneTerminalReprocessEntries,
  } = useReprocessTracking();

  // OODA-14: Document mutations extracted to useDocumentMutations hook
  const {
    deleteMutation,
    reprocessMutation,
    cancelMutation,
  } = useDocumentMutations({
    onReprocessSuccess: () => setPipelineDialogOpen(true),
    // SPEC-050-REPROCESS: wire the new-track-id callback into the tracking state.
    onReprocessTriggered: addReprocessEntry,
  });

  // SPEC-050: Track which document IDs are currently being deleted so rows can
  // show "Deleting" visual state immediately on confirm (before query invalidation).
  const [deletingDocumentIds, setDeletingDocumentIds] = useState<Set<string>>(new Set());

  /**
   * SPEC-050: Wrap deleteMutation.mutate to track in-progress deletion IDs.
   * WHY: The row must show a "Deleting" state immediately after the user
   * confirms the delete dialog — before the server responds and the documents
   * query is invalidated.
   */
  const handleDeleteDocument = useCallback(
    (id: string) => {
      setDeletingDocumentIds((prev) => new Set([...prev, id]));
      deleteMutation.mutate(id, {
        onSettled: () => {
          setDeletingDocumentIds((prev) => {
            const next = new Set(prev);
            next.delete(id);
            return next;
          });
        },
      });
    },
    [deleteMutation],
  );

  // OODA-29: Document queries extracted to useDocumentQueries hook
  // VS-03: page=1 with large pageSize fetches everything at once for virtual scroll
  const VIRTUAL_PAGE_SIZE = 500;
  const { data, isLoading, isError, error, refetch, pipelineStatus, queryClient } = useDocumentQueries({
    tenantId: selectedTenantId,
    workspaceId: selectedWorkspaceId,
    currentPage: 1,
    pageSize: VIRTUAL_PAGE_SIZE,
    statusFilter,
  });

  // OODA-05: WebSocket subscription for real-time document status updates
  // WHY: Extracted to useDocumentWebSocket hook for SRP compliance
  useDocumentWebSocket(data?.items, queryClient);

  // OODA-04: Detect stuck documents using extracted hook
  useStuckDetection(data?.items, {
    timeout: 30000,
    checkInterval: 30000,
  });

  // OODA-21: Document dropzone with file validation
  const { getRootProps, getInputProps, isDragActive, openFileDialog } = useDocumentDropzone({
    onFilesAccepted: handleFilesAccepted,
    t,
  });

  // OODA-19: Filter and sort documents using extracted hook
  const { documents, totalCount, statusCounts } = useDocumentFiltering({
    documents: data?.items || [],
    searchQuery,
    statusFilter,
    sortField,
    sortDirection,
    pageSize: VIRTUAL_PAGE_SIZE,
    serverStatusCounts: data?.status_counts,
  });

  // SPEC-048: clear upload chrome when documents reach terminal state
  useEffect(() => {
    pruneTerminalUploads(documents ?? []);
    // SPEC-050-REPROCESS: also prune reprocess progress panels on terminal state
    pruneTerminalReprocessEntries(documents ?? []);
  }, [documents, pruneTerminalUploads, pruneTerminalReprocessEntries]);

  const pipelineUi = useMemo(
    () => resolvePipelineUiState(documents, pipelineStatus),
    [documents, pipelineStatus],
  );

  // Only mute siblings while a run is actively working (not merely queued)
  const workingRunDocumentIds = useMemo(() => {
    const ids = new Set<string>();
    for (const run of buildIngestionRunViews(documents).values()) {
      if (run.stageStatus === 'active') ids.add(run.documentId);
    }
    return ids;
  }, [documents]);

  // OODA-16: Bulk selection extracted to useBulkSelection hook
  // SPEC-050 GAP-FIX: Bulk delete confirmation callback.
  // WHY: useBulkSelection owns selection state; DocumentManager owns the
  // confirmation dialog. The callback bridges them (SRP + DIP).
  // Defined before useBulkSelection so it can be passed as onDeleteRequested.
  const handleBulkDeleteRequested = useCallback((selectedDocuments: Document[]) => {
    setBulkDeleteTargets(selectedDocuments);
    setBulkDeleteDialogOpen(true);
  }, []);

  const {
    selectedIds,
    selectedCount,
    isAllSelected,
    handleSelectAll,
    handleSelectOne,
    handleClearSelection,
    handleBulkDelete,
    handleBulkReprocess,
  } = useBulkSelection({ documents, onDeleteRequested: handleBulkDeleteRequested });

  // SPEC-050 GAP-FIX: Confirmed bulk delete — delete each document through
  // handleDeleteDocument so the per-row dimming state also applies.
  // Defined AFTER useBulkSelection because it uses handleClearSelection.
  const handleBulkDeleteConfirmed = useCallback(() => {
    for (const doc of bulkDeleteTargets) {
      handleDeleteDocument(doc.id);
    }
    setBulkDeleteTargets([]);
    setBulkDeleteDialogOpen(false);
    handleClearSelection();
  }, [bulkDeleteTargets, handleDeleteDocument, handleClearSelection]);

  // OODA-28: Document handlers extracted to useDocumentHandlers hook
  const {
    handleDocumentClick,
    handleDocumentDoubleClick,
    handleViewDetails,
    handlePreviewClose,
    handleViewInGraph,
    handleViewPdf,
  } = useDocumentHandlers({
    setSelectedDocument,
    setPreviewPanelOpen,
    setViewerDialogOpen,
    setViewerPdfId,
  });

  /**
   * OODA-19: Keyboard shortcuts for power users
   * WHY: Keyboard shortcuts improve efficiency and accessibility
   * 
   * Shortcuts:
   * - Escape: Clear selection or close preview panel
   * - Ctrl/Cmd + A: Select all documents
   * - R: Refresh document list (when not in input)
   */
  // OODA-18: Document keyboard shortcuts (Escape, Ctrl+A, R)
  useDocumentKeyboard({
    previewPanelOpen,
    selectedCount,
    onPreviewClose: handlePreviewClose,
    onSelectAll: handleSelectAll,
    onClearSelection: handleClearSelection,
    onRefresh: refetch,
    t,
  });

  // OODA-22 / SPEC-048 DEF-06: Working vs Queued in tab title
  useDocumentTitle({
    totalCount,
    processingCount: pipelineUi.activeDocCount,
    queuedCount: pipelineUi.waitingDocCount,
  });

  if (isError) {
    return <DocumentErrorAlert error={error} onRetry={refetch} />;
  }

  return (
    <div className="flex h-full overflow-hidden">
      {/* Main Content - Flex column for proper scroll zones */}
      <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
        {/* Fixed Header Zone */}
        <div className="shrink-0 px-4 pt-4 space-y-3 bg-background">
          <DocumentHeader
            totalCount={totalCount}
            failedCount={statusCounts.failed + statusCounts.cancelled}
            showPipelineIndicator={pipelineUi.showPipelineIndicator}
            pipelineAlertMode={pipelineUi.alertMode}
            activeDocCount={pipelineUi.activeDocCount}
            pipelineWaitingOnly={pipelineUi.isQueuedOnly}
            pipelineDialogOpen={pipelineDialogOpen}
            onPipelineDialogChange={setPipelineDialogOpen}
            onRefresh={refetch}
            tenantId={selectedTenantId ?? undefined}
            workspaceId={selectedWorkspaceId ?? undefined}
            documents={documents}
          />

          {/* OODA-30: Toolbar section extracted to DocumentToolbarSection */}
          <DocumentToolbarSection
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            statusFilter={statusFilter}
            onStatusFilterChange={setStatusFilter}
            sortField={sortField}
            onSortFieldChange={setSortField}
            sortDirection={sortDirection}
            onSortDirectionChange={setSortDirection}
            statusCounts={statusCounts}
            pipelineStatus={pipelineStatus}
            documents={documents}
            onOpenPipelineDetails={() => setPipelineDialogOpen(true)}
            onReprocessStuckDocuments={(stuckDocs) => {
              for (const doc of stuckDocs) {
                reprocessMutation.mutate({ id: doc.id, mode: 'full' });
              }
            }}
            isReprocessingStuck={reprocessMutation.isPending}
            getRootProps={getRootProps}
            getInputProps={getInputProps}
            isDragActive={isDragActive}
            openFileDialog={openFileDialog}
            pdfParserBackend={pdfParserBackend}
            onPdfParserBackendChange={setPdfParserBackend}
            selectedCount={selectedCount}
            onBulkReprocess={() => {
              // WHY: Open the bulk choice dialog so the user picks full
              // re-conversion vs. entity-only before reprocessing the batch.
              if (selectedCount === 0) return;
              setBulkReprocessOpen(true);
            }}
            onBulkDelete={handleBulkDelete}
            onClearSelection={handleClearSelection}
            uploadingFiles={uploadingFiles}
            isUploading={isUploading}
            onRemoveUpload={removeUploadingFile}
            onUploadComplete={handleUploadComplete}
            onUploadFailed={handleUploadFailed}
          />

        </div>

      {/* SPEC-050-REPROCESS: IngestionProgressPanel for each active reprocess.
          WHY: Fresh uploads show IngestionProgressPanel (stages, cost, ETA, cancel).
          Reprocess now gets identical feedback through the tracking state above.
          These panels appear between the toolbar and the table, in a shrink-0
          zone so they never overlap with scrollable content. */}
      {reprocessEntries.length > 0 && (
        <div
          className="shrink-0 space-y-1.5 px-4 pb-2"
          data-testid="spec050-reprocess-progress-panels"
        >
          {reprocessEntries.map((entry) => (
            <div
              key={entry.trackId}
              className="relative rounded-lg border bg-card/80 p-2 shadow-sm"
              data-testid="spec050-reprocess-panel"
              data-track-id={entry.trackId}
            >
              <IngestionProgressPanel
                trackId={entry.trackId}
                documentName={entry.documentName}
                compact={true}
                onComplete={() => removeReprocessEntry(entry.trackId)}
                onFailed={() => removeReprocessEntry(entry.trackId)}
                onCancel={() => removeReprocessEntry(entry.trackId)}
              />
            </div>
          ))}
        </div>
      )}

      {/* OODA-26: Table section extracted to DocumentTableSection */}
      <DocumentTableSection
        documents={documents}
        totalCount={totalCount}
        isLoading={isLoading}
        selectedIds={selectedIds}
        selectedDocument={selectedDocument}
        searchQuery={searchQuery}
        statusFilter={statusFilter}
        isAllSelected={isAllSelected}
        activeRunDocumentIds={workingRunDocumentIds}
        onSelectAll={handleSelectAll}
        onSelectOne={handleSelectOne}
        onRowClick={handleDocumentClick}
        onRowDoubleClick={handleDocumentDoubleClick}
        onViewDetails={handleViewDetails}
        onViewInGraph={handleViewInGraph}
        onViewPdf={handleViewPdf}
        onRetry={(id) => {
          // Pass document name for IngestionProgressPanel display
          const doc = documents.find((d) => d.id === id);
          const name = doc?.file_name || doc?.title || id.slice(0, 8);
          reprocessMutation.mutate({ id, name });
        }}
        onReprocess={(id) => {
          // WHY: Open the choice dialog for the target document so the user can
          // pick between full PDF re-conversion and entity-only re-extraction.
          const target = documents.find((d) => d.id === id) ?? null;
          setReprocessTarget(target ?? ({ id } as Document));
        }}
        onCancel={(trackId) => cancelMutation.mutate(trackId)}
        onDelete={handleDeleteDocument}
        isRetrying={reprocessMutation.isPending}
        isCancelling={cancelMutation.isPending}
        deletingDocumentIds={deletingDocumentIds}
        onUploadClick={openFileDialog}
        onClearFilter={() => {
          setStatusFilter('all');
          setSearchQuery('');
        }}
        sortField={sortField}
        sortDirection={sortDirection}
        onSort={handleColumnSort}
      />
      </div>

      {/* OADA-27: Right panel extracted to DocumentPreviewRightPanel */}
      <DocumentPreviewRightPanel
        isOpen={previewPanelOpen}
        onToggle={() => setPreviewPanelOpen(!previewPanelOpen)}
        onClose={handlePreviewClose}
        selectedDocument={selectedDocument}
        onDelete={(id) => {
          // SPEC-050 GAP-FIX: Route preview panel delete through confirm dialog.
          // WHY: Previously called handleDeleteDocument directly — no impact preview.
          const target = documents.find((d) => d.id === id) ?? selectedDocument;
          if (target) {
            setDeleteConfirmTarget(target);
          } else {
            // Fallback: direct delete if we can't find the document
            handleDeleteDocument(id);
          }
        }}
        onReprocess={(id) => {
          // WHY: Open the choice dialog for the target document so the user can
          // pick between full re-conversion and entity-only re-extraction. For
          // non-PDF docs the dialog still shows but the mode only affects PDFs.
          const target = documents.find((d) => d.id === id) ?? null;
          setReprocessTarget(target ?? ({ id } as Document));
        }}
        onViewInGraph={handleViewInGraph}
        onViewFull={(doc) => router.push(`/documents/${doc.id}`)}
        isDeleting={deleteMutation.isPending}
        isReprocessing={reprocessMutation.isPending}
        viewerDialogOpen={viewerDialogOpen}
        onViewerDialogChange={setViewerDialogOpen}
        viewerPdfId={viewerPdfId}
      />

      {/* Duplicate upload dialog — shown when backend returns duplicate_of */}
      <DuplicateUploadDialog
        open={pendingDuplicates.length > 0}
        duplicates={pendingDuplicates}
        onResolve={resolvePendingDuplicates}
      />

      <LargePdfAdmissionDialog
        open={largePdfAdmissionOpen}
        previews={largePdfPreviews}
        onOpenChange={setLargePdfAdmissionOpen}
        onConfirm={handleAdmissionConfirm}
        onCancel={handleAdmissionCancel}
      />

      {/* Reprocess choice dialog — lets the user choose full PDF re-conversion
          vs. entity-only re-extraction before queueing the reprocess task. */}
      <ReprocessDialog
        open={reprocessTarget !== null}
        document={reprocessTarget}
        onConfirm={(choice: ReprocessChoice) => {
          if (!reprocessTarget?.id) return;
          // SPEC-050-REPROCESS: Pass document name so IngestionProgressPanel shows
          // a meaningful filename instead of a truncated ID.
          const docName =
            reprocessTarget.file_name ||
            reprocessTarget.title ||
            reprocessTarget.id.slice(0, 8);
          reprocessMutation.mutate({
            id: reprocessTarget.id,
            mode: choice.mode,
            name: docName,
          });
          setReprocessTarget(null);
        }}
        onCancel={() => setReprocessTarget(null)}
      />

      {/* Bulk reprocess choice dialog — one mode applied to all selected docs. */}
      <BulkReprocessDialog
        open={bulkReprocessOpen}
        count={selectedCount}
        onConfirm={(choice: BulkReprocessChoice) => {
          setBulkReprocessOpen(false);
          void handleBulkReprocess(choice.mode);
        }}
        onCancel={() => setBulkReprocessOpen(false)}
      />

      {/* SPEC-050 GAP-FIX: Bulk delete confirmation (toolbar Delete button).
          WHY: Previously the toolbar Delete fired deleteDocument() directly with
          no confirmation or impact preview. This dialog gives the same quality
          of experience as the per-row delete in DocumentActionsMenu. */}
      <BulkDeleteConfirmDialog
        open={bulkDeleteDialogOpen}
        onOpenChange={(open) => {
          setBulkDeleteDialogOpen(open);
          if (!open) setBulkDeleteTargets([]);
        }}
        documents={bulkDeleteTargets}
        onConfirm={handleBulkDeleteConfirmed}
        isDeleting={deleteMutation.isPending}
      />

      {/* SPEC-050 GAP-FIX: Single delete confirm for preview panel.
          WHY: The preview panel's Delete button previously called handleDeleteDocument()
          directly — bypassing the confirm dialog. Now it opens this dialog first. */}
      <DeleteConfirmDialog
        open={deleteConfirmTarget !== null}
        onOpenChange={(open) => {
          if (!open) setDeleteConfirmTarget(null);
        }}
        document={deleteConfirmTarget}
        onConfirm={(id) => {
          handleDeleteDocument(id);
          setDeleteConfirmTarget(null);
        }}
        isDeleting={deleteMutation.isPending}
      />
    </div>
  );
}

export default DocumentManager;
