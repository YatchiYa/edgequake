'use client';

/**
 * Shared operation-phase presenter (cleaning / queued / deleting).
 *
 * DRY: ProgressPanelRow, ServerStageStepper, and delete session rows use the
 * same copy/vocabulary so badge + feedback zone never disagree.
 *
 * @implements SPEC-048 reprocess admission
 * @implements SPEC-050 delete progress parity
 */

import { cn } from '@/lib/utils';
import { BrushCleaning, Clock, Loader2, Trash2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

/** Admission + delete operation phases sharing one presenter. */
export type OperationPhaseKind = 'cleaning' | 'queued' | 'deleting';

/** @deprecated Prefer OperationPhaseKind — kept for reprocess call-site clarity. */
export type AdmissionPhaseKind = OperationPhaseKind;

export interface AdmissionPhaseRowProps {
  phase: OperationPhaseKind;
  /** Optional document name for ProgressPanelRow layout. */
  documentName?: string;
  /** Prefer server stage_message / WS phase_label when present. */
  stageMessage?: string | null;
  /** Optional N/M counts line (delete phases). */
  countsLabel?: string | null;
  /** compact = pill for stepper; row = feedback-zone with filename. */
  variant?: 'row' | 'pill';
  className?: string;
  'data-testid'?: string;
}

export function admissionPhaseCopy(
  phase: OperationPhaseKind,
  t: (key: string, fallback: string) => string,
  stageMessage?: string | null,
): { title: string; detail: string } {
  if (phase === 'cleaning') {
    const detail =
      (stageMessage && stageMessage.trim()) ||
      t(
        'documents.reprocess.cleaningDetail',
        'Removing prior knowledge graph…',
      );
    return {
      title: t('documents.reprocess.cleaning', 'Cleaning'),
      detail,
    };
  }
  if (phase === 'deleting') {
    const detail =
      (stageMessage && stageMessage.trim()) ||
      t('documents.delete.progressDetail', 'Removing document data…');
    return {
      title: t('documents.delete.progress', 'Deleting'),
      detail,
    };
  }
  const detail =
    (stageMessage && stageMessage.trim()) ||
    t('documents.reprocess.queuedDetail', 'Waiting for a free worker…');
  return {
    title: t('documents.reprocess.queued', 'Queued'),
    detail,
  };
}

/** @deprecated Alias — same as admissionPhaseCopy. */
export const operationPhaseCopy = admissionPhaseCopy;

function phaseTone(phase: OperationPhaseKind): {
  pill: string;
  dot: string;
  spinner: string;
  testId: string;
} {
  switch (phase) {
    case 'cleaning':
      return {
        pill: 'bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-200',
        dot: 'bg-rose-500',
        spinner: 'text-rose-500',
        testId: 'spec048-stage-cleaning',
      };
    case 'deleting':
      return {
        pill: 'bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-200',
        dot: 'bg-rose-500',
        spinner: 'text-rose-500',
        testId: 'spec050-stage-deleting',
      };
    default:
      return {
        pill: 'bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-200',
        dot: 'bg-amber-500',
        spinner: 'text-amber-500',
        testId: 'spec048-stage-queued',
      };
  }
}

/**
 * Visual chip / row for cleaning, queued, or deleting.
 */
export function AdmissionPhaseRow({
  phase,
  documentName,
  stageMessage,
  countsLabel,
  variant = 'row',
  className,
  'data-testid': testId,
}: AdmissionPhaseRowProps) {
  const { t } = useTranslation();
  const { title, detail } = admissionPhaseCopy(phase, t, stageMessage);
  const tone = phaseTone(phase);
  const detailWithCounts =
    countsLabel && countsLabel.trim()
      ? `${detail} · ${countsLabel.trim()}`
      : detail;

  if (variant === 'pill') {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-xs',
          tone.pill,
          className,
        )}
        data-testid={testId ?? tone.testId}
        data-admission={phase}
        data-state="pending"
        role="status"
      >
        <span
          className={cn('h-1.5 w-1.5 rounded-full animate-pulse', tone.dot)}
        />
        {phase === 'cleaning' ? (
          <BrushCleaning className="h-3 w-3 shrink-0" aria-hidden />
        ) : phase === 'deleting' ? (
          <Trash2 className="h-3 w-3 shrink-0" aria-hidden />
        ) : (
          <Clock className="h-3 w-3 shrink-0" aria-hidden />
        )}
        <span>
          {title} — {detailWithCounts}
        </span>
      </div>
    );
  }

  return (
    <div
      className={cn('flex items-center gap-2 py-1 pr-8', className)}
      data-testid={
        testId ??
        (phase === 'deleting' ? 'delete-progress-row' : 'reprocess-admission-row')
      }
      data-admission={phase}
      role="status"
      aria-live="polite"
      aria-atomic="true"
    >
      <Loader2
        className={cn('h-4 w-4 animate-spin shrink-0', tone.spinner)}
        aria-hidden
      />
      <div className="min-w-0 flex-1">
        {documentName ? (
          <p className="text-sm font-medium truncate">{documentName}</p>
        ) : null}
        <p className="text-xs font-medium text-foreground">{title}</p>
        <p className="text-xs text-muted-foreground">{detailWithCounts}</p>
      </div>
    </div>
  );
}

/** Alias for call sites that prefer operation naming. */
export const OperationPhaseRow = AdmissionPhaseRow;
