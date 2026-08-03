/**
 * Cancelled Active Runs lifecycle SSOT.
 *
 * Active Runs = live work. Terminal cancelled gets a brief acknowledgement
 * then auto-dismisses (or manual Dismiss). Session memory prevents reappearance
 * on poll/refresh for the same document+track.
 *
 * Aged cancels (updatedAt older than grace) never enter the panel.
 */

export const CANCELLED_RUN_GRACE_MS = 8_000;

export type CancelledRunKey = string;

interface CancelledRunEntry {
  observedAt: number;
  dismissed: boolean;
  /** When true, grace timer is paused (hover/focus). */
  pauseStartedAt: number | null;
  accumulatedPauseMs: number;
}

const entries = new Map<CancelledRunKey, CancelledRunEntry>();

/** Stable key for document + optional track. */
export function cancelledRunKey(
  documentId: string,
  trackId?: string | null,
): CancelledRunKey {
  return `${documentId}::${trackId ?? ""}`;
}

function parseCancelledAt(
  cancelledAt: string | number | null | undefined,
): number | null {
  if (cancelledAt == null) return null;
  if (typeof cancelledAt === "number" && Number.isFinite(cancelledAt)) {
    return cancelledAt;
  }
  const t = Date.parse(String(cancelledAt));
  return Number.isNaN(t) ? null : t;
}

export type ObserveCancelledOpts = {
  /** When the cancel became terminal (doc.updated_at). Ages out immediately. */
  cancelledAt?: string | number | null;
};

/**
 * First observation pins the grace window; subsequent calls are no-ops.
 * If cancelledAt is already older than grace, mark dismissed (no flash).
 */
export function observeCancelledRun(
  key: CancelledRunKey,
  nowMs: number = Date.now(),
  opts: ObserveCancelledOpts = {},
): void {
  if (!key || entries.has(key)) return;

  const cancelledAt = parseCancelledAt(opts.cancelledAt);
  if (cancelledAt != null && nowMs - cancelledAt >= CANCELLED_RUN_GRACE_MS) {
    entries.set(key, {
      observedAt: cancelledAt,
      dismissed: true,
      pauseStartedAt: null,
      accumulatedPauseMs: 0,
    });
    return;
  }

  entries.set(key, {
    observedAt: cancelledAt ?? nowMs,
    dismissed: false,
    pauseStartedAt: null,
    accumulatedPauseMs: 0,
  });
}

export function dismissCancelledRun(key: CancelledRunKey): void {
  const existing = entries.get(key);
  if (existing) {
    existing.dismissed = true;
    existing.pauseStartedAt = null;
    return;
  }
  entries.set(key, {
    observedAt: Date.now(),
    dismissed: true,
    pauseStartedAt: null,
    accumulatedPauseMs: 0,
  });
}

export function pauseCancelledRunGrace(
  key: CancelledRunKey,
  nowMs: number = Date.now(),
): void {
  const entry = entries.get(key);
  if (!entry || entry.dismissed || entry.pauseStartedAt != null) return;
  entry.pauseStartedAt = nowMs;
}

export function resumeCancelledRunGrace(
  key: CancelledRunKey,
  nowMs: number = Date.now(),
): void {
  const entry = entries.get(key);
  if (!entry || entry.pauseStartedAt == null) return;
  entry.accumulatedPauseMs += Math.max(0, nowMs - entry.pauseStartedAt);
  entry.pauseStartedAt = null;
}

function effectiveElapsedMs(entry: CancelledRunEntry, nowMs: number): number {
  let pause = entry.accumulatedPauseMs;
  if (entry.pauseStartedAt != null) {
    pause += Math.max(0, nowMs - entry.pauseStartedAt);
  }
  return Math.max(0, nowMs - entry.observedAt - pause);
}

/**
 * True while the cancelled ack should stay in Active Runs.
 * Call observeCancelledRun first (or this will observe on first check).
 */
export function isCancelledRunVisible(
  key: CancelledRunKey,
  nowMs: number = Date.now(),
  graceMs: number = CANCELLED_RUN_GRACE_MS,
  opts: ObserveCancelledOpts = {},
): boolean {
  observeCancelledRun(key, nowMs, opts);
  const entry = entries.get(key);
  if (!entry || entry.dismissed) return false;
  return effectiveElapsedMs(entry, nowMs) < graceMs;
}

/** Remaining ms until auto-dismiss; 0 when hidden/dismissed. */
export function cancelledRunRemainingMs(
  key: CancelledRunKey,
  nowMs: number = Date.now(),
  graceMs: number = CANCELLED_RUN_GRACE_MS,
): number {
  const entry = entries.get(key);
  if (!entry || entry.dismissed) return 0;
  return Math.max(0, graceMs - effectiveElapsedMs(entry, nowMs));
}

/** Test helper — clear session memory. */
export function resetCancelledRunLifecycleForTests(): void {
  entries.clear();
}
