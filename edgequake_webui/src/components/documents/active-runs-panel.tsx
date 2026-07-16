/**
 * SPEC-048: Active runs panel — morph target after track_id (DEF-10).
 *
 * Progress UX (first principles):
 * - Prefer stage N/M when countable (FP-03)
 * - Indeterminate when unknown / queued (FP-04)
 * - Overall bar is a weighted estimate, capped <100% until completed
 */

"use client";

import { ServerStageStepper } from "@/components/documents/server-stage-stepper";
import {
  stageDisplayName,
  type IngestionRunView,
} from "@/lib/pipeline/ingestion-run-view";
import { buildStageTimeline } from "@/lib/pipeline/stage-timeline";
import { Progress } from "@/components/ui/progress";

interface ActiveRunsPanelProps {
  runs: IngestionRunView[];
}

export function ActiveRunsPanel({ runs }: ActiveRunsPanelProps) {
  const active = runs.filter(
    (r) =>
      r.stageStatus === "active" ||
      r.stageStatus === "pending" ||
      (Boolean(r.trackId) &&
        r.stage !== "completed" &&
        r.stage !== "failed" &&
        r.stageStatus !== "failed"),
  );
  if (active.length === 0) return null;

  const anyWorking = active.some((r) => r.stageStatus === "active");
  const title = anyWorking
    ? active.length > 1
      ? "Active runs"
      : "Active run"
    : active.length > 1
      ? "Queued runs"
      : "Queued run";

  return (
    <div
      className="space-y-3 rounded-lg border border-sky-200/80 bg-sky-50/40 p-3 dark:border-sky-900 dark:bg-sky-950/20"
      data-testid="spec048-active-runs-panel"
    >
      <div className="flex items-baseline justify-between gap-2">
        <div className="text-sm font-medium tracking-tight">{title}</div>
        <div className="text-xs tabular-nums text-muted-foreground">
          {active.length}
        </div>
      </div>
      {active.map((run) => {
        const timeline = buildStageTimeline(run);
        const admission = timeline.admissionPhase;
        const isAdmission = Boolean(admission);
        const overallPct = Math.round(timeline.overallProgress01 * 100);
        const stagePct =
          typeof timeline.stageProgress01 === "number"
            ? Math.round(timeline.stageProgress01 * 100)
            : undefined;
        const hasStageCounts = Boolean(
          run.counts && run.counts.total > 0,
        );

        return (
          <div
            key={run.documentId}
            className="space-y-2 rounded-md border border-border/80 bg-background p-2.5 shadow-sm"
            data-testid="spec048-active-run-card"
            data-document-id={run.documentId}
            data-stage={run.stage}
            data-mode={run.mode ?? "full"}
            data-admission={admission ?? "running"}
          >
            <div className="flex items-center justify-between gap-2 text-sm">
              <span className="truncate font-medium text-foreground">
                {run.filename}
              </span>
              <span
                className="shrink-0 text-xs tabular-nums text-sky-700 dark:text-sky-300"
                data-testid="spec048-run-headline"
              >
                {run.counts
                  ? `${stageDisplayName(run.stage)} · ${run.counts.current}/${run.counts.total}`
                  : stageDisplayName(run.stage)}
              </span>
            </div>
            <ServerStageStepper run={run} />

            {isAdmission ? (
              <div
                className="h-1.5 w-full overflow-hidden rounded bg-muted"
                data-testid="spec048-run-progress-indeterminate"
                data-admission={admission ?? undefined}
              >
                <div
                  className={
                    admission === "cleaning"
                      ? "h-full w-1/3 animate-pulse rounded bg-rose-400/70"
                      : "h-full w-1/3 animate-pulse rounded bg-amber-400/70"
                  }
                />
              </div>
            ) : (
              <div className="space-y-1.5">
                {/* FP-03: stage bar from real counts when known */}
                {hasStageCounts && typeof stagePct === "number" ? (
                  <div
                    className="space-y-0.5"
                    data-testid="spec048-stage-progress"
                  >
                    <div className="flex items-center justify-between gap-2 text-[10px] text-muted-foreground">
                      <span>
                        This stage
                        {timeline.stageCountsLabel
                          ? ` · ${timeline.stageCountsLabel}`
                          : ""}
                      </span>
                      <span className="tabular-nums">{stagePct}%</span>
                    </div>
                    <Progress
                      value={stagePct}
                      className="h-1.5 [&_[data-slot=progress-indicator]]:bg-sky-500"
                    />
                  </div>
                ) : (
                  <div
                    className="h-1.5 w-full overflow-hidden rounded bg-muted"
                    data-testid="spec048-run-progress-indeterminate"
                  >
                    <div className="h-full w-1/3 animate-pulse rounded bg-sky-400/70" />
                  </div>
                )}

                {/* FP-10: weighted overall estimate — never 100% mid-flight */}
                <div
                  className="space-y-0.5"
                  data-testid="spec048-overall-progress"
                >
                  <div className="flex items-center justify-between gap-2 text-[10px] text-muted-foreground">
                    <span>Overall (est.)</span>
                    <span
                      className="tabular-nums"
                      data-testid="spec048-run-overall-pct"
                    >
                      {overallPct}%
                    </span>
                  </div>
                  <Progress
                    value={overallPct}
                    className="h-1 [&_[data-slot=progress-indicator]]:bg-sky-400/80"
                  />
                </div>
              </div>
            )}

            {run.mode && run.mode !== "full" ? (
              <div
                className="text-[11px] text-muted-foreground"
                data-testid="spec048-run-mode"
              >
                Reprocess mode: {run.mode}
              </div>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

export default ActiveRunsPanel;
