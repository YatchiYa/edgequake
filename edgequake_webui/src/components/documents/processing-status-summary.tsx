/**
 * ProcessingStatusSummary — thin wrapper around IngestionAlertBanner.
 *
 * @fileoverview Kept for backward-compatible imports (OODA-11).
 */
'use client';

import type { Document, PipelineStatus } from '@/types';
import { IngestionAlertBanner } from './ingestion-alert-banner';

export interface ProcessingStatusSummaryProps {
  pipelineStatus: PipelineStatus;
  documents: Document[];
  onOpenDetails: () => void;
  onReprocessStuck?: (documents: Document[]) => void;
  isReprocessing?: boolean;
}

export function ProcessingStatusSummary(props: ProcessingStatusSummaryProps) {
  return <IngestionAlertBanner {...props} />;
}

export default ProcessingStatusSummary;
