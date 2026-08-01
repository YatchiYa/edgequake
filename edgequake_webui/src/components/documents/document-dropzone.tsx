'use client';

import { cn } from '@/lib/utils';
import { Upload } from 'lucide-react';
import type React from 'react';
import type { DropzoneInputProps, DropzoneRootProps } from 'react-dropzone';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useTranslation } from 'react-i18next';
import { MAX_UPLOAD_LABEL } from '@/lib/api/upload-limits';

/**
 * Props for the DocumentDropzone component.
 */
export interface DocumentDropzoneProps {
  /** Props to spread on the dropzone container */
  getRootProps: <T extends DropzoneRootProps>(props?: T) => T;
  /** Props to spread on the hidden file input */
  getInputProps: <T extends DropzoneInputProps>(props?: T) => T;
  /** Whether a drag operation is currently active over the zone */
  isDragActive: boolean;
  /** Function to programmatically open file dialog (explicit click handler) */
  openFileDialog: () => void;
  /** Per-upload PDF parser backend override. */
  pdfParserBackend: 'default' | 'vision' | 'edgeparse';
  /** Change handler for the PDF parser override selector. */
  onPdfParserBackendChange: (value: 'default' | 'vision' | 'edgeparse') => void;
  /**
   * SPEC-048: compact chrome while ingestion is working so progress UI stays primary.
   */
  quiet?: boolean;
  /**
   * SPEC-099 LAW-099-4: denser band when feedback zone has live work.
   * Always remains a full-width drop target (never removed).
   */
  collapsed?: boolean;
}

function ParserSelect({
  pdfParserBackend,
  onPdfParserBackendChange,
  compact,
}: {
  pdfParserBackend: 'default' | 'vision' | 'edgeparse';
  onPdfParserBackendChange: (value: 'default' | 'vision' | 'edgeparse') => void;
  compact: boolean;
}) {
  const { t } = useTranslation();
  return (
    <div
      className={cn('flex items-center gap-2 shrink-0', compact && 'opacity-80')}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => event.stopPropagation()}
    >
      {!compact && (
        <span className="text-xs text-muted-foreground whitespace-nowrap">
          {t('documents.upload.pdfParser', 'Parser for this upload')}
        </span>
      )}
      <Select
        value={pdfParserBackend}
        onValueChange={(value: 'default' | 'vision' | 'edgeparse') =>
          onPdfParserBackendChange(value)
        }
      >
        <SelectTrigger
          className={cn(
            'bg-background',
            compact ? 'w-[120px] h-7 text-xs' : 'w-[190px] h-9',
          )}
          data-testid="spec038-upload-parser-select"
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="default">
            {t('documents.upload.pdfParserDefault', 'Workspace Default')}
          </SelectItem>
          <SelectItem value="vision">
            {t('documents.upload.pdfParserVision', 'Vision')}
          </SelectItem>
          <SelectItem value="edgeparse">
            {t('documents.upload.pdfParserEdgeParse', 'EdgeParse')}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

/**
 * Always-on file upload drop zone (idle expand / busy collapse).
 *
 * SPEC-099: the drop zone is never removed — collapse only shrinks chrome.
 * Drag-and-drop, click, and keyboard activation remain available.
 *
 * @implements FEAT0001 - Document ingestion with entity extraction
 * @implements SPEC-099 F-099-04 - collapse when feedback zone has live work
 */
export function DocumentDropzone({
  getRootProps,
  getInputProps,
  isDragActive,
  openFileDialog,
  pdfParserBackend,
  onPdfParserBackendChange,
  quiet = false,
  collapsed = false,
}: DocumentDropzoneProps) {
  const { t } = useTranslation();
  const compact = quiet || collapsed;

  const rootProps = getRootProps({
    onClick: (e: React.MouseEvent) => {
      e.stopPropagation();
      openFileDialog();
    },
    onKeyDown: (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        e.stopPropagation();
        openFileDialog();
      }
    },
    role: 'button' as const,
    'aria-label': collapsed
      ? t('documents.upload.uploadCollapsed', 'Add files — click or drop')
      : t('documents.upload.uploadDrop', 'Upload files by clicking or dragging'),
    tabIndex: 0,
  });

  return (
    <div
      {...rootProps}
      data-testid="document-dropzone"
      data-upload="true"
      data-quiet={quiet ? 'true' : 'false'}
      data-collapsed={collapsed ? 'true' : 'false'}
      className={cn(
        // Always a full-width single-line drop band — never a multi-paragraph hero
        // that steals the inventory flex budget (SPEC-099 scroll layout).
        'w-full border-dashed cursor-pointer transition-all duration-200',
        'flex items-center gap-3 min-w-0',
        collapsed
          ? 'rounded-md border px-3 py-1.5 gap-2'
          : compact
            ? 'rounded-lg border px-3 py-2 gap-2'
            : 'rounded-lg border-2 px-4 py-2.5 gap-3',
        isDragActive
          ? 'border-primary bg-primary/5 ring-2 ring-primary/20'
          : collapsed || quiet
            ? 'border-muted-foreground/20 bg-muted/15 hover:border-primary/40 hover:bg-muted/25'
            : 'border-muted-foreground/20 hover:border-primary/50 hover:bg-muted/30',
      )}
    >
      <input {...getInputProps()} data-testid="document-dropzone-input" />
      <div
        className={cn(
          'rounded-lg transition-all shrink-0',
          compact || collapsed ? 'p-1.5' : 'p-2',
          isDragActive ? 'bg-primary/10' : 'bg-muted/50',
        )}
      >
        <Upload
          className={cn(
            'transition-all duration-200',
            compact || collapsed ? 'h-4 w-4' : 'h-5 w-5',
            isDragActive ? 'text-primary scale-110' : 'text-muted-foreground',
          )}
        />
      </div>
      <div className="min-w-0 flex-1 overflow-hidden">
        {isDragActive ? (
          <p className="truncate text-sm font-medium text-primary">
            {t('documents.upload.uploadDropActive', 'Drop files here')}
          </p>
        ) : collapsed ? (
          <p className="truncate text-xs text-muted-foreground">
            {t('documents.upload.addFilesDrop', 'Drop files here or click to add')}
          </p>
        ) : quiet ? (
          <p className="truncate text-xs text-muted-foreground">
            {t(
              'documents.upload.uploadWhileWorking',
              'Add more files anytime · max {{limit}}',
              { limit: MAX_UPLOAD_LABEL },
            )}
          </p>
        ) : (
          <p
            className="truncate text-sm text-muted-foreground"
            title={t(
              'documents.upload.uploadDropWithLimit',
              'Drag & drop or click to upload • TXT, MD, JSON, PDF, PNG, JPG, GIF, WEBP (max {{limit}})',
              { limit: MAX_UPLOAD_LABEL },
            )}
          >
            {t(
              'documents.upload.uploadDropWithLimit',
              'Drag & drop or click to upload • TXT, MD, JSON, PDF, PNG, JPG, GIF, WEBP (max {{limit}})',
              { limit: MAX_UPLOAD_LABEL },
            )}
          </p>
        )}
      </div>
      <ParserSelect
        pdfParserBackend={pdfParserBackend}
        onPdfParserBackendChange={onPdfParserBackendChange}
        compact={compact || collapsed}
      />
    </div>
  );
}
