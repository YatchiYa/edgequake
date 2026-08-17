'use client';

import type { StatusCounts } from '@/hooks/use-document-filtering';
import {
    buildIngestionRunViews,
    selectPrimaryRun,
} from '@/lib/pipeline/ingestion-run-view';
<<<<<<< HEAD
import { resolvePipelineUiState } from '@/lib/pipeline/pipeline-document-state';
=======
import {
    resolvePipelineUiState,
    type PipelineUiState,
} from '@/lib/pipeline/pipeline-document-state';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import type { Document, PipelineStatus } from '@/types';
import { useMemo } from 'react';
import { BatchActionsBar } from './batch-actions-bar';
import { DocumentDropzone, type DocumentDropzoneProps } from './document-dropzone';
import type { DocStatus, SortField } from './document-filters';
import { DocumentFilters } from './document-filters';
import { DocumentSearchBar } from './document-search-bar';
import { ProcessingStatusSummary } from './processing-status-summary';

export interface DocumentToolbarSectionProps {
  // Search
  searchQuery: string;
  onSearchChange: (value: string) => void;
  
  // Filters
  statusFilter: DocStatus;
  onStatusFilterChange: (value: DocStatus) => void;
  sortField: SortField;
  onSortFieldChange: (value: SortField) => void;
  sortDirection: 'asc' | 'desc';
  onSortDirectionChange: (value: 'asc' | 'desc') => void;
  statusCounts: StatusCounts;
  
  // Pipeline status
  pipelineStatus: PipelineStatus | undefined;
  /** SPEC-099: shared resolve from shell — avoids dual busy detection */
  pipelineUi?: PipelineUiState;
  documents: Document[];
  onOpenPipelineDetails: () => void;
  onReprocessStuckDocuments?: (documents: Document[]) => void;
  isReprocessingStuck?: boolean;
  /**
   * When the unified feedback zone already narrates the same runs, hide the
   * non-stuck processing banner to avoid duplicate headlines. Stuck banner
   * (CTA) always stays visible.
   */
  demotePipelineBanner?: boolean;
<<<<<<< HEAD
=======
  /**
   * SPEC-099: collapse upload slot when feedback zone has live work.
   */
  collapseUploadSlot?: boolean;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  
  // Dropzone
  getRootProps: DocumentDropzoneProps['getRootProps'];
  getInputProps: DocumentDropzoneProps['getInputProps'];
  isDragActive: boolean;
  openFileDialog: () => void;
  pdfParserBackend: 'default' | 'vision' | 'edgeparse';
  onPdfParserBackendChange: (value: 'default' | 'vision' | 'edgeparse') => void;
  
  // Bulk actions
  selectedCount: number;
  onBulkReprocess: () => void;
  onBulkDelete: () => void;
  onClearSelection: () => void;
}

export function DocumentToolbarSection({
  searchQuery,
  onSearchChange,
  statusFilter,
  onStatusFilterChange,
  sortField,
  onSortFieldChange,
  sortDirection,
  onSortDirectionChange,
  statusCounts,
  pipelineStatus,
  pipelineUi: pipelineUiProp,
  documents,
  onOpenPipelineDetails,
  onReprocessStuckDocuments,
  isReprocessingStuck,
  demotePipelineBanner = false,
<<<<<<< HEAD
=======
  collapseUploadSlot = false,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  getRootProps,
  getInputProps,
  isDragActive,
  openFileDialog,
  pdfParserBackend,
  onPdfParserBackendChange,
  selectedCount,
  onBulkReprocess,
  onBulkDelete,
  onClearSelection,
}: DocumentToolbarSectionProps) {
  const runViews = useMemo(
    () => buildIngestionRunViews(documents),
    [documents],
  );
  const primaryRun = useMemo(() => selectPrimaryRun(runViews), [runViews]);
<<<<<<< HEAD
  const pipelineUi = useMemo(
=======
  const pipelineUiFallback = useMemo(
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    () =>
      resolvePipelineUiState(
        documents,
        pipelineStatus ?? {
          is_busy: Boolean(primaryRun && primaryRun.stageStatus === 'active'),
          running_tasks: primaryRun?.stageStatus === 'active' ? 1 : 0,
          queued_tasks: primaryRun?.stageStatus === 'pending' ? 1 : 0,
          completed_tasks: 0,
          failed_tasks: 0,
          tasks: [],
        },
      ),
    [documents, pipelineStatus, primaryRun],
  );
<<<<<<< HEAD
=======
  const pipelineUi = pipelineUiProp ?? pipelineUiFallback;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  // Hide chrome once every document is terminal (ignore stale pipelineStatus).
  // Demote non-stuck banners when the feedback zone already shows the same runs.
  const showBanner =
    pipelineUi.showPipelineIndicator &&
    (pipelineUi.isStuck || !demotePipelineBanner);
  const quietDropzone =
    pipelineUi.isActivelyProcessing ||
    primaryRun?.stageStatus === 'active';

<<<<<<< HEAD
=======
  const selectionMode = selectedCount > 0;

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  return (
    <>
      {/* SPEC-099 F-099-16: selection replaces primary toolbar row (not stacked) */}
      {selectionMode ? (
        <div
          className="pb-3 border-b"
          data-testid="spec099-selection-toolbar"
        >
          <BatchActionsBar
            selectedCount={selectedCount}
            onReprocess={onBulkReprocess}
            onDelete={onBulkDelete}
            onClear={onClearSelection}
          />
        </div>
      ) : (
        <div
          className="flex flex-col sm:flex-row sm:items-center gap-3 pb-3 border-b"
          data-testid="spec099-primary-toolbar"
        >
          <DocumentSearchBar
            value={searchQuery}
            onChange={onSearchChange}
          />
          <DocumentFilters
            status={statusFilter}
            onStatusChange={onStatusFilterChange}
            sortField={sortField}
            onSortFieldChange={onSortFieldChange}
            sortDirection={sortDirection}
            onSortDirectionChange={onSortDirectionChange}
            statusCounts={statusCounts}
          />
        </div>
      )}

      {/* Processing Status Summary — stuck CTA always; otherwise demote when zone owns narrative */}
      {showBanner && (
        <ProcessingStatusSummary
          pipelineStatus={
            pipelineStatus ?? {
              is_busy: Boolean(primaryRun && primaryRun.stageStatus === 'active'),
              running_tasks: primaryRun?.stageStatus === 'active' ? 1 : 0,
              queued_tasks: primaryRun?.stageStatus === 'pending' ? 1 : 0,
              completed_tasks: 0,
              failed_tasks: 0,
              tasks: [],
            }
          }
          documents={documents}
          onOpenDetails={onOpenPipelineDetails}
          onReprocessStuck={onReprocessStuckDocuments}
          isReprocessing={isReprocessingStuck}
        />
      )}

<<<<<<< HEAD
      {/* Compact Upload Zone — quieter while a run is active (SPEC-048) */}
=======
      {/* Compact Upload Zone — quieter while a run is active (SPEC-048);
          collapsed when feedback zone owns live narrative (SPEC-099) */}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      <DocumentDropzone
        getRootProps={getRootProps}
        getInputProps={getInputProps}
        isDragActive={isDragActive}
        openFileDialog={openFileDialog}
        pdfParserBackend={pdfParserBackend}
        onPdfParserBackendChange={onPdfParserBackendChange}
<<<<<<< HEAD
        quiet={quietDropzone}
      />

      {/* Bulk Actions Bar */}
      <BatchActionsBar
        selectedCount={selectedCount}
        onReprocess={onBulkReprocess}
        onDelete={onBulkDelete}
        onClear={onClearSelection}
=======
        quiet={quietDropzone && !collapseUploadSlot}
        collapsed={collapseUploadSlot}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      />
    </>
  );
}
