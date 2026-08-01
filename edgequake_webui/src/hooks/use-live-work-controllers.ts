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

  const runViewOpts = useMemo(() => {
    const pending =
      pipelineStatus?.pending_tasks ?? pipelineStatus?.queued_tasks ?? 0;
    const processing =
      pipelineStatus?.processing_tasks ?? pipelineStatus?.running_tasks ?? 0;
    return {
      hasQueueCoverage: hasQueueCoverage(pipelineStatus, pending, processing),
    };
  }, [pipelineStatus]);

  const isLiveRunIds = useMemo(() => {
    const ids = new Set<string>();
    for (const run of buildIngestionRunViews(documents, runViewOpts).values()) {
      if (
        run.stageStatus === "active" ||
        run.stageStatus === "pending" ||
        run.stage === "stopping"
      ) {
        ids.add(run.documentId);
      }
    }
    return ids;
  }, [documents, runViewOpts]);

  const allRuns = useMemo(
    () => [...buildIngestionRunViews(documents, runViewOpts).values()],
    [documents, runViewOpts],
  );

  const stagesByDocId = useMemo(() => {
    const map = new Map<string, string | null | undefined>();
    for (const doc of documents ?? []) {
      map.set(doc.id, doc.current_stage);
    }
    return map;
  }, [documents]);

  const queuingSessionDocIds = useMemo(
    () => documentIdsWithQueuingSession(reprocessEntries, stagesByDocId),
    [reprocessEntries, stagesByDocId],
  );

  const activeRunsForPanel = useMemo(
    () => filterRunsExcludingQueuingSession(allRuns, queuingSessionDocIds),
    [allRuns, queuingSessionDocIds],
  );

  const showActiveRuns = activeRunsForPanel.length > 0;

  const activeRunsDisplayed = useMemo(() => {
    if (pipelineUiAlertMode !== "stuck" || !stuckDocIds || stuckDocIds.size === 0) {
      return activeRunsForPanel;
    }
    const stuckRuns = activeRunsForPanel.filter((r) =>
      stuckDocIds.has(r.documentId),
    );
    return stuckRuns.length > 0 ? stuckRuns : activeRunsForPanel;
  }, [activeRunsForPanel, pipelineUiAlertMode, stuckDocIds]);

  const sessionReprocessEntries = useMemo(
    () =>
      reprocessEntries.filter((entry) => {
        if (shouldShowReprocessQueuingPanel(entry.trackId)) return true;
        if (shouldUsePdfReprocessPanel(Boolean(entry.isPdf), entry.mode)) {
          return true;
        }
        if (!showActiveRuns) return true;
        return !activeRunsDisplayed.some((r) => r.documentId === entry.documentId);
      }),
    [reprocessEntries, showActiveRuns, activeRunsDisplayed],
  );

  const clientOnlyUploads = useMemo(
    () => uploadingFiles.filter((f) => !f.trackId),
    [uploadingFiles],
  );
  const trackedUploads = useMemo(
    () => uploadingFiles.filter((f) => Boolean(f.trackId)),
    [uploadingFiles],
  );
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
