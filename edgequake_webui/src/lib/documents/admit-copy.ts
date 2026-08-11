/**
 * SPEC-122 LAW-122-1 — Admit ≠ searchable copy SSOT (pure, no React).
 *
 * Toast / banner / upload-list headers consume these builders so we never
 * invent a second matrix of “upload done = ready” strings.
 *
 * @implements SPEC-122 Phase A P0
 */

export type BulkIngestBannerInput = {
  pending: number;
  processing: number;
  completed: number;
  /** From queue-metrics `max_tasks_per_tenant`; omit when unknown. */
  maxTasksPerTenant?: number | null;
};

/** True when the Documents bulk banner should render. */
export function shouldShowBulkBanner(pending: number, processing: number): boolean {
  return pending + processing > 0;
}

/**
 * Concurrency lane honesty (LAW-122-2 / UX spec).
 * Returns null when the lane size is unknown so callers can omit the clause.
 */
export function concurrencyLaneHint(
  maxTasksPerTenant?: number | null,
): string | null {
  if (
    maxTasksPerTenant == null ||
    !Number.isFinite(maxTasksPerTenant) ||
    maxTasksPerTenant < 1
  ) {
    return null;
  }
  if (maxTasksPerTenant === 1) {
    return "Processing one document at a time";
  }
  return `Processing up to ${maxTasksPerTenant} documents in parallel`;
}

/**
 * One-line bulk ingest physics for the Documents banner.
 * Empty when nothing is pending/processing (EC-T5 / U2).
 */
export function bulkIngestBannerLine(
  input: BulkIngestBannerInput,
): string | null {
  const pending = Math.max(0, input.pending | 0);
  const processing = Math.max(0, input.processing | 0);
  const completed = Math.max(0, input.completed | 0);
  if (!shouldShowBulkBanner(pending, processing)) {
    return null;
  }
  const counts = `Processing ${processing} · ${pending} queued · ${completed} completed`;
  const hint = concurrencyLaneHint(input.maxTasksPerTenant);
  return hint ? `${counts} — ${hint}` : counts;
}

/** Admit success toast / fallback (never implies searchable). */
export function admitSuccessMessage(count: number): string {
  const n = Math.max(0, count | 0);
  return `${n} file(s) admitted; processing queued`;
}

/** Partial admit toast — successes are queued, not ready. */
export function admitPartialMessage(success: number, failed: number): string {
  const ok = Math.max(0, success | 0);
  const bad = Math.max(0, failed | 0);
  return `Admitted ${ok} file(s); ${bad} failed — processing queued for successful uploads`;
}

/** Upload-progress list header after transfer finishes (LAW-122-1). */
export function transferCompleteHeader(): string {
  return "Transfer complete — processing queued";
}

/**
 * Forbidden substrings for admit-success copy (unit gate).
 * Also blocks “not available for query / View in Graph” pollution on admit.
 */
export const ADMIT_FORBIDDEN_CLAIM_RE =
  /\b(ready|searchable|available for query|not available|view in graph)\b/i;
