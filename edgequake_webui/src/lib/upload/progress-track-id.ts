/**
 * Progress identity (SPEC-054 / GitHub #300).
 *
 * Server `task_id` is the sole progress / cancel / retry key.
 * Optional client `track_id` is batch correlation only (upload batch or
 * reprocess_* batch).
 */

export interface ProgressIdentityFields {
  task_id?: string | null;
  track_id?: string | null;
}

/**
 * Resolve the WebUI progress subscription key for upload or reprocess.
 *
 * Prefer `task_id` (v0.17.0+ SSOT). Fall back to `track_id` for older servers
 * that may omit `task_id`.
 */
export function resolveProgressTrackId(
  response: ProgressIdentityFields,
): string | undefined {
  const taskId = response.task_id?.trim();
  if (taskId) return taskId;
  const trackId = response.track_id?.trim();
  return trackId || undefined;
}

/** @deprecated Prefer {@link resolveProgressTrackId} — same SSOT for upload + reprocess. */
export function resolvePdfProgressTrackId(
  response: ProgressIdentityFields,
): string | undefined {
  return resolveProgressTrackId(response);
}
