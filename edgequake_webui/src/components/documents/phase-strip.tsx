/**
 * SPEC-091 IS3 — default 4-phase ActiveRuns strip (Admit / Prepare / Extract / Materialize).
 * Full wire UnifiedStage chips stay on Pipeline / Details (ServerStageStepper).
 */

"use client";

import { cn } from "@/lib/utils";
import {
  mapWireStageToPhase,
  PHASE_STRIP_LABELS,
  PHASE_STRIP_ORDER,
  type IngestionPhaseId,
  type IngestionRunView,
} from "@/lib/pipeline/ingestion-run-view";
import { buildStageTimeline } from "@/lib/pipeline/stage-timeline";

function phaseStatus(
  phase: IngestionPhaseId,
  active: IngestionPhaseId,
  failed: boolean,
): "done" | "active" | "pending" | "failed" {
  const ai = PHASE_STRIP_ORDER.indexOf(active);
  const pi = PHASE_STRIP_ORDER.indexOf(phase);
  if (failed && pi === ai) return "failed";
  if (pi < ai) return "done";
  if (pi === ai) return "active";
  return "pending";
}

function statusClasses(
  status: "done" | "active" | "pending" | "failed",
): string {
  switch (status) {
    case "done":
      return "text-emerald-700 dark:text-emerald-400";
    case "active":
      return "bg-sky-100 text-sky-800 dark:bg-sky-950 dark:text-sky-200";
    case "failed":
      return "bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-200";
    default:
      return "text-muted-foreground";
  }
}

function dotClasses(status: "done" | "active" | "pending" | "failed"): string {
  switch (status) {
    case "done":
      return "bg-emerald-500";
    case "active":
      return "bg-sky-500 animate-pulse";
    case "failed":
      return "bg-rose-500";
    default:
      return "bg-muted-foreground/40";
  }
}

export interface PhaseStripProps {
  run: IngestionRunView;
  className?: string;
}

export function PhaseStrip({ run, className }: PhaseStripProps) {
  const active = mapWireStageToPhase(run.stage);
  const failed = run.stageStatus === "failed" || run.stage === "failed";
  // IS-AC-06: preserve wire stage ids as data attributes / contract hooks.
  // Include skipped (e.g. merge mode) so e2e can assert data-state=skipped.
  // Stages omitted from the timeline entirely (non-PDF converting) stay absent.
  const wireSteps = buildStageTimeline(run).steps;

  return (
    <div
      className={cn("flex flex-wrap items-center gap-1.5 text-xs", className)}
      data-testid="spec091-phase-strip"
      data-wire-stage={run.stage}
      data-phase={active}
    >
      {PHASE_STRIP_ORDER.map((phase) => {
        const status = phaseStatus(phase, active, failed);
        return (
          <span
            key={phase}
            className={cn(
              "inline-flex items-center gap-1 rounded px-1.5 py-0.5",
              statusClasses(status),
            )}
            data-testid={`spec091-phase-${phase}`}
            data-state={status}
            data-wire-stage={run.stage}
            title={PHASE_STRIP_LABELS[phase]}
          >
            <span
              className={cn("h-1.5 w-1.5 rounded-full", dotClasses(status))}
            />
            {PHASE_STRIP_LABELS[phase]}
          </span>
        );
      })}
      {/* Screen-reader + e2e wire markers (skipped converting omitted). */}
      <span className="sr-only" aria-hidden="true">
        {wireSteps.map((step) => (
          <span
            key={step.id}
            data-testid={`spec048-stage-${step.id}`}
            data-state={step.status}
            data-wire-stage={step.id}
          />
        ))}
      </span>
    </div>
  );
}
