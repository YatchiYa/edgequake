/**
 * SPEC-099 — live-work controller view-model (runs / upload / reprocess / delete).
 *
 * Pure derivation from shell-owned state so DocumentManager stays a thin composer.
 */
"use client";

import {
  documentIdsWithQueuingSession,
  filterRunsExcludingQueuingSession,
  shouldShowReprocessQueuingPanel,
} from "@/lib/documents/progress-admit";
import { hasPanelVisibleActiveRuns } from "@/lib/pipeline/active-runs-partition";
import {
  buildIngestionRunViews,
  type IngestionRunView,
} from "@/lib/pipeline/ingestion-run-view";
import { hasQueueCoverage } from "@/lib/pipeline/pipeline-document-state";
import type { Document, PipelineStatus } from "@/types";
import { useMemo } from "react";
import type { UploadingFile } from "@/components/documents/types";
import { shouldUsePdfReprocessPanel } from "@/hooks/use-reprocess-tracking";

export interface ReprocessEntryLike {
  documentId: string;
  trackId: string;
  documentName: string;
  isPdf?: boolean;
  mode?: string;
}

export interface UseLiveWorkControllersInput {
  documents: Document[];
  pipelineStatus: PipelineStatus | undefined;
  uploadingFiles: UploadingFile[];
  reprocessEntries: ReprocessEntryLike[];
  deleteSessionCount: number;
  pipelineUiAlertMode?: string;
  stuckDocIds?: Set<string>;
}

export interface LiveWorkControllers {
  hasLiveWork: boolean;
  isLiveRunIds: Set<string>;
  showActiveRuns: boolean;
  showUploadList: boolean;
  activeRunsDisplayed: IngestionRunView[];
  uploadFilesForList: UploadingFile[];
  sessionReprocessEntries: ReprocessEntryLike[];
  allRuns: IngestionRunView[];
}

/**
 * Pure live-work derivation (unit-testable SSOT for the hook).
 * Ordinary Failed runs do not open ActiveRuns / feedback zone.
 */
export function deriveLiveWorkControllers(
  input: UseLiveWorkControllersInput,
): LiveWorkControllers {
  const {
    documents,
    pipelineStatus,
    uploadingFiles,
    reprocessEntries,
    deleteSessionCount,
    pipelineUiAlertMode,
    stuckDocIds,
  } = input;

  const pending =
    pipelineStatus?.pending_tasks ?? pipelineStatus?.queued_tasks ?? 0;
  const processing =
    pipelineStatus?.processing_tasks ?? pipelineStatus?.running_tasks ?? 0;
  const runViewOpts = {
    hasQueueCoverage: hasQueueCoverage(pipelineStatus, pending, processing),
  };

  const runViews = buildIngestionRunViews(documents, runViewOpts);
  const isLiveRunIds = new Set<string>();
  for (const run of runViews.values()) {
    if (
      run.stageStatus === "active" ||
      run.stageStatus === "pending" ||
      run.stage === "stopping"
    ) {
      isLiveRunIds.add(run.documentId);
    }
  }

  const allRuns = [...runViews.values()];

  const stagesByDocId = new Map<string, string | null | undefined>();
  for (const doc of documents ?? []) {
    stagesByDocId.set(doc.id, doc.current_stage);
  }

  const queuingSessionDocIds = documentIdsWithQueuingSession(
    reprocessEntries,
    stagesByDocId,
  );

  const activeRunsForPanel = filterRunsExcludingQueuingSession(
    allRuns,
    queuingSessionDocIds,
  );

  let activeRunsDisplayed = activeRunsForPanel;
  if (
    pipelineUiAlertMode === "stuck" &&
    stuckDocIds &&
    stuckDocIds.size > 0
  ) {
    const stuckRuns = activeRunsForPanel.filter((r) =>
      stuckDocIds.has(r.documentId),
    );
    activeRunsDisplayed =
      stuckRuns.length > 0 ? stuckRuns : activeRunsForPanel;
  }

  // SSOT with ActiveRunsPanel: ordinary Failed must not open the zone.
  const showActiveRuns = hasPanelVisibleActiveRuns(activeRunsDisplayed);

  const sessionReprocessEntries = reprocessEntries.filter((entry) => {
    if (shouldShowReprocessQueuingPanel(entry.trackId)) return true;
    if (shouldUsePdfReprocessPanel(Boolean(entry.isPdf), entry.mode)) {
      return true;
    }
    if (!showActiveRuns) return true;
    return !activeRunsDisplayed.some((r) => r.documentId === entry.documentId);
  });

  const clientOnlyUploads = uploadingFiles.filter((f) => !f.trackId);
  const trackedUploads = uploadingFiles.filter((f) => Boolean(f.trackId));
  const showUploadList =
    clientOnlyUploads.length > 0 ||
    (trackedUploads.length > 0 && !showActiveRuns);
  const uploadFilesForList = showActiveRuns ? clientOnlyUploads : uploadingFiles;

  const hasLiveWork =
    showActiveRuns ||
    showUploadList ||
    sessionReprocessEntries.length > 0 ||
    deleteSessionCount > 0;

  return {
    hasLiveWork,
    isLiveRunIds,
    showActiveRuns,
    showUploadList,
    activeRunsDisplayed,
    uploadFilesForList,
    sessionReprocessEntries,
    allRuns,
  };
}

export function useLiveWorkControllers(
  input: UseLiveWorkControllersInput,
): LiveWorkControllers {
  const {
    documents,
    pipelineStatus,
    uploadingFiles,
    reprocessEntries,
    deleteSessionCount,
    pipelineUiAlertMode,
    stuckDocIds,
  } = input;

  return useMemo(
    () =>
      deriveLiveWorkControllers({
        documents,
        pipelineStatus,
        uploadingFiles,
        reprocessEntries,
        deleteSessionCount,
        pipelineUiAlertMode,
        stuckDocIds,
      }),
    [
      documents,
      pipelineStatus,
      uploadingFiles,
      reprocessEntries,
      deleteSessionCount,
      pipelineUiAlertMode,
      stuckDocIds,
    ],
  );
}
