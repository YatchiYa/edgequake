/**
 * SPEC-048 / SPEC-086: Active runs panel — shared IngestionRunCard presenter.
 * PDF converting page detail nests under the card (not a second upload product).
 *
 * SPEC-086 dual-run UX: never mix orphan failed shells under "Active runs"
 * beside a live PDF — partition Working/Queued vs Needs attention.
 *
 * Cancelled: compact ack with grace auto-dismiss + Dismiss (not Failed/Queued forever).
 */

"use client";

import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useReducer } from "react";
import { IngestionRunCard } from "@/components/documents/ingestion-run-card";
import { PdfUploadProgress } from "@/components/documents/pdf-upload-progress";
import { Button } from "@/components/ui/button";
import { cancelTask } from "@/lib/api/edgequake";
import {
  CANCELLED_RUN_GRACE_MS,
  cancelledRunKey,
  cancelledRunRemainingMs,
  dismissCancelledRun,
  isCancelledRunVisible,
  observeCancelledRun,
  pauseCancelledRunGrace,
  resumeCancelledRunGrace,
} from "@/lib/pipeline/cancelled-run-lifecycle";
import {
  shouldNestPdfPageMeter,
  type IngestionRunView,
} from "@/lib/pipeline/ingestion-run-view";

interface ActiveRunsPanelProps {
  runs: IngestionRunView[];
  /** Delete/remove a failed attention shell (orphan staging re-upload class). */
  onDismissFailed?: (documentId: string) => void;
}

export function isOrphanFailedAttention(run: IngestionRunView): boolean {
  return (
    run.stageStatus === "failed" &&
    /re-upload|interrupted|orphaned staging|document received,\s*starting processing|prior interrupted/i.test(
      run.message,
    )
  );
}

function runLifecycleKey(run: IngestionRunView): string {
  return cancelledRunKey(run.documentId, run.trackId);
}

/**
 * Live work + brief cancelled acknowledgement (grace window).
 * Stopping always stays; cancelled only while lifecycle says visible.
 */
export function isLiveWorkingOrQueued(
  run: IngestionRunView,
  nowMs: number = Date.now(),
): boolean {
  if (isOrphanFailedAttention(run)) return false;
  if (run.stage === "stopping") return true;
  if (run.stage === "cancelled" || run.stageStatus === "cancelled") {
    const key = runLifecycleKey(run);
    const opts = { cancelledAt: run.updatedAt };
    observeCancelledRun(key, nowMs, opts);
    return isCancelledRunVisible(key, nowMs, CANCELLED_RUN_GRACE_MS, opts);
  }
  return (
    run.stageStatus === "active" ||
    run.stageStatus === "pending" ||
    (Boolean(run.trackId) &&
      run.stage !== "completed" &&
      run.stage !== "failed" &&
      run.stageStatus !== "failed")
  );
}

/** Split live work from orphan attention shells (testable SSOT). */
export function partitionActiveRuns(
  runs: IngestionRunView[],
  nowMs: number = Date.now(),
): {
  working: IngestionRunView[];
  attention: IngestionRunView[];
} {
  const working: IngestionRunView[] = [];
  const attention: IngestionRunView[] = [];
  for (const run of runs) {
    if (isOrphanFailedAttention(run)) {
      attention.push(run);
    } else if (isLiveWorkingOrQueued(run, nowMs)) {
      working.push(run);
    }
  }
  return { working, attention };
}

/** Honest section title — never "Queued run" for cancelled-only. */
export function workingSectionTitle(working: IngestionRunView[]): string {
  if (working.length === 0) return "Active run";
  const allCancelled = working.every(
    (r) => r.stage === "cancelled" || r.stageStatus === "cancelled",
  );
  if (allCancelled) {
    return working.length > 1 ? "Cancelled" : "Cancelled";
  }
  const allStoppingOrCancelled = working.every(
    (r) =>
      r.stage === "stopping" ||
      r.stage === "cancelled" ||
      r.stageStatus === "cancelled",
  );
  const anyStopping = working.some((r) => r.stage === "stopping");
  if (allStoppingOrCancelled && anyStopping) {
    return working.length > 1 ? "Stopping…" : "Stopping…";
  }
  const anyWorking = working.some((r) => r.stageStatus === "active");
  if (anyWorking) {
    return working.length > 1 ? "Active runs" : "Active run";
  }
  return working.length > 1 ? "Queued runs" : "Queued run";
}

function renderRunCard(
  run: IngestionRunView,
  onDismissFailed: ((documentId: string) => void) | undefined,
  onDismissCancelled: ((run: IngestionRunView) => void) | undefined,
  onCancelTrack: ((trackId: string) => void) | undefined,
  onTick: () => void,
) {
  const isCancelled =
    run.stage === "cancelled" || run.stageStatus === "cancelled";
  const key = runLifecycleKey(run);

  return (
    <IngestionRunCard
      key={run.documentId}
      run={run}
      data-testid="spec048-active-run-card"
      onCancel={
        run.trackId &&
        run.stageStatus !== "failed" &&
        run.stage !== "stopping" &&
        !isCancelled &&
        onCancelTrack
          ? () => onCancelTrack(run.trackId!)
          : undefined
      }
      onDismiss={
        isCancelled && onDismissCancelled
          ? () => onDismissCancelled(run)
          : isOrphanFailedAttention(run) && onDismissFailed
            ? () => onDismissFailed(run.documentId)
            : undefined
      }
      onGracePause={
        isCancelled
          ? () => {
              pauseCancelledRunGrace(key);
              onTick();
            }
          : undefined
      }
      onGraceResume={
        isCancelled
          ? () => {
              resumeCancelledRunGrace(key);
              onTick();
            }
          : undefined
      }
      nestedDetail={
        // LAW-IS2 / F-IS-06: second progress product only when list lacks page counts.
        shouldNestPdfPageMeter(run) && run.trackId ? (
          <PdfUploadProgress
            trackId={run.trackId}
            filename={run.filename}
            compact
            nested
          />
        ) : undefined
      }
    />
  );
}

export function ActiveRunsPanel({
  runs,
  onDismissFailed,
}: ActiveRunsPanelProps) {
  const queryClient = useQueryClient();
  const [, tick] = useReducer((n: number) => n + 1, 0);

  const onCancelTrack = (trackId: string) => {
    void import("@/lib/documents/cancel-intent").then(
      ({ pinCancelIntent, patchDocumentsCancelOptimistic }) => {
        pinCancelIntent(trackId);
        patchDocumentsCancelOptimistic(queryClient, trackId);
      },
    );
    void cancelTask(trackId)
      .catch(() => {
        /* terminal cancelled may still be on KV */
      })
      .finally(() => {
        void queryClient.invalidateQueries({ queryKey: ["documents"] });
        void queryClient.invalidateQueries({ queryKey: ["tasks"] });
        void queryClient.invalidateQueries({ queryKey: ["pipeline-status"] });
      });
  };

  const onDismissCancelled = (run: IngestionRunView) => {
    dismissCancelledRun(runLifecycleKey(run));
    tick();
  };

  const nowMs = Date.now();
  const { working, attention } = partitionActiveRuns(runs, nowMs);

  // Auto-dismiss cancelled acks when grace expires (pause-aware).
  useEffect(() => {
    const cancelled = working.filter(
      (r) => r.stage === "cancelled" || r.stageStatus === "cancelled",
    );
    if (cancelled.length === 0) return;

    let timer: ReturnType<typeof setTimeout> | undefined;
    const schedule = () => {
      const now = Date.now();
      let soonest = Number.POSITIVE_INFINITY;
      for (const run of cancelled) {
        const rem = cancelledRunRemainingMs(
          runLifecycleKey(run),
          now,
          CANCELLED_RUN_GRACE_MS,
        );
        if (rem > 0 && rem < soonest) soonest = rem;
        if (rem === 0) {
          tick();
          return;
        }
      }
      if (!Number.isFinite(soonest)) return;
      timer = setTimeout(() => {
        tick();
      }, soonest + 25);
    };
    schedule();
    return () => {
      if (timer) clearTimeout(timer);
    };
  }, [working, runs]);

  if (working.length === 0 && attention.length === 0) return null;

  const dismissAll = () => {
    if (!onDismissFailed) return;
    for (const run of attention) {
      onDismissFailed(run.documentId);
    }
  };

  return (
    <div
      className="space-y-4 rounded-lg border border-sky-200/80 bg-sky-50/40 p-3 dark:border-sky-900 dark:bg-sky-950/20"
      data-testid="spec048-active-runs-panel"
    >
      {working.length > 0 && (
        <section
          className="space-y-3"
          data-testid="spec048-active-runs-working"
        >
          <div className="flex items-baseline justify-between gap-2">
            <div className="text-sm font-medium tracking-tight">
              {workingSectionTitle(working)}
            </div>
            <div className="text-xs tabular-nums text-muted-foreground">
              {working.length}
            </div>
          </div>
          {working.map((run) =>
            renderRunCard(
              run,
              onDismissFailed,
              onDismissCancelled,
              onCancelTrack,
              tick,
            ),
          )}
        </section>
      )}

      {attention.length > 0 && (
        <section
          className="space-y-3 rounded-md border border-amber-200/80 bg-amber-50/50 p-2.5 dark:border-amber-900/60 dark:bg-amber-950/30"
          data-testid="spec086-needs-attention"
        >
          <div className="flex items-center justify-between gap-2">
            <div className="min-w-0">
              <div className="text-sm font-medium tracking-tight">
                Needs attention
              </div>
              <p className="mt-0.5 text-xs text-muted-foreground">
                Prior interrupted upload(s) — dismiss and re-upload. Not part of
                the current active run.
              </p>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <span className="text-xs tabular-nums text-muted-foreground">
                {attention.length}
              </span>
              {onDismissFailed && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs"
                  data-testid="spec086-dismiss-all-attention"
                  onClick={dismissAll}
                >
                  Dismiss all
                </Button>
              )}
            </div>
          </div>
          {attention.map((run) =>
            renderRunCard(
              run,
              onDismissFailed,
              onDismissCancelled,
              onCancelTrack,
              tick,
            ),
          )}
        </section>
      )}
    </div>
  );
}

export default ActiveRunsPanel;
