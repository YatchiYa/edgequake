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
import { ReasoningEffortSelect } from '@/components/settings/reasoning-effort-select';
import { useTranslation } from 'react-i18next';
import { MAX_UPLOAD_LABEL } from '@/lib/api/upload-limits';
import { formatWorkspaceDefaultPdfParserLabel } from '@/lib/pdf/resolve-pdf-parser-backend';
import { effectiveEffortWhenAuto } from '@/lib/settings/reasoning-effort-supported';
import type { PdfParserBackend } from '@/types/graph';

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
   * Workspace default `pdf_parser_backend` — shown in the inherit option label
   * (e.g. Workspace Default (Vision)). Falls back to server → Vision when unset.
   */
  workspacePdfParserBackend?: PdfParserBackend | null;
  /** SPEC-109: optional vision reasoning effort for VLM convert. */
  visionReasoningEffort?: string;
  onVisionReasoningEffortChange?: (value: string | undefined) => void;
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
  workspacePdfParserBackend,
  compact,
  /** When true, omit the side label — used inside the Vision combo row. */
  hideSideLabel,
  triggerClassName,
}: {
  pdfParserBackend: 'default' | 'vision' | 'edgeparse';
  onPdfParserBackendChange: (value: 'default' | 'vision' | 'edgeparse') => void;
  workspacePdfParserBackend?: PdfParserBackend | null;
  compact: boolean;
  hideSideLabel?: boolean;
  triggerClassName?: string;
}) {
  const { t } = useTranslation();
  const workspaceDefaultLabel = formatWorkspaceDefaultPdfParserLabel(
    t,
    workspacePdfParserBackend,
  );
  return (
    <div
      className={cn(
        'flex items-center gap-2',
        hideSideLabel ? 'min-w-0 flex-1' : 'shrink-0',
        compact && 'opacity-80',
      )}
      onClick={(event) => event.stopPropagation()}
      onKeyDown={(event) => event.stopPropagation()}
    >
      {!compact && !hideSideLabel && (
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
            triggerClassName ??
              (compact
                ? 'min-w-[10.5rem] w-auto max-w-[14rem] h-7 text-xs'
                : 'min-w-[13.5rem] w-auto max-w-[18rem] h-9'),
          )}
          data-testid="spec038-upload-parser-select"
          title={
            pdfParserBackend === 'default' ? workspaceDefaultLabel : undefined
          }
        >
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="default">{workspaceDefaultLabel}</SelectItem>
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
  workspacePdfParserBackend,
  visionReasoningEffort,
  onVisionReasoningEffortChange,
  quiet = false,
  collapsed = false,
}: DocumentDropzoneProps) {
  const { t } = useTranslation();
  const compact = quiet || collapsed;
  const showVisionEffort =
    pdfParserBackend === 'vision' && typeof onVisionReasoningEffortChange === 'function';

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
      <div
        className={cn(
          'shrink-0 flex flex-col justify-center gap-1',
          (compact || collapsed) && 'opacity-80',
        )}
        data-testid="upload-parser-vision-combo"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => event.stopPropagation()}
      >
        {showVisionEffort && !compact && !collapsed && (
          <div className="grid grid-cols-2 gap-2 w-[22rem]">
            <span className="text-[11px] font-medium text-muted-foreground truncate">
              {t('documents.upload.pdfParserShort', 'Parser')}
            </span>
            <span className="text-[11px] font-medium text-muted-foreground truncate">
              {t('documents.upload.visionReasoningEffort', 'Vision effort')}
            </span>
          </div>
        )}
        <div
          className={cn(
            'flex items-center gap-2',
            showVisionEffort && 'w-[22rem]',
          )}
        >
          <ParserSelect
            pdfParserBackend={pdfParserBackend}
            onPdfParserBackendChange={onPdfParserBackendChange}
            workspacePdfParserBackend={workspacePdfParserBackend}
            compact={compact || collapsed}
            hideSideLabel={showVisionEffort}
            triggerClassName={
              showVisionEffort
                ? compact || collapsed
                  ? 'w-full min-w-0 h-7 text-xs'
                  : 'w-full min-w-0 h-9'
                : undefined
            }
          />
          {showVisionEffort && (
            <div className="min-w-0 flex-1" data-testid="pdf-vision-reasoning-effort">
              <ReasoningEffortSelect
                value={visionReasoningEffort}
                onChange={onVisionReasoningEffortChange}
                hideLabel
                hideHint
                compactTrigger
                className="w-full min-w-0"
                triggerClassName={
                  compact || collapsed ? 'h-7 text-xs' : 'h-9'
                }
                label={t('documents.upload.visionReasoningEffort', 'Vision effort')}
                effectiveWhenAuto={effectiveEffortWhenAuto(
                  undefined,
                  undefined,
                  undefined,
                  'structured',
                )}
                data-testid="pdf-vision-reasoning-effort-select"
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
