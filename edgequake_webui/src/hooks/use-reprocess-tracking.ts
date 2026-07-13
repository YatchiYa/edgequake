/**
 * @module useReprocessTracking
 * @description Tracks in-progress reprocess operations so DocumentManager can
 * show IngestionProgressPanel per reprocessed document — identical feedback to a
 * fresh upload (SPEC-050-REPROCESS).
 *
 * WHY (First Principles):
 *   Fresh upload → UploadingFile state → IngestionProgressPanel (stages, cost, ETA, cancel)
 *   Reprocess     → No UploadingFile entry → only ActiveRunsPanel (compact stepper)
 *
 *   The gap: after reprocess confirm, the user has NO dedicated stage panel.
 *   Fix: maintain a lightweight Map of { documentName, trackId } entries and expose
 *   them for IngestionProgressPanel rendering. When the document reaches a terminal
 *   state the entry is automatically pruned.
 *
 * Design (SRP / DIP):
 *   - This hook owns ONLY the state for active reprocess progress entries.
 *   - It does NOT own rendering (IngestionProgressPanel is rendered by the caller).
 *   - It does NOT own the reprocess mutation (useDocumentMutations fires the callback).
 *   - Cleanup is driven by the documents list from the existing useDocumentQueries.
 *
 * @implements SPEC-050-REPROCESS: Reprocess feedback parity with fresh upload.
 */
'use client';

import {
    getDocumentDisplayStatus,
    isTerminalStatus,
} from '@/components/documents/status-badge';
import type { Document } from '@/types';
import { useCallback, useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * A single in-progress reprocess entry.
 */
export interface ReprocessEntry {
  /** Human-readable document name shown in IngestionProgressPanel. */
  documentName: string;
  /** The new task tracking ID returned by POST /documents/reprocess. */
  trackId: string;
}

/**
 * Return type for useReprocessTracking.
 */
export interface UseReprocessTrackingReturn {
  /**
   * All currently-active reprocess entries (unordered).
   * Render IngestionProgressPanel for each one.
   */
  reprocessEntries: ReprocessEntry[];

  /**
   * Add a new reprocess entry.
   * Called from the onReprocessTriggered callback in useDocumentMutations.
   * Idempotent: duplicate trackIds are de-duplicated.
   */
  addReprocessEntry: (documentName: string, trackId: string) => void;

  /**
   * Explicitly remove a single entry (e.g. on IngestionProgressPanel.onComplete).
   */
  removeReprocessEntry: (trackId: string) => void;

  /**
   * Prune entries whose backing document has reached a terminal state.
   * Call this in the same useEffect that drives pruneTerminalUploads.
   *
   * WHY: IngestionProgressPanel's onComplete fires reliably for the "completed"
   * state, but for error/cancelled paths the panel may not fire onFailed.
   * Pruning from the documents list is the safety net.
   */
  pruneTerminalReprocessEntries: (docs: Document[]) => void;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Tracks in-progress reprocess operations for IngestionProgressPanel display.
 *
 * Usage:
 * ```tsx
 * const { reprocessEntries, addReprocessEntry, removeReprocessEntry, pruneTerminalReprocessEntries }
 *   = useReprocessTracking();
 *
 * // Wire addReprocessEntry into useDocumentMutations:
 * useDocumentMutations({ onReprocessTriggered: addReprocessEntry });
 *
 * // Prune when documents update:
 * useEffect(() => pruneTerminalReprocessEntries(documents), [documents]);
 *
 * // Render:
 * {reprocessEntries.map(e => (
 *   <IngestionProgressPanel
 *     key={e.trackId}
 *     trackId={e.trackId}
 *     documentName={e.documentName}
 *     compact
 *     onComplete={() => removeReprocessEntry(e.trackId)}
 *     onFailed={() => removeReprocessEntry(e.trackId)}
 *   />
 * ))}
 * ```
 */
export function useReprocessTracking(): UseReprocessTrackingReturn {
  // Use a Map internally for O(1) de-dup and removal; expose as array for rendering.
  const [entries, setEntries] = useState<Map<string, ReprocessEntry>>(
    () => new Map(),
  );

  const addReprocessEntry = useCallback(
    (documentName: string, trackId: string) => {
      setEntries((prev) => {
        // Idempotent: don't add if already tracking this trackId.
        if (prev.has(trackId)) return prev;
        const next = new Map(prev);
        next.set(trackId, { documentName, trackId });
        return next;
      });
    },
    [],
  );

  const removeReprocessEntry = useCallback((trackId: string) => {
    // SPEC-050-REPROCESS: Keep completed/failed panels visible for 3 seconds so
    // the user can see the final state (same pattern as upload progress panels).
    // WHY: Without this delay, the panel disappears instantly when processing
    // completes quickly (< 1s on fast hardware), giving the user no feedback.
    setTimeout(() => {
      setEntries((prev) => {
        if (!prev.has(trackId)) return prev;
        const next = new Map(prev);
        next.delete(trackId);
        return next;
      });
    }, 3000);
  }, []);

  const pruneTerminalReprocessEntries = useCallback((docs: Document[]) => {
    if (!docs.length) return;
    // Collect terminal trackIds without mutating state inside the updater,
    // then schedule deferred removal for each. This avoids calling setEntries
    // inside a setEntries updater (which is prohibited in React).
    setEntries((prev) => {
      if (prev.size === 0) return prev;
      for (const [trackId] of prev) {
        const match = docs.find((d) => d.track_id === trackId);
        if (!match) continue;
        const displayStatus = getDocumentDisplayStatus(match);
        if (isTerminalStatus(displayStatus)) {
          // Defer removal to maintain the 3-second visibility window.
          // Use a closure-captured ref so this is safe even if the component
          // re-renders between now and the timeout firing.
          const captured = trackId;
          setTimeout(() => {
            setEntries((p) => {
              if (!p.has(captured)) return p;
              const n = new Map(p);
              n.delete(captured);
              return n;
            });
          }, 3000);
        }
      }
      return prev; // Synchronous state is unchanged; setTimeout handles removal.
    });
  }, []);

  return {
    reprocessEntries: [...entries.values()],
    addReprocessEntry,
    removeReprocessEntry,
    pruneTerminalReprocessEntries,
  };
}
