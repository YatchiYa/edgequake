'use client';

import type { StatusCounts } from '@/hooks/use-document-filtering';
import {
    buildIngestionRunViews,
    selectPrimaryRun,
} from '@/lib/pipeline/ingestion-run-view';
import { resolvePipelineUiState } from '@/lib/pipeline/pipeline-document-state';
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
  documents,
  onOpenPipelineDetails,
  onReprocessStuckDocuments,
  isReprocessingStuck,
  demotePipelineBanner = false,
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
  const pipelineUi = useMemo(
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
  // Hide chrome once every document is terminal (ignore stale pipelineStatus).
  // Demote non-stuck banners when the feedback zone already shows the same runs.
  const showBanner =
    pipelineUi.showPipelineIndicator &&
    (pipelineUi.isStuck || !demotePipelineBanner);
  const quietDropzone =
    pipelineUi.isActivelyProcessing ||
    primaryRun?.stageStatus === 'active';

  return (
    <>
      {/* Search and Filters */}
      <div className="flex flex-col sm:flex-row sm:items-center gap-3 pb-3 border-b">
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

      {/* Compact Upload Zone — quieter while a run is active (SPEC-048) */}
      <DocumentDropzone
        getRootProps={getRootProps}
        getInputProps={getInputProps}
        isDragActive={isDragActive}
        openFileDialog={openFileDialog}
        pdfParserBackend={pdfParserBackend}
        onPdfParserBackendChange={onPdfParserBackendChange}
        quiet={quietDropzone}
      />

      {/* Bulk Actions Bar */}
      <BatchActionsBar
        selectedCount={selectedCount}
        onReprocess={onBulkReprocess}
        onDelete={onBulkDelete}
        onClear={onClearSelection}
      />
    </>
  );
}
