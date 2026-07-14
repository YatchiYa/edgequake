/**
 * @module useBulkDeletionProgress
 * @description Listens for BulkDeletion* WebSocket events and tracks per-document progress.
 *
 * WHY: The bulk delete endpoint is a single HTTP call but the server broadcasts
 * per-document progress via WebSocket. This hook subscribes to those events
 * so the UI can show a real-time progress bar and per-document list.
 *
 * @implements SPEC-050: Bulk delete progress (AC-050-05).
 */
'use client';

import { getWebSocketClient } from '@/lib/websocket';
import type { BulkDeletionCompletedEvent, BulkDeletionItemProgressEvent, BulkDeletionStartedEvent, WebSocketProgressMessage } from '@/types/ingestion';
import { useCallback, useEffect, useState } from 'react';

export interface BulkDeletionItemState {
  document_id: string;
  entities_removed: number;
  relationships_removed: number;
}

export interface BulkDeletionProgressState {
  total: number;
  completed: number;
  skipped: number;
  items: BulkDeletionItemState[];
  isComplete: boolean;
  totalEntitiesRemoved: number;
  totalRelationshipsRemoved: number;
}

/**
 * Tracks bulk deletion progress from WebSocket events.
 *
 * @param enabled - Whether to listen for events (e.g. only while dialog is open and mutation is pending)
 */
export function useBulkDeletionProgress(enabled: boolean): BulkDeletionProgressState & { reset: () => void } {
  const [state, setState] = useState<BulkDeletionProgressState>({
    total: 0,
    completed: 0,
    skipped: 0,
    items: [],
    isComplete: false,
    totalEntitiesRemoved: 0,
    totalRelationshipsRemoved: 0,
  });

  const reset = useCallback(() => {
    setState({
      total: 0,
      completed: 0,
      skipped: 0,
      items: [],
      isComplete: false,
      totalEntitiesRemoved: 0,
      totalRelationshipsRemoved: 0,
    });
  }, []);

  useEffect(() => {
    if (!enabled) return;

    const client = getWebSocketClient();

    const handleMessage = (message: WebSocketProgressMessage) => {
      if (message.type === 'BulkDeletionStarted') {
        const ev = message as BulkDeletionStartedEvent;
        setState((prev) => ({ ...prev, total: ev.data.total, completed: 0, items: [], isComplete: false }));
      } else if (message.type === 'BulkDeletionItemProgress') {
        const ev = message as BulkDeletionItemProgressEvent;
        setState((prev) => ({
          ...prev,
          completed: ev.data.completed,
          total: ev.data.total,
          items: [
            ...prev.items.filter((i) => i.document_id !== ev.data.document_id),
            {
              document_id: ev.data.document_id,
              entities_removed: ev.data.entities_removed,
              relationships_removed: ev.data.relationships_removed,
            },
          ],
        }));
      } else if (message.type === 'BulkDeletionCompleted') {
        const ev = message as BulkDeletionCompletedEvent;
        setState((prev) => ({
          ...prev,
          completed: ev.data.deleted_count,
          skipped: ev.data.skipped_count,
          totalEntitiesRemoved: ev.data.total_entities_removed,
          totalRelationshipsRemoved: ev.data.total_relationships_removed,
          isComplete: true,
        }));
      }
    };

    // Listen for all progress events from the global WS client.
    // WHY: The WS client emits all messages (including deletion) as "progress".
    client.on('progress', handleMessage as (...args: unknown[]) => void);

    return () => {
      client.off('progress', handleMessage as (...args: unknown[]) => void);
    };
  }, [enabled]);

  return { ...state, reset };
}
