/**
 * Dependency-free bounded scheduler for browser file admission.
 *
 * Transfer concurrency is intentionally separate from worker/task fairness.
 * Three requests overlap network/admission latency without consuming every
 * browser connection or flooding the backend.
 *
 * SPEC-132 LAW-132-3: each `run` always releases its slot in `finally`, so one
 * hung/failed admit cannot freeze siblings forever once the task settles
 * (XHR timeout / network error).
 */
export const MAX_CONCURRENT_FILE_UPLOADS = 3;

/** Stable-enough identity for suppressing the same file while it is in flight. */
export function fileUploadFingerprint(file: File): string {
  return `${file.name}\u0000${file.size}\u0000${file.lastModified}`;
}

/** Client-only row identity. Server progress continues to use task track_id. */
export function createUploadId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `upload-file-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

/** Immutable row update that remains correct when concurrent batches reorder. */
export function updateByUploadId<T extends { uploadId: string }>(
  entries: readonly T[],
  uploadId: string,
  update: Partial<T> | ((current: T) => Partial<T>),
): T[] {
  return entries.map((entry) => {
    if (entry.uploadId !== uploadId) return entry;
    const patch = typeof update === "function" ? update(entry) : update;
    return { ...entry, ...patch };
  });
}

export interface BoundedExecutor {
  run<R>(task: () => Promise<R>): Promise<R>;
}

/** Queue tasks behind one shared concurrency cap, including later batches. */
export function createBoundedExecutor(concurrency: number): BoundedExecutor {
  const limit = Math.max(1, Math.floor(concurrency));
  const waiting: Array<() => void> = [];
  let active = 0;

  const release = () => {
    active -= 1;
    waiting.shift()?.();
  };

  return {
    async run<R>(task: () => Promise<R>): Promise<R> {
      if (active >= limit) {
        await new Promise<void>((resolve) => waiting.push(resolve));
      }
      active += 1;
      try {
        return await task();
      } finally {
        release();
      }
    },
  };
}

/**
 * Map every item with at most `concurrency` workers.
 *
 * Results preserve input order. Rejections are returned by `Promise.all`;
 * callers that require all-settled behavior should catch inside `worker`, which
 * keeps domain-specific error reporting out of this generic helper.
 */
export async function mapWithConcurrency<T, R>(
  items: readonly T[],
  concurrency: number,
  worker: (item: T, index: number) => Promise<R>,
): Promise<R[]> {
  if (items.length === 0) return [];

  const executor = createBoundedExecutor(concurrency);
  return Promise.all(
    items.map((item, index) => executor.run(() => worker(item, index))),
  );
}

/** SPEC-132: classify admit/transfer failures for per-file UI copy. */
export function isUploadTimeoutMessage(message: string): boolean {
  return /timed out/i.test(message);
}

/** Honest per-file failure copy (LAW-132-3) — siblings keep running. */
export function perFileUploadErrorMessage(
  error: unknown,
  fallback = "Upload failed",
): string {
  const raw =
    error instanceof Error && error.message.trim().length > 0
      ? error.message
      : fallback;
  if (isUploadTimeoutMessage(raw)) {
    return `${raw}. This file failed; other selected files continue.`;
  }
  return raw;
}
