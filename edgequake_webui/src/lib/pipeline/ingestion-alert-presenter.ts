/**
 * Presentation mapping for ingestion alert modes (banner, header, dialog).
 *
 * WHY: Single place for colors, icons, and i18n keys — DRY across UI surfaces.
 */

import type { TFunction } from 'i18next';
import type { PipelineUiState } from './pipeline-document-state';

export type IngestionAlertMode = 'working' | 'queued' | 'stuck' | 'mixed';

export type IngestionAlertVariant = 'blue' | 'amber' | 'rose' | 'mixed';

export interface IngestionAlertHeadline {
  mode: IngestionAlertMode;
  variant: IngestionAlertVariant;
  text: string;
  showSpinner: boolean;
  showClock: boolean;
  showAlert: boolean;
  dataTestId: string;
}

export function buildIngestionAlertHeadline(
  state: PipelineUiState,
  t: TFunction,
): IngestionAlertHeadline {
  const mode = state.alertMode;
  switch (mode) {
    case 'mixed':
      return {
        mode,
        variant: 'mixed',
        text: t('pipeline.processingAndWaiting', '{{processing}} processing · {{waiting}} waiting', {
          processing: state.activeDocCount,
          waiting: state.waitingDocCount,
        }),
        showSpinner: true,
        showClock: false,
        showAlert: false,
        dataTestId: 'ingestion-alert-mixed',
      };
    case 'working':
      return {
        mode,
        variant: 'blue',
        text: t('pipeline.processing', 'Processing {{count}} document(s)', {
          count: state.activeDocCount,
        }),
        showSpinner: true,
        showClock: false,
        showAlert: false,
        dataTestId: 'ingestion-alert-working',
      };
    case 'stuck':
      return {
        mode,
        variant: 'rose',
        text: t('pipeline.stuckHeadline', '{{count}} document(s) need attention', {
          count: state.stuckDocCount,
        }),
        showSpinner: false,
        showClock: false,
        showAlert: true,
        dataTestId: 'ingestion-alert-stuck',
      };
    case 'queued':
      return {
        mode,
        variant: 'amber',
        text: t('pipeline.waitingToStart', '{{count}} document(s) waiting to start', {
          count: state.waitingDocCount,
        }),
        showSpinner: false,
        showClock: true,
        showAlert: false,
        dataTestId: 'ingestion-alert-queued',
      };
  }
}

export function ingestionAlertContainerClass(variant: IngestionAlertVariant): string {
  switch (variant) {
    case 'blue':
      return 'bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-950/50';
    case 'amber':
      return 'bg-amber-50 dark:bg-amber-950/30 border-amber-200 dark:border-amber-800 hover:bg-amber-100 dark:hover:bg-amber-950/50';
    case 'rose':
      return 'bg-rose-50 dark:bg-rose-950/30 border-rose-200 dark:border-rose-800 hover:bg-rose-100 dark:hover:bg-rose-950/50';
    case 'mixed':
      return 'bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800 hover:bg-blue-100 dark:hover:bg-blue-950/50';
  }
}

export function ingestionAlertTitleClass(variant: IngestionAlertVariant): string {
  switch (variant) {
    case 'blue':
    case 'mixed':
      return 'text-blue-700 dark:text-blue-300';
    case 'amber':
      return 'text-amber-800 dark:text-amber-300';
    case 'rose':
      return 'text-rose-800 dark:text-rose-300';
  }
}

export function ingestionAlertDetailClass(variant: IngestionAlertVariant): string {
  switch (variant) {
    case 'blue':
    case 'mixed':
      return 'text-blue-600 dark:text-blue-400';
    case 'amber':
      return 'text-amber-700 dark:text-amber-400';
    case 'rose':
      return 'text-rose-700 dark:text-rose-400';
  }
}
