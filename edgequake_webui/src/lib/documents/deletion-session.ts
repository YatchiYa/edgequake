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
  entitiesRemoved?: number;
  relationshipsRemoved?: number;
  chunksDeleted?: number;
  embeddingsDeleted?: number;
  error?: string | null;
  dismissed: boolean;
  /** Auto-dismiss timer id after success. */
  dismissTimer?: ReturnType<typeof setTimeout>;
}

const sessions = new Map<string, DeletionSessionEntry>();
const listeners = new Set<() => void>();

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

/** Optimistic fields so the table badge shows Deleting immediately. */
export const DELETE_OPTIMISTIC_FIELDS = {
  status: "deleting",
  current_stage: "deleting",
  stage_message: "Removing document data…",
  stage_progress: 0,
} as const;

type DocumentsQueryData = { items?: Document[] } | undefined;

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
 */
export function beginDeleteSession(input: {
  documentId: string;
  documentName: string;
}): DeletionSessionEntry {
  const existing = sessions.get(input.documentId);
  if (existing?.dismissTimer) clearTimeout(existing.dismissTimer);

  const entry: DeletionSessionEntry = {
    documentId: input.documentId,
    documentName: input.documentName,
    phase: null,
    phaseLabel: "Removing document data…",
    itemsProcessed: 0,
    itemsTotal: 0,
    status: "active",
    dismissed: false,
  };
  sessions.set(input.documentId, entry);
  notify();
  return entry;
}

/** Hide panel only — deletion continues server-side. */
export function dismissDeleteSession(documentId: string): void {
  const entry = sessions.get(documentId);
  if (!entry) return;
  if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
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
  sessions.set(input.documentId, {
    ...entry,
    status: "active",
    phase: input.phase,
    phaseLabel: input.phaseLabel || entry.phaseLabel,
    itemsProcessed: input.itemsProcessed,
    itemsTotal: input.itemsTotal,
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

/** Test helper. */
export function clearDeleteSessionsForTests(): void {
  for (const entry of sessions.values()) {
    if (entry.dismissTimer) clearTimeout(entry.dismissTimer);
  }
  sessions.clear();
  notify();
}
