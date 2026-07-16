/**
 * @module useDeletionProgress
 * @description Subscribes to single-doc Deletion* WebSocket events and mirrors
 * them into the delete-session SSOT for the feedback zone.
 *
 * @implements SPEC-050: Delete progress parity with ingestion.
 */

'use client';

import {
  applyDeletionCompleted,
  applyDeletionFailed,
  applyDeletionPhase,
  applyDeletionStarted,
  getDeleteSessions,
  subscribeDeleteSessions,
  type DeletionSessionEntry,
} from '@/lib/documents/deletion-session';
import { getWebSocketClient } from '@/lib/websocket';
import type {
  DeletionCompletedEvent,
  DeletionPhaseEvent,
  DeletionStartedEvent,
  WebSocketProgressMessage,
} from '@/types/ingestion';
import { useEffect, useSyncExternalStore } from 'react';

function subscribe(cb: () => void): () => void {
  return subscribeDeleteSessions(cb);
}

function getSnapshot(): DeletionSessionEntry[] {
  return getDeleteSessions();
}

/**
 * Reactive list of in-flight / completing delete sessions for the feedback zone.
 * Also attaches the global WS listener while any consumer is mounted.
 */
export function useDeletionSessions(): DeletionSessionEntry[] {
  const sessions = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);

  useEffect(() => {
    const client = getWebSocketClient();

    const handleMessage = (message: WebSocketProgressMessage) => {
      if (message.type === 'DeletionStarted') {
        const ev = message as DeletionStartedEvent;
        applyDeletionStarted(ev.data.document_id);
      } else if (message.type === 'DeletionPhase') {
        const ev = message as DeletionPhaseEvent;
        applyDeletionPhase({
          documentId: ev.data.document_id,
          phase: ev.data.phase,
          phaseLabel: ev.data.phase_label,
          itemsProcessed: ev.data.items_processed,
          itemsTotal: ev.data.items_total,
        });
      } else if (message.type === 'DeletionCompleted') {
        const ev = message as DeletionCompletedEvent;
        applyDeletionCompleted({
          documentId: ev.data.document_id,
          chunksDeleted: ev.data.chunks_deleted,
          entitiesRemoved: ev.data.entities_removed,
          relationshipsRemoved: ev.data.relationships_removed,
          embeddingsDeleted: ev.data.embeddings_deleted,
          partialFailure: ev.data.partial_failure,
          error: ev.data.error,
        });
      }
    };

    client.on('progress', handleMessage as (...args: unknown[]) => void);

    return () => {
      client.off('progress', handleMessage as (...args: unknown[]) => void);
    };
  }, []);

  return sessions;
}

/** Mark a session failed when HTTP DELETE rejects (WS may not have completed). */
export function markDeleteSessionFailed(
  documentId: string,
  error: string,
): void {
  applyDeletionFailed(documentId, error);
}
