'use client';

/**
 * @module ProgressPanelRow
 * @description DRY wrapper that renders PdfUploadProgress or IngestionProgressPanel
 * inside a standard card row with a dismiss (X) button.
 *
 * WHY (DRY principle):
 *   UploadProgressList and the reprocess panels in DocumentManager both need to
 *   show either PdfUploadProgress (PDFs) or IngestionProgressPanel (text docs)
 *   with an absolute-positioned dismiss button. Without this component that
 *   pattern was duplicated in two places.
 *
 * Design (SRP / OCP):
 *   - SRP: this component ONLY handles the row wrapper + component selection.
 *   - OCP: adding a new progress component variant requires only a new `variant`
 *     value, not changes to callers.
 *   - DIP: callers pass the resolved `trackId` and `isPdf`; this component does
 *     not know whether the source was an upload or a reprocess.
 *
 * @implements SPEC-051: Upload-parity progress for reprocess.
 */

import { Button } from '@/components/ui/button';
import { X } from 'lucide-react';
import { IngestionProgressPanel } from './ingestion-progress-panel';
import { PdfUploadProgress } from './pdf-upload-progress';

export interface ProgressPanelRowProps {
  /**
   * The LIVE task track_id to subscribe to for WebSocket progress events.
   *
   * For uploads this is always correct (returned by the upload endpoint).
   * For reprocess this MUST be derived from the documents cache AFTER the
   * worker updates it (not the "reprocess_..." batch ID from POST /reprocess).
   */
  trackId: string;
  /** Display name shown in the progress panel. */
  documentName: string;
  /**
   * When true, renders PdfUploadProgress (6-phase PDF pipeline).
   * When false (default), renders IngestionProgressPanel (stage-based).
   *
   * Set to `true` only for PDFs being processed with mode="full" so the
   * user sees PDF conversion phases (page N/M, etc.).
   */
  isPdf?: boolean;
  /** Callback when the user clicks the dismiss (X) button. */
  onRemove?: () => void;
  /** Forwarded to the progress component. */
  onComplete?: () => void;
  /** Forwarded to the progress component. */
  onFailed?: (error: string) => void;
  /** Forwarded to the progress component (cancel button inside panel). */
  onCancel?: () => void;
  /** data-testid for the row wrapper. */
  'data-testid'?: string;
  /** data-track-id attribute for Playwright selectors. */
  'data-track-id'?: string;
}

/**
 * A single progress row used in both UploadProgressList and the reprocess
 * panels section in DocumentManager.
 */
export function ProgressPanelRow({
  trackId,
  documentName,
  isPdf = false,
  onRemove,
  onComplete,
  onFailed,
  onCancel,
  'data-testid': testId,
  'data-track-id': dataTrackId,
}: ProgressPanelRowProps) {
  return (
    <div
      className="relative p-2 rounded-lg border bg-card"
      data-testid={testId ?? (isPdf ? 'pdf-progress-row' : 'text-ingestion-progress-row')}
      data-track-id={dataTrackId ?? trackId}
    >
      {isPdf ? (
        <PdfUploadProgress
          trackId={trackId}
          filename={documentName}
          compact={true}
          onComplete={onComplete}
          onFailed={onFailed}
        />
      ) : (
        <IngestionProgressPanel
          trackId={trackId}
          documentName={documentName}
          compact={true}
          onComplete={onComplete}
          onFailed={onFailed}
          onCancel={onCancel}
        />
      )}
      {onRemove && (
        <Button
          variant="ghost"
          size="icon"
          className="absolute top-1 right-1 h-6 w-6"
          onClick={onRemove}
          aria-label="Dismiss progress"
        >
          <X className="h-3 w-3" />
        </Button>
      )}
    </div>
  );
}
