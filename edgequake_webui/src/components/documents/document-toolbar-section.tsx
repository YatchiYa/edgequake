'use client';

import type { StatusCounts } from '@/hooks/use-document-filtering';
import type { Document, PipelineStatus } from '@/types';
import { BatchActionsBar } from './batch-actions-bar';
import { DocumentDropzone, type DocumentDropzoneProps } from './document-dropzone';
import type { DocStatus, SortField } from './document-filters';
import { DocumentFilters } from './document-filters';
import { DocumentSearchBar } from './document-search-bar';
import { ProcessingStatusSummary } from './processing-status-summary';
import type { UploadingFile } from './types';
import { UploadProgressList } from './upload-progress-list';
import { ActiveRunsPanel } from './active-runs-panel';
import {
  buildIngestionRunViews,
  selectPrimaryRun,
} from '@/lib/pipeline/ingestion-run-view';
import { resolvePipelineUiState } from '@/lib/pipeline/pipeline-document-state';
import { useMemo } from 'react';

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
  
  // Upload progress
  uploadingFiles: UploadingFile[];
  isUploading: boolean;
  onRemoveUpload: (index: number) => void;
  onUploadComplete: (index: number) => void;
  onUploadFailed: (index: number, error: string) => void;
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
  uploadingFiles,
  isUploading,
  onRemoveUpload,
  onUploadComplete,
  onUploadFailed,
}: DocumentToolbarSectionProps) {
  const runViews = useMemo(
    () => buildIngestionRunViews(documents),
    [documents],
  );
  const activeRuns = useMemo(() => [...runViews.values()], [runViews]);
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
  const showBanner = pipelineUi.showPipelineIndicator;
  const quietDropzone =
    pipelineUi.isActivelyProcessing ||
    primaryRun?.stageStatus === 'active';
  // Stuck attention path owns the narrative — hide "Active run" chrome
  const showActiveRuns = activeRuns.length > 0 && pipelineUi.alertMode !== 'stuck';

  // Client upload rows without track_id still use UploadProgressList
  const clientOnlyUploads = uploadingFiles.filter((f) => !f.trackId);
  const trackedUploads = uploadingFiles.filter((f) => Boolean(f.trackId));

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

      {/* Processing Status Summary */}
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

      {/* SPEC-048: server stepper for tracked runs; client FSM only pre-track */}
      {showActiveRuns ? <ActiveRunsPanel runs={activeRuns} /> : null}
      {(clientOnlyUploads.length > 0 ||
        (trackedUploads.length > 0 && activeRuns.length === 0)) && (
        <UploadProgressList
          uploadingFiles={
            activeRuns.length > 0 ? clientOnlyUploads : uploadingFiles
          }
          isUploading={isUploading}
          onRemove={onRemoveUpload}
          onComplete={onUploadComplete}
          onFailed={onUploadFailed}
        />
      )}
    </>
  );
}
