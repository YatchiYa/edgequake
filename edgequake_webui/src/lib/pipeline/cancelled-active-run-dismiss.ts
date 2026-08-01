/**
 * Durable dismiss + freeze cache for Cancelled Active Runs.
 *
 * Prefer sessionStorage (survives refresh in the tab). Fall back to process
 * memory when Storage is unavailable (SSR / Vitest).
 */

const DISMISS_KEY = "edgequake:ar-dismissed-cancelled";
const FREEZE_KEY = "edgequake:ar-cancelled-from-stage";

const memoryStore = new Map<string, string>();

function storageGet(key: string): string | null {
  if (typeof sessionStorage !== "undefined") {
    try {
      return sessionStorage.getItem(key);
    } catch {
      /* private mode */
    }
  }
  return memoryStore.get(key) ?? null;
}

function storageSet(key: string, value: string): void {
  memoryStore.set(key, value);
  if (typeof sessionStorage !== "undefined") {
    try {
      sessionStorage.setItem(key, value);
    } catch {
      /* quota / private mode — memory still holds it for this session */
    }
  }
}

function storageRemove(key: string): void {
  memoryStore.delete(key);
  if (typeof sessionStorage !== "undefined") {
    try {
      sessionStorage.removeItem(key);
    } catch {
      /* ignore */
    }
  }
}

/** Test helper: clear both memory and sessionStorage. */
export function clearCancelledActiveRunDismissStorage(): void {
  memoryStore.clear();
  storageRemove(DISMISS_KEY);
  storageRemove(FREEZE_KEY);
}

function readJsonArray(key: string): string[] {
  try {
    const raw = storageGet(key);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((v): v is string => typeof v === "string");
  } catch {
    return [];
  }
}

function writeJsonArray(key: string, values: string[]): void {
  storageSet(key, JSON.stringify([...new Set(values)]));
}

function readJsonRecord(key: string): Record<string, string> {
  try {
    const raw = storageGet(key);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }
    const out: Record<string, string> = {};
    for (const [k, v] of Object.entries(parsed as Record<string, unknown>)) {
      if (typeof v === "string" && v.trim()) out[k] = v;
    }
    return out;
  } catch {
    return {};
  }
}

function writeJsonRecord(key: string, values: Record<string, string>): void {
  storageSet(key, JSON.stringify(values));
}

/** Load dismissed cancelled document IDs (survives refresh). */
export function loadDismissedCancelledIds(): Set<string> {
  return new Set(readJsonArray(DISMISS_KEY));
}

export function persistDismissedCancelledId(documentId: string): Set<string> {
  const next = loadDismissedCancelledIds();
  next.add(documentId);
  writeJsonArray(DISMISS_KEY, [...next]);
  return next;
}

/** Drop dismiss entries for docs that are no longer cancelled. */
export function pruneDismissedCancelledIds(
  stillCancelledIds: ReadonlySet<string>,
): Set<string> {
  const prev = loadDismissedCancelledIds();
  const next = [...prev].filter((id) => stillCancelledIds.has(id));
  if (next.length !== prev.size) {
    writeJsonArray(DISMISS_KEY, next);
  }
  return new Set(next);
}

/**
 * Cache last known freeze stage (Stopping / cancel) so refresh stays honest
 * even before API cancelled_from_stage is present.
 */
export function rememberCancelledFromStage(
  documentId: string,
  stage: string,
): void {
  const key = stage.toLowerCase().trim();
  if (
    !key ||
    key === "cancelled" ||
    key === "stopping" ||
    key === "failed" ||
    key === "completed"
  ) {
    return;
  }
  const map = readJsonRecord(FREEZE_KEY);
  map[documentId] = key;
  writeJsonRecord(FREEZE_KEY, map);
}

export function loadCancelledFromStage(
  documentId: string,
): string | undefined {
  return readJsonRecord(FREEZE_KEY)[documentId];
}

export function clearCancelledFromStage(documentId: string): void {
  const map = readJsonRecord(FREEZE_KEY);
  if (!(documentId in map)) return;
  delete map[documentId];
  writeJsonRecord(FREEZE_KEY, map);
}

/** Parse ISO updated_at for durable TTL (survives refresh). */
export function parseRunUpdatedAtMs(
  updatedAt: string | undefined | null,
): number | undefined {
  if (!updatedAt) return undefined;
  const ms = Date.parse(updatedAt);
  return Number.isFinite(ms) ? ms : undefined;
}
