/**
 * @module DocumentTableSection
 * @description Document table with virtual scrolling and a truly fixed header.
 *
 * Architecture:
 *   ┌─────────────────────────────────────────┐
 *   │  shrink-0  │ Header row (NEVER scrolls) │
 *   ├────────────┤─────────────────────────────┤
 *   │ flex-1     │ Body (overflow-y-auto)       │
 *   │ overflow   │ Virtual rows scroll here     │
 *   └────────────┴─────────────────────────────┘
 *
 * WHY: Any approach that puts the header INSIDE the scroll container
 * (sticky, absolute positioning, overflow:clip tricks) fails because the
 * scroll container IS the parent — it scrolls the header away.
 * The only reliable solution is to place the header in a shrink-0 sibling
 * of the scroll container, outside the scrollable region entirely.
 * Column width parity is maintained via a shared <colgroup> on both tables
 * using table-fixed layout.
 *
 * @implements FEAT0001 - Document list display
 * @implements FEAT0401 - Document filtering
 */
'use client';

import { Checkbox } from '@/components/ui/checkbox';
import {
    TableBody,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table';
import type { SortDirection, SortField } from '@/lib/documents/document-sort';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { Document } from '@/types';
import { FileText } from 'lucide-react';
import { memo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { DocumentTableRow } from './document-table-row';
import { DocumentTableStates } from './document-table-states';
import { SortableColumnHeader } from './sortable-column-header';

/** Estimated row height for the virtualizer (px). */
const ESTIMATED_ROW_HEIGHT = 52;

/**
 * Shared colgroup — defines column widths for BOTH the header table and
 * the body table so they stay in perfect alignment (DRY / single source of truth).
 * table-fixed on both tables means colgroup widths are authoritative.
 */
<<<<<<< HEAD
function TableColGroup() {
  return (
    <colgroup>
      <col style={{ width: '2.5rem' }} /><col /><col style={{ width: '8.5rem' }} /><col style={{ width: '5rem' }} /><col style={{ width: '5.5rem' }} /><col style={{ width: '9rem' }} /><col style={{ width: '9rem' }} /><col style={{ width: '10.5rem' }} />
=======
function TableColGroup({ showCostColumn }: { showCostColumn: boolean }) {
  return (
    <colgroup>
      <col style={{ width: '2.5rem' }} />
      <col />
      <col style={{ width: '8.5rem' }} />
      <col style={{ width: '5rem' }} />
      {showCostColumn ? <col style={{ width: '5.5rem' }} /> : null}
      <col style={{ width: '9rem' }} />
      <col style={{ width: '9rem' }} />
      <col style={{ width: '10.5rem' }} />
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
    </colgroup>
  );
}

/**
 * Props for DocumentTableSection component.
 */
export interface DocumentTableSectionProps {
  /** Documents to display (all, not paginated — virtualizer handles windowing) */
  documents: Document[];
  /** Total count for filtering info */
  totalCount: number;
  /** Whether data is loading */
  isLoading: boolean;
  /** Selected document IDs */
  selectedIds: Set<string>;
  /** Currently active document for preview */
  selectedDocument: Document | null;
  /** Current search query */
  searchQuery: string;
  /** Current status filter */
  statusFilter: string;
  /** Whether all are selected */
  isAllSelected: boolean;
  /** Document IDs currently in an active ingestion run (SPEC-048 mute others) */
  activeRunDocumentIds?: Set<string>;
  onSelectAll: (checked: boolean) => void;
  onSelectOne: (id: string, checked: boolean) => void;
  onRowClick: (doc: Document) => void;
  onRowDoubleClick: (doc: Document) => void;
  onViewDetails: (doc: Document) => void;
  onViewInGraph: (doc: Document) => void;
  onViewPdf: (doc: Document) => void;
  onRetry: (id: string) => void;
  onReprocess: (id: string) => void;
  onCancel: (trackId: string) => void;
  onDelete: (id: string) => void;
  isRetrying: boolean;
  isCancelling: boolean;
  /** SPEC-050: IDs of documents currently being deleted (for row dimming). */
  deletingDocumentIds?: Set<string>;
  onUploadClick: () => void;
  onClearFilter?: () => void;
  /** True when ingest/upload is busy but the list is still empty */
  isBusyUpdating?: boolean;
  /** Active sort field (shared with toolbar — DRY) */
  sortField: SortField;
  /** Active sort direction */
  sortDirection: SortDirection;
  /** Column header sort toggle */
  onSort: (field: SortField) => void;
<<<<<<< HEAD
=======
  /** SPEC-099: Cost column opt-in */
  showCostColumn?: boolean;
  /** SPEC-099: overflow honesty affordance */
  overflowLabel?: string | null;
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
}

/**
 * Document table with virtual scrolling.
 * VS-01: Spacer-row virtualizer pattern — header stays sticky, columns align.
 * WHY: Wrapped in memo so re-renders from preview-panel/dialog state changes
 * in DocumentManager don't cause the entire table to re-render.
 */
export const DocumentTableSection = memo(function DocumentTableSection({
  documents,
  totalCount,
  isLoading,
  selectedIds,
  selectedDocument,
  searchQuery,
  statusFilter,
  isAllSelected,
  activeRunDocumentIds,
  onSelectAll,
  onSelectOne,
  onRowClick,
  onRowDoubleClick,
  onViewDetails,
  onViewInGraph,
  onViewPdf,
  onRetry,
  onReprocess,
  onCancel,
  onDelete,
  isRetrying,
  isCancelling,
  deletingDocumentIds,
  onUploadClick,
  onClearFilter,
  isBusyUpdating = false,
  sortField,
  sortDirection,
  onSort,
<<<<<<< HEAD
}: DocumentTableSectionProps) {
  const { t } = useTranslation();
  const scrollRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: documents.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    overscan: 8,
  });

  const virtualItems = virtualizer.getVirtualItems();
  const totalVirtualHeight = virtualizer.getTotalSize();
  const paddingTop = virtualItems.length > 0 ? virtualItems[0].start : 0;
  const paddingBottom =
    virtualItems.length > 0
      ? totalVirtualHeight - virtualItems[virtualItems.length - 1].end
      : 0;

  const showTable = !isLoading && documents.length > 0;
=======
  showCostColumn = false,
  overflowLabel = null,
}: DocumentTableSectionProps) {
  const { t } = useTranslation();
  const scrollRef = useRef<HTMLDivElement>(null);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  const virtualizer = useVirtualizer({
    count: documents.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    overscan: 8,
  });

  const virtualItems = virtualizer.getVirtualItems();
  const totalVirtualHeight = virtualizer.getTotalSize();
  // Offset for the visible window inside the spacer (TanStack absolute pattern).
  // Do NOT use <tr height=padding> spacers — table min-height leaks into
  // document.scrollHeight and creates the page-level white-band scrollbar.
  const windowOffset = virtualItems[0]?.start ?? 0;

  const showTable = !isLoading && documents.length > 0;

  // SPEC-099: must be a real flex child (not a Fragment). A Fragment breaks the
  // min-h-0 / flex-1 chain → virtualizer padding blows page height, header +
  // dropzone scroll away, and a large empty white band appears below rows.
  return (
<<<<<<< HEAD
    <>
=======
    <div
      className="flex min-h-0 min-w-0 flex-1 flex-col overflow-clip"
      data-testid="documents-inventory-section"
    >
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
      {/* ── ZONE 1: shrink-0 header (never inside the scroll container) ── */}
      <div className="shrink-0 px-4 pt-3 bg-background">
        {/* Count / filter info */}
        {showTable && (
          <div className="flex items-center gap-2 mb-1.5">
            <FileText className="h-3.5 w-3.5 text-muted-foreground" aria-hidden="true" />
<<<<<<< HEAD
            <span className="text-xs text-muted-foreground tabular-nums">
=======
            <span
              className="text-xs text-muted-foreground tabular-nums"
              data-testid="spec099-inventory-count"
            >
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              {searchQuery || statusFilter !== 'all'
                ? t('documents.filter.showingFiltered', '{{count}} of {{total}}', {
                    count: documents.length,
                    total: totalCount,
                  })
                : t('documents.documentCount', '{{count}} documents', { count: totalCount })}
            </span>
<<<<<<< HEAD
=======
            {overflowLabel ? (
              <span
                className="text-xs text-amber-700 dark:text-amber-400"
                data-testid="spec099-scale-overflow"
              >
                {overflowLabel}
              </span>
            ) : null}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
          </div>
        )}

        {/* Column header row — physically outside the scroll container */}
        {showTable && (
          <div className="border border-border border-b-0 rounded-t-lg bg-muted/40 overflow-hidden shadow-sm">
            <table className="w-full table-fixed caption-bottom text-sm" role="presentation">
<<<<<<< HEAD
              <TableColGroup />
=======
              <TableColGroup showCostColumn={showCostColumn} />
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
              <TableHeader>
                <TableRow className="hover:bg-transparent">
                  <TableHead scope="col" className="rounded-tl-lg">
                    <Checkbox
                      checked={isAllSelected}
                      onCheckedChange={(checked) => onSelectAll(!!checked)}
                      aria-label={t('documents.bulk.selectAll', 'Select all')}
<<<<<<< HEAD
                    />
                  </TableHead>
                  <SortableColumnHeader
                    field="title"
                    label={t('documents.table.title', 'Title')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="status"
                    label={t('documents.table.status', 'Status')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="entity_count"
                    label={t('documents.table.entities', 'Entities')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                    align="center"
                  />
                  <SortableColumnHeader
                    field="cost_usd"
                    label={t('documents.table.cost', 'Cost')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                    align="center"
                  />
                  <SortableColumnHeader
                    field="created_at"
                    label={t('documents.table.created', 'Created')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="updated_at"
                    label={t('documents.table.updated', 'Last Updated')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <TableHead scope="col" className="rounded-tr-lg">
                    <span className="sr-only">{t('documents.table.actions', 'Actions')}</span>
                  </TableHead>
                </TableRow>
              </TableHeader>
            </table>
          </div>
        )}
      </div>

      {/* ── ZONE 2: flex-1 scroll container (body only) ── */}
      <div ref={scrollRef} className="flex-1 min-h-0 overflow-y-auto px-4 pb-3">
        {/* Loading / empty states — shown when no table */}
        <DocumentTableStates
          isLoading={isLoading}
          isEmpty={documents.length === 0}
          onUploadClick={onUploadClick}
          statusFilter={statusFilter}
          searchQuery={searchQuery}
          onClearFilter={onClearFilter}
          isBusyUpdating={isBusyUpdating}
        />

        {showTable && (
          <div
            className="border border-border rounded-b-lg overflow-hidden shadow-sm bg-background"
            aria-label={t('documents.table.ariaLabel', 'Documents list')}
          >
            <table className="w-full table-fixed caption-bottom text-sm">
              <TableColGroup />
              <TableBody>
                {paddingTop > 0 && (
                  <tr aria-hidden="true"><td style={{ height: paddingTop }} /></tr>
                )}
                {virtualItems.map((virtualRow) => {
                  const doc = documents[virtualRow.index];
                  if (!doc) return null;
                  const isLiveRun = activeRunDocumentIds?.has(doc.id) ?? false;
                  const isBackground =
                    Boolean(activeRunDocumentIds && activeRunDocumentIds.size > 0) &&
                    !isLiveRun;
                  return (
                    <DocumentTableRow
                      key={doc.id}
                      doc={doc}
                      index={virtualRow.index}
                      isSelected={selectedIds.has(doc.id)}
                      isActive={selectedDocument?.id === doc.id}
                      isBackground={isBackground}
                      searchQuery={searchQuery}
                      onSelect={onSelectOne}
                      onClick={onRowClick}
                      onDoubleClick={onRowDoubleClick}
                      onViewDetails={onViewDetails}
                      onViewInGraph={onViewInGraph}
                      onViewPdf={onViewPdf}
                      onRetry={onRetry}
                      onReprocess={onReprocess}
                      onCancel={onCancel}
                      onDelete={onDelete}
                      isRetrying={isRetrying}
                      isCancelling={isCancelling}
                      isDeleting={deletingDocumentIds?.has(doc.id) ?? false}
                    />
                  );
                })}
                {paddingBottom > 0 && (
                  <tr aria-hidden="true"><td style={{ height: paddingBottom }} /></tr>
                )}
              </TableBody>
=======
                    />
                  </TableHead>
                  <SortableColumnHeader
                    field="title"
                    label={t('documents.table.title', 'Title')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="status"
                    label={t('documents.table.status', 'Status')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="entity_count"
                    label={t('documents.table.entities', 'Entities')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                    align="center"
                  />
                  {showCostColumn ? (
                    <SortableColumnHeader
                      field="cost_usd"
                      label={t('documents.table.cost', 'Cost')}
                      activeField={sortField}
                      direction={sortDirection}
                      onSort={onSort}
                      align="center"
                    />
                  ) : null}
                  <SortableColumnHeader
                    field="created_at"
                    label={t('documents.table.created', 'Created')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <SortableColumnHeader
                    field="updated_at"
                    label={t('documents.table.updated', 'Last Updated')}
                    activeField={sortField}
                    direction={sortDirection}
                    onSort={onSort}
                  />
                  <TableHead scope="col" className="rounded-tr-lg">
                    <span className="sr-only">{t('documents.table.actions', 'Actions')}</span>
                  </TableHead>
                </TableRow>
              </TableHeader>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
            </table>
          </div>
        )}
      </div>
<<<<<<< HEAD
    </>
=======

      {/* ── ZONE 2: flex-1 scroll container (body only) ── */}
      <div
        ref={scrollRef}
        className="min-h-[9rem] flex-1 overflow-x-hidden overflow-y-auto overscroll-contain px-4 pb-3 [contain:paint]"
        data-testid="documents-table-scroll"
      >
        {/* Loading / empty states — shown when no table */}
        <DocumentTableStates
          isLoading={isLoading}
          isEmpty={documents.length === 0}
          onUploadClick={onUploadClick}
          statusFilter={statusFilter}
          searchQuery={searchQuery}
          onClearFilter={onClearFilter}
          isBusyUpdating={isBusyUpdating}
        />

        {showTable && (
          <div
            className="relative w-full"
            style={{ height: totalVirtualHeight }}
            data-testid="documents-virtual-spacer"
          >
            <div
              className="absolute left-0 right-0 border border-border rounded-b-lg overflow-hidden shadow-sm bg-background"
              style={{ transform: `translateY(${windowOffset}px)` }}
              aria-label={t('documents.table.ariaLabel', 'Documents list')}
            >
              <table className="w-full table-fixed caption-bottom text-sm">
                <TableColGroup showCostColumn={showCostColumn} />
                <TableBody>
                  {virtualItems.map((virtualRow) => {
                    const doc = documents[virtualRow.index];
                    if (!doc) return null;
                    const bareId = doc.id.replace(/^staging:/, "");
                    const isLiveRun =
                      Boolean(activeRunDocumentIds?.has(doc.id)) ||
                      Boolean(activeRunDocumentIds?.has(bareId));
                    const isBackground =
                      Boolean(activeRunDocumentIds && activeRunDocumentIds.size > 0) &&
                      !isLiveRun;
                    return (
                      <DocumentTableRow
                        key={doc.id}
                        doc={doc}
                        index={virtualRow.index}
                        isSelected={selectedIds.has(doc.id)}
                        isActive={selectedDocument?.id === doc.id}
                        isBackground={isBackground}
                        isLiveRun={isLiveRun}
                        showCostColumn={showCostColumn}
                        searchQuery={searchQuery}
                        onSelect={onSelectOne}
                        onClick={onRowClick}
                        onDoubleClick={onRowDoubleClick}
                        onViewDetails={onViewDetails}
                        onViewInGraph={onViewInGraph}
                        onViewPdf={onViewPdf}
                        onRetry={onRetry}
                        onReprocess={onReprocess}
                        onCancel={onCancel}
                        onDelete={onDelete}
                        isRetrying={isRetrying}
                        isCancelling={isCancelling}
                        isDeleting={
                          (deletingDocumentIds?.has(doc.id) ?? false) ||
                          (doc.status || '').toLowerCase() === 'deleting' ||
                          (doc.current_stage || '').toLowerCase() === 'deleting'
                        }
                      />
                    );
                  })}
                </TableBody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  );
});

export default DocumentTableSection;
