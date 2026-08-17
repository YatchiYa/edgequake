/**
 * Delete session SSOT (SPEC-050 feedback-zone progress).
 *
 * WHY (SRP): Own in-flight delete entries + phase fields separately from
 * reprocess admit / PDF progress polling. Paint before network; update from WS.
 */

import type { QueryClient } from "@tanstack/react-query";
import type { Document } from "@/types";

export type DeletionSessionStatus = "active" | "completed" | "failed";

export interface DeletionSessionEntry {
  documentId: string;
  documentName: string;
  phase: string | null;
  phaseLabel: string;
  itemsProcessed: number;
  itemsTotal: number;
  status: DeletionSessionStatus;
  /** Durable deletion task id for poll fallback (SPEC-069). */
  trackId?: string | null;
  /** Wall clock when phase/counts last advanced (liveness). */
  phaseUpdatedAt: number;
  /** Wall clock when the session was opened. */
  startedAt: number;
  entitiesRemoved?: number;
  relationshipsRemoved?: number;
  chunksDeleted?: number;
  embeddingsDeleted?: number;
  error?: string | null;
  dismissed: boolean;
  /** Auto-dismiss timer id after success. */
  dismissTimer?: ReturnType<typeof setTimeout>;
}

/** Hex short id used as last-resort label — never preferred over a real name. */
export function isHexShortDocumentLabel(name: string): boolean {
  return /^[0-9a-f]{8}$/i.test(name.trim());
}

/**
 * Prefer title/file_name over hex id slices (SPEC-069).
 * Never downgrade an existing better documentName.
 */
export function preferDocumentName(
  current: string | undefined,
  next: string | undefined,
): string {
  const cur = (current ?? "").trim();
  const nxt = (next ?? "").trim();
  if (!nxt) return cur;
  if (!cur) return nxt;
  if (isHexShortDocumentLabel(cur) && !isHexShortDocumentLabel(nxt)) return nxt;
  if (!isHexShortDocumentLabel(cur) && isHexShortDocumentLabel(nxt)) return cur;
  return cur;
}

const sessions = new Map<string, DeletionSessionEntry>();
const listeners = new Set<() => void>();

<<<<<<< HEAD
=======
/**
 * SPEC-098 LAW-098-10: pin deleting ids so list polls cannot restore
 * Completed/Ready while a delete session is active.
 */
const deletingPins = new Set<string>();

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/** Cached snapshot for useSyncExternalStore — must be referentially stable. */
let cachedSnapshot: DeletionSessionEntry[] = [];

function rebuildSnapshot(): DeletionSessionEntry[] {
  cachedSnapshot = [...sessions.values()].filter((s) => !s.dismissed);
  return cachedSnapshot;
}

function notify(): void {
  rebuildSnapshot();
  for (const cb of listeners) cb();
}

export function subscribeDeleteSessions(cb: () => void): () => void {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
}

/**
 * Active (non-dismissed) delete sessions.
 * Returns a cached array so React's useSyncExternalStore does not loop.
 */
export function getDeleteSessions(): DeletionSessionEntry[] {
  return cachedSnapshot;
}

export function getDeleteSession(
  documentId: string,
): DeletionSessionEntry | undefined {
  return sessions.get(documentId);
}

<<<<<<< HEAD
=======
/** Active delete session ids (for table dimming — one SSOT). */
export function getActiveDeletingDocumentIds(): Set<string> {
  const ids = new Set<string>();
  for (const entry of sessions.values()) {
    if (!entry.dismissed && entry.status === "active") {
      ids.add(entry.documentId);
    }
  }
  for (const id of deletingPins) {
    ids.add(id);
  }
  return ids;
}

export function pinDeletingDocuments(
  documentIds: string | Iterable<string>,
): void {
  const ids =
    typeof documentIds === "string"
      ? [documentIds]
      : Array.from(documentIds);
  for (const id of ids) {
    deletingPins.add(id);
  }
}

export function unpinDeletingDocuments(
  documentIds: string | Iterable<string>,
): void {
  const ids =
    typeof documentIds === "string"
      ? [documentIds]
      : Array.from(documentIds);
  for (const id of ids) {
    deletingPins.delete(id);
  }
}

export function isDeletingPinned(documentId: string): boolean {
  return deletingPins.has(documentId);
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/** Optimistic fields so the table badge shows Deleting immediately. */
export const DELETE_OPTIMISTIC_FIELDS = {
  status: "deleting",
  current_stage: "deleting",
  stage_message: "Removing document data…",
  stage_progress: 0,
} as const;

type DocumentsQueryData = { items?: Document[] } | undefined;

<<<<<<< HEAD
=======
/**
 * Re-apply deleting fields when a poll returns terminal success for a pinned id.
 * If the poll omitted the row (already gone), leave it omitted.
 */
export function protectDeletingDocumentsInQueryData<T extends DocumentsQueryData>(
  data: T,
): T {
  if (!data?.items || deletingPins.size === 0) return data;
  let changed = false;
  const items = data.items.map((doc) => {
    if (!deletingPins.has(doc.id)) return doc;
    const status = (doc.status || "").toLowerCase();
    if (status === "deleting" || status === "delete_failed") return doc;
    // Stale Completed/Ready (or other terminal success) must not win mid-delete.
    if (
      status === "completed" ||
      status === "indexed" ||
      status === "partial_success" ||
      status === ""
    ) {
      changed = true;
      return {
        ...doc,
        ...DELETE_OPTIMISTIC_FIELDS,
      };
    }
    return doc;
  });
  return changed ? { ...data, items } : data;
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
export function patchDocumentsDeletingOptimistic(
  queryClient: QueryClient,
  documentIds: string | Iterable<string>,
): void {
  const ids =
    typeof documentIds === "string"
      ? new Set([documentIds])
      : new Set(documentIds);

  queryClient.setQueriesData(
    { queryKey: ["documents"] },
    (oldData: DocumentsQueryData) => {
      if (!oldData?.items) return oldData;
      return {
        ...oldData,
        items: oldData.items.map((doc) =>
          ids.has(doc.id)
            ? {
                ...doc,
                ...DELETE_OPTIMISTIC_FIELDS,
              }
            : doc,
        ),
      };
    },
  );
}

/**
 * Paint-first: open a feedback-zone delete row before DELETE HTTP returns.
 * SPEC-069: must not downgrade an existing better `documentName` (hex overwrite).
 */
export function beginDeleteSession(input: {
  documentId: string;
  documentName: string;
  trackId?: string | null;
}): DeletionSessionEntry {
  const existing = sessions.get(input.documentId);
  if (existing?.dismissTimer) clearTimeout(existing.dismissTimer);

<<<<<<< HEAD
=======
  pinDeletingDocuments(input.documentId);

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  const now = Date.now();
  if (existing && !existing.dismissed && existing.status === "active") {
    const documentName = preferDocumentName(
      existing.documentName,
      input.documentName,
    );
    const entry: DeletionSessionEntry = {
      ...existing,
      documentName,
      trackId: input.trackId ?? existing.trackId,
      dismissed: false,
    };
    sessions.set(input.documentId, entry);
    notify();
    return entry;
  }

  const entry: DeletionSessionEntry = {
    documentId: input.documentId,
    documentName: preferDocumentName(undefined, input.documentName),
    phase: null,
    phaseLabel: "Removing document data…",
    itemsProcessed: 0,
    itemsTotal: 0,
    status: "active",
    trackId: input.trackId ?? null,
    phaseUpdatedAt: now,
    startedAt: now,
    dismissed: false,
  };
  sessions.set(input.documentId, entry);
  notify();
  return entry;
}

/** Bind deletion track id after HTTP 202 admit (poll fallback). */
export function bindDeleteSessionTrackId(
  documentId: string,
  trackId: string | null | undefined,
): void {
  if (!trackId) return;
  const entry = sessions.get(documentId);
  if (!entry || entry.dismissed) return;
  sessions.set(documentId, { ...entry, trackId });
  notify();
}

/** Hide panel only — deletion continues server-side. */
export function dismissDeleteSession(documentId: string): void {
  const entry = sessions.get(documentId);
<<<<<<< HEAD
  if (!entry) return;
  if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
=======
  if (!entry) {
    unpinDeletingDocuments(documentId);
    return;
  }
  if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
  // Abort path (bulk HTTP failed before admit): release pin with dismiss.
  if (entry.status === "active" && !entry.trackId) {
    unpinDeletingDocuments(documentId);
  }
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  sessions.set(documentId, { ...entry, dismissed: true });
  notify();
  // Drop after a tick so listeners see dismissed=true once
  setTimeout(() => {
    sessions.delete(documentId);
    notify();
  }, 0);
}

export function applyDeletionStarted(documentId: string): void {
  const entry = sessions.get(documentId);
  if (!entry || entry.dismissed) return;
  sessions.set(documentId, {
    ...entry,
    status: "active",
    phaseLabel: entry.phaseLabel || "Removing document data…",
  });
  notify();
}

export function applyDeletionPhase(input: {
  documentId: string;
  phase: string;
  phaseLabel: string;
  itemsProcessed: number;
  itemsTotal: number;
}): void {
  const entry = sessions.get(input.documentId);
  if (!entry || entry.dismissed) return;
  const advanced =
    input.phase !== entry.phase ||
    input.itemsProcessed !== entry.itemsProcessed ||
    input.itemsTotal !== entry.itemsTotal;
  sessions.set(input.documentId, {
    ...entry,
    status: "active",
    phase: input.phase,
    phaseLabel: input.phaseLabel || entry.phaseLabel,
    itemsProcessed: input.itemsProcessed,
    itemsTotal: input.itemsTotal,
    phaseUpdatedAt: advanced ? Date.now() : entry.phaseUpdatedAt,
  });
  notify();
}

const AUTO_DISMISS_MS = 2500;

export function applyDeletionCompleted(input: {
  documentId: string;
  chunksDeleted: number;
  entitiesRemoved: number;
  relationshipsRemoved: number;
  embeddingsDeleted: number;
  partialFailure: boolean;
  error: string | null;
}): void {
  const entry = sessions.get(input.documentId);
  if (!entry || entry.dismissed) return;

  if (entry.dismissTimer) clearTimeout(entry.dismissTimer);

  const failed = input.partialFailure && Boolean(input.error);
  const next: DeletionSessionEntry = {
    ...entry,
    status: failed ? "failed" : "completed",
    phase: "completed",
    phaseLabel: failed
      ? input.error || "Deletion failed"
      : formatDeleteSuccessDetail(input),
    entitiesRemoved: input.entitiesRemoved,
    relationshipsRemoved: input.relationshipsRemoved,
    chunksDeleted: input.chunksDeleted,
    embeddingsDeleted: input.embeddingsDeleted,
    error: input.error,
  };

<<<<<<< HEAD
=======
  unpinDeletingDocuments(input.documentId);

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  if (!failed) {
    next.dismissTimer = setTimeout(() => {
      sessions.delete(input.documentId);
      notify();
    }, AUTO_DISMISS_MS);
  }

  sessions.set(input.documentId, next);
  notify();
}

export function applyDeletionFailed(
  documentId: string,
  error: string,
): void {
  const entry = sessions.get(documentId);
  if (!entry || entry.dismissed) return;
  if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
<<<<<<< HEAD
=======
  unpinDeletingDocuments(documentId);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  sessions.set(documentId, {
    ...entry,
    status: "failed",
    phase: "failed",
    phaseLabel: error,
    error,
  });
  notify();
}

export function formatDeleteSuccessDetail(input: {
  entitiesRemoved: number;
  relationshipsRemoved: number;
  chunksDeleted?: number;
}): string {
  const parts: string[] = [];
  if (input.entitiesRemoved > 0) {
    parts.push(`${input.entitiesRemoved} entities`);
  }
  if (input.relationshipsRemoved > 0) {
    parts.push(`${input.relationshipsRemoved} relationships`);
  }
  if (input.chunksDeleted && input.chunksDeleted > 0) {
    parts.push(`${input.chunksDeleted} chunks`);
  }
  if (parts.length === 0) return "Document removed";
  return `Removed ${parts.join(", ")}`;
}

export function formatDeleteCountsLabel(
  entry: DeletionSessionEntry,
): string | null {
  if (entry.itemsTotal > 0) {
    return `${entry.itemsProcessed}/${entry.itemsTotal}`;
  }
  return null;
}

/**
 * SPEC-069: long graph phase liveness when counts stay empty / unchanged.
 * Pass `now` so React can re-render on a tick without mutating the store.
 */
export function formatDeleteLivenessLabel(
  entry: DeletionSessionEntry,
  now: number = Date.now(),
): string | null {
  if (entry.status !== "active") return null;
  const phase = (entry.phase ?? "").toLowerCase();
  const inGraph =
    phase === "removing_graph" ||
    entry.phaseLabel.toLowerCase().includes("graph");
  if (!inGraph) return null;
  const silentMs = now - (entry.phaseUpdatedAt || entry.startedAt);
  if (silentMs < 4000) return null;
  const elapsedSec = Math.max(1, Math.floor((now - entry.startedAt) / 1000));
  if (entry.itemsTotal > 0) {
    return `Still working… ${elapsedSec}s`;
  }
  return `Still working on graph… ${elapsedSec}s`;
}

/** Compose stage message with optional liveness suffix. */
export function formatDeleteStageMessage(
  entry: DeletionSessionEntry,
  now: number = Date.now(),
): string {
  const live = formatDeleteLivenessLabel(entry, now);
  if (!live) return entry.phaseLabel;
  if (entry.phaseLabel.includes("Still working")) return entry.phaseLabel;
  return `${entry.phaseLabel} — ${live}`;
}

<<<<<<< HEAD
=======
/** SPEC-098 LAW-098-11: feedback header must not say “Deleting N” when all failed. */
export function formatDeleteProgressHeader(sessions: {
  status: DeletionSessionStatus;
}[]): {
  text: string;
  pulse: boolean;
  activeCount: number;
  failedCount: number;
} {
  const activeCount = sessions.filter((s) => s.status === "active").length;
  const failedCount = sessions.filter((s) => s.status === "failed").length;
  const total = sessions.length;
  if (total === 0) {
    return { text: "", pulse: false, activeCount: 0, failedCount: 0 };
  }
  if (activeCount === 0 && failedCount > 0) {
    return {
      text: `Delete failed (${failedCount})`,
      pulse: false,
      activeCount,
      failedCount,
    };
  }
  if (activeCount > 0 && failedCount > 0) {
    return {
      text: `Deleting ${activeCount} · failed ${failedCount}`,
      pulse: true,
      activeCount,
      failedCount,
    };
  }
  return {
    text: `Deleting ${activeCount || total} document(s)`,
    pulse: activeCount > 0,
    activeCount,
    failedCount,
  };
}

>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
/** Test helper. */
export function clearDeleteSessionsForTests(): void {
  for (const entry of sessions.values()) {
    if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
  }
  sessions.clear();
<<<<<<< HEAD
=======
  deletingPins.clear();
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
  notify();
}
