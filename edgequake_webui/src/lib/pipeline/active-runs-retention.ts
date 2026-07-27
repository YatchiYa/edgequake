/**
 * SPEC-086 / LAW-28: Active Runs retention for cancelled terminals.
 *
 * Cancelled cards dwell briefly then leave Active Runs (document row remains).
 * Stopping stays until the API leaves ui_phase=stopping.
 *
 * TTL prefers durable `updatedAt` (survives refresh). Observation clock is a
 * fallback when timestamps are missing.
 */

import type { IngestionRunView } from "./ingestion-run-view";
import { parseRunUpdatedAtMs } from "./cancelled-active-run-dismiss";

/** Auto-dismiss Cancelled Active Run cards after this dwell. */
export const CANCELLED_ACTIVE_RUN_TTL_MS = 12_000;

export type CancelledObservationClock = {
  /** documentId → first observation time (ms since epoch) of terminal cancel. */
  firstSeenAt: Map<string, number>;
};

export function createCancelledObservationClock(): CancelledObservationClock {
  return { firstSeenAt: new Map() };
}

/**
 * Record first-seen timestamps for newly observed cancelled runs.
 * Clears entries that are no longer cancelled / no longer present.
 */
export function noteCancelledObservations(
  clock: CancelledObservationClock,
  runs: IngestionRunView[],
  nowMs: number = Date.now(),
): void {
  const liveCancelled = new Set<string>();
  for (const run of runs) {
    if (run.stageStatus === "cancelled" || run.stage === "cancelled") {
      liveCancelled.add(run.documentId);
      if (!clock.firstSeenAt.has(run.documentId)) {
        // Prefer durable cancel timestamp from the API.
        const durable = parseRunUpdatedAtMs(run.updatedAt);
        clock.firstSeenAt.set(run.documentId, durable ?? nowMs);
      }
    }
  }
  for (const id of [...clock.firstSeenAt.keys()]) {
    if (!liveCancelled.has(id)) {
      clock.firstSeenAt.delete(id);
    }
  }
}

/** Anchor time for cancelled TTL: updatedAt > observation clock. */
export function cancelledRetentionAnchorMs(
  run: Pick<IngestionRunView, "documentId" | "updatedAt">,
  clock: CancelledObservationClock,
): number | undefined {
  return (
    parseRunUpdatedAtMs(run.updatedAt) ?? clock.firstSeenAt.get(run.documentId)
  );
}

/** True while a cancelled run should still appear in Active Runs Working. */
export function isCancelledWithinTtl(
  clock: CancelledObservationClock,
  run: Pick<IngestionRunView, "documentId" | "updatedAt">,
  nowMs: number = Date.now(),
  ttlMs: number = CANCELLED_ACTIVE_RUN_TTL_MS,
): boolean {
  const anchor = cancelledRetentionAnchorMs(run, clock);
  if (anchor === undefined) return true; // not yet noted — include this frame
  return nowMs - anchor < ttlMs;
}

export function isCancelledRun(
  run: Pick<IngestionRunView, "stage" | "stageStatus">,
): boolean {
  return run.stageStatus === "cancelled" || run.stage === "cancelled";
}

export function isStoppingRun(
  run: Pick<IngestionRunView, "stage" | "stageStatus">,
): boolean {
  return run.stageStatus === "stopping" || run.stage === "stopping";
}

/**
 * Filter Working runs: drop cancelled past TTL or locally dismissed.
 * Stopping is never TTL-evicted here (API drives exit).
 */
export function filterWorkingRunsForRetention(
  working: IngestionRunView[],
  opts: {
    clock: CancelledObservationClock;
    dismissedCancelledIds: ReadonlySet<string>;
    nowMs?: number;
    ttlMs?: number;
  },
): IngestionRunView[] {
  const nowMs = opts.nowMs ?? Date.now();
  const ttlMs = opts.ttlMs ?? CANCELLED_ACTIVE_RUN_TTL_MS;
  noteCancelledObservations(opts.clock, working, nowMs);
  return working.filter((run) => {
    if (!isCancelledRun(run)) return true;
    if (opts.dismissedCancelledIds.has(run.documentId)) return false;
    return isCancelledWithinTtl(opts.clock, run, nowMs, ttlMs);
  });
}

/**
 * Future TTL deadlines for cancelled Active Run cards still within the dwell.
 * Already-expired rows are omitted — callers must not sync-bump for those
 * (render-time filter already hides them; sync setState loops).
 */
export function cancelledRetentionDeadlines(
  working: IngestionRunView[],
  opts: {
    clock: CancelledObservationClock;
    dismissedCancelledIds: ReadonlySet<string>;
    nowMs?: number;
    ttlMs?: number;
  },
): Array<{ id: string; deadline: number }> {
  const nowMs = opts.nowMs ?? Date.now();
  const ttlMs = opts.ttlMs ?? CANCELLED_ACTIVE_RUN_TTL_MS;
  noteCancelledObservations(opts.clock, working, nowMs);
  const out: Array<{ id: string; deadline: number }> = [];
  for (const run of working) {
    if (!isCancelledRun(run)) continue;
    if (opts.dismissedCancelledIds.has(run.documentId)) continue;
    const anchor = cancelledRetentionAnchorMs(run, opts.clock) ?? nowMs;
    const deadline = anchor + ttlMs;
    if (deadline > nowMs) {
      out.push({ id: run.documentId, deadline });
    }
  }
  out.sort((a, b) => a.id.localeCompare(b.id));
  return out;
}

/** Section title when only cancelled dwell remains. */
export function workingSectionTitleForRuns(working: IngestionRunView[]): string {
  const anyLive = working.some(
    (r) =>
      (r.stageStatus === "active" || r.stageStatus === "stopping") &&
      !isCancelledRun(r),
  );
  const anyQueued = working.some(
    (r) => r.stageStatus === "pending" && !isCancelledRun(r),
  );
  const onlyCancelled =
    working.length > 0 && working.every((r) => isCancelledRun(r));

  if (onlyCancelled) {
    return "Recently cancelled";
  }
  if (anyLive) {
    return working.length > 1 ? "Active runs" : "Active run";
  }
  if (anyQueued) {
    return working.length > 1 ? "Queued runs" : "Queued run";
  }
  return working.length > 1 ? "Active runs" : "Active run";
}
